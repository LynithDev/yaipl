use crate::{error, errors::{DynamicError, EvaluatorError}, parser::ast::{ArithmeticOperator, Assignment, BinaryExpression, BlockStatement, Expression, ForStatement, FunctionCallExpression, FunctionDeclareExpression, Identifier, IfStatement, Literal, LogicalOperator, Node, Operator, ReturnStatement, UnaryExpression, WhileStatement}};

use self::{environment::Environment, object::{Object, ObjectType}};

pub mod environment;
pub mod object;
pub mod yaipl_std;

pub type EvaluatorResult<T> = Result<T, DynamicError>;
pub type StatementResult<T> = EvaluatorResult<(T, bool)>;

pub struct Evaluator<'a> {
    env: Environment<'a>,
    ast: &'a Vec<Node>,
}

impl<'a> Evaluator<'a> {
    pub fn with_env(ast: &'a Vec<Node>, env: Environment<'a>) -> Self {
        Self {
            env,
            ast
        }
    }

    pub fn new(ast: &'a Vec<Node>) -> Self {
        let mut env = Environment::new();
        yaipl_std::initialize(&mut env);

        Self::with_env(ast, env)
    }

    pub fn eval(&mut self) -> Result<Object, DynamicError> {
        let mut result: (Object, bool) = (Object::void(), false);
        
        for node in self.ast {
            result = self.eval_statement(node)?;
            if result.1 {
                break;
            }
        }

        Ok(result.0)
    }

    fn eval_statement(&mut self, node: &'a Node) -> StatementResult<Object> {
        match node {
            Node::BlockStatement(block) => self.eval_block(block),
            Node::BreakStatement(_) => Ok((Object::void(), true)),
            Node::ContinueStatement(_) => Ok((Object::void(), true)),
            Node::EmptyStatement(_) => Ok((Object::void(), false)),
            Node::ExpressionStatement(expr) => Ok((self.eval_expression(&expr.0)?, false)),
            Node::IfStatement(statement) => self.eval_if(statement),
            Node::ElseStatement(statement) => self.eval_block(&statement.0),
            Node::ReturnStatement(statement) => self.eval_return(statement),
            Node::WhileStatement(statement) => self.eval_while(statement),
            Node::ForStatement(statement) => self.eval_for(statement),
            _ => error!(format!("Not implemented statement {:#?}", node))
        }
    }

    fn eval_for(&mut self, statement: &'a ForStatement) -> StatementResult<Object> {
        let ForStatement(setter, condition, assignment, body) = statement;

        let setter = match setter {
            Expression::AssignmentExpr(setter) => setter,
            _ => error!(EvaluatorError::InvalidExpression { 
                expected: String::from("AssignmentExpr")
            })
        };

        let condition = match condition {
            Expression::BinaryExpr(condition) => condition,
            _ => error!(EvaluatorError::InvalidExpression { 
                expected: String::from("BinaryExpr")
            })
        };

        let assignment = match assignment {
            Expression::AssignmentExpr(assignment) => assignment,
            _ => error!(EvaluatorError::InvalidExpression { 
                expected: String::from("AssignmentExpr")
            })
        };

        let mut result = (Object::void(), false);

        let scope_size = self.new_scope();
        self.eval_assignment_expression(setter)?;
        while self.eval_binary_expression(condition)?.as_boolean().expect("Couldn't take as boolean") {
            result = self.eval_block(body)?;
            
            if result.1 {
                break;
            }

            self.eval_assignment_expression(assignment)?;
        }
        self.destroy_scope(scope_size);

        Ok(result)
    }

    fn eval_while(&mut self, statement: &'a WhileStatement) -> StatementResult<Object> {
        let WhileStatement(condition, block) = statement;
        let mut result = (Object::void(), false);

        let scope_size = self.new_scope();
        while self.eval_expression(condition)?.as_boolean().expect("Couldn't take as boolean") {
            result = self.eval_block(block)?;
            if result.1 {
                break;
            }
        }
        self.destroy_scope(scope_size);

        Ok(result)
    }

    fn eval_return(&mut self, statement: &'a ReturnStatement) -> StatementResult<Object> {
        let ReturnStatement(expression) = statement;
        let mut result = Object::void();

        if let Some(expression) = expression {
            result = self.eval_expression(expression)?;
        }

        Ok((result, true))
    }

    fn eval_if(&mut self, statement: &'a IfStatement) -> StatementResult<Object> {
        let IfStatement(condition, block, elif) = statement;
        let condition = self.eval_expression(condition)?;

        if condition.is(ObjectType::Boolean) {
            let scope_size = self.new_scope();

            let result = if condition.as_boolean().expect("Couldn't take as boolean") {
                self.eval_block(block)
            } else if let Some(elif) = elif {
                self.eval_statement(&elif)
            } else {
                Ok((Object::void(), false))
            };

            self.destroy_scope(scope_size);
            return result;
        };

        Ok((Object::void(), false))
    }

    fn eval_expression(&mut self, expression: &'a Expression) -> EvaluatorResult<Object> {
        Ok(match expression {
            Expression::AssignmentExpr(expression) => self.eval_assignment_expression(expression)?,
            Expression::BinaryExpr(expression) => self.eval_binary_expression(expression)?,
            Expression::BlockExpr(expression) => (self.eval_block(expression)?).0,
            Expression::FunctionCallExpr(expression) => self.eval_func_call_expression(expression)?,
            Expression::FunctionDeclareExpr(expression) => self.eval_func_declare_expression(expression)?,
            Expression::GroupExpr(expression) => self.eval_expression(expression)?,
            Expression::IdentifierExpr(expression) => self.eval_identifier(expression)?,
            Expression::LiteralExpr(expression) => self.eval_literal(expression)?,
            Expression::UnaryExpr(expression) => self.eval_unary_expression(expression)?,
        })
    }

    fn new_scope(&self) -> usize {
        self.env.size()
    }

    fn destroy_scope(&mut self, size: usize) {
        self.env.truncate(size);
    }

    fn eval_func_call_expression(&mut self, expression: &'a FunctionCallExpression) -> EvaluatorResult<Object> {
        let FunctionCallExpression(identifier, args) = expression;
        let object = self.env.get(&identifier.0);

        if let Some(object) = object {
            let object = object.to_owned();

            let mut built_args: Vec<Object> = Vec::new();
            for arg in args {
                built_args.push(self.eval_expression(arg)?);
            }
            
            let result = match object.get_type() {
                ObjectType::Function => {
                    let function = object.as_function().expect("Couldn't take as function");
            
                    let scope_size = self.new_scope();
                    for (index, arg) in built_args.iter().enumerate() {
                        let identifier = &function.1[index];
                        self.env.set(&identifier.0, arg.to_owned());
                    }
                    
                    let result = self.eval_block(&function.2)?;
                    self.destroy_scope(scope_size);
                    result
                },
                ObjectType::NativeFunction => {
                    let function = object.as_native_function().expect("Couldn't take as natve function");
                    
                    ((function.2)(&mut self.env, built_args), false)
                },
                _ => error!(EvaluatorError::InvalidType { 
                    expected: vec![ObjectType::Function, ObjectType::NativeFunction],
                    found: object.get_type(),
                })
            };

            return Ok(result.0);
        }
        
        error!(EvaluatorError::ObjectNotFound { name: identifier.0.to_owned() })
    }

    fn eval_block(&mut self, expression: &'a BlockStatement) -> StatementResult<Object> {
        let mut result = (Object::void(), false);
        
        for statement in &expression.0 {
            result = self.eval_statement(statement)?;
            if result.1 {
                break;
            }
        }

        Ok(result)
    }

    fn eval_func_declare_expression(&mut self, expression: &'a FunctionDeclareExpression) -> EvaluatorResult<Object> {
        let object = Object::function(expression);
        self.env.set(&expression.0.0, object);

        Ok(Object::void())
    }

    fn eval_identifier(&self, expression: &Identifier) -> EvaluatorResult<Object> {
        let Identifier(identifier) = expression;
        
        match self.env.get(identifier) {
            Some(object) => Ok(object.to_owned()),
            None => error!(EvaluatorError::ObjectNotFound { name: identifier.to_owned() })
        }
    }

    fn eval_assignment_expression(&mut self, expression: &'a Assignment) -> EvaluatorResult<Object> {
        let Assignment(identifier, literal) = expression;

        let value = self.eval_statement(&literal)?.0;
        self.env.set(&identifier.0, value);
        Ok(Object::void())
    }

    fn eval_literal(&self, expression: &Literal) -> EvaluatorResult<Object> {
        Ok(match expression {
            Literal::Integer(num) => Object::integer(num.0),
            Literal::Boolean(bool) => Object::boolean(bool.0),
            Literal::Float(num) => Object::float(num.0),
            Literal::String(str) => Object::string(&str.0),
            Literal::List(list) => Object::list(&list.0),
            Literal::Null => Object::null(),
        })
    }

    fn eval_unary_expression(&mut self, expression: &'a UnaryExpression) -> EvaluatorResult<Object> {
        let UnaryExpression(operator, expr) = expression;

        let object = self.eval_expression(expr)?;
        if operator == &Operator::Logical(LogicalOperator::Not) {
            if object.is(ObjectType::Boolean) {
                return Ok(object);
            }
        }

        if operator == &Operator::Arithmetic(ArithmeticOperator::Minus) {
            return Ok(match object.get_type() {
                ObjectType::Integer => Object::integer(-object.as_integer().expect("Couldn't take as integer")),
                ObjectType::Float => Object::float(-object.as_f32().expect("Couldn't take as float")),
                _ => error!(EvaluatorError::InvalidType { 
                    expected: vec![ObjectType::Integer, ObjectType::Float],
                    found: object.get_type(),
                })
            });
        }

        error!(EvaluatorError::InvalidExpression { 
            expected: String::from("UnaryExpression")
        })
    }

    fn eval_binary_expression(&mut self, expression: &'a BinaryExpression) -> EvaluatorResult<Object> {
        let BinaryExpression(left, operator, right) = expression;

        let lhs = self.eval_expression(left)?;
        let rhs = self.eval_expression(right)?;

        let result = match operator {
            Operator::Logical(op) => match op {
                LogicalOperator::Or => lhs.or(rhs),
                LogicalOperator::Not => error!("Not operator not implemented for binary expression"),
                LogicalOperator::And => lhs.and(rhs),
                LogicalOperator::Equal => lhs.equal(rhs),
                LogicalOperator::NotEqual => lhs.not_equal(rhs),
                LogicalOperator::GreaterThan => lhs.greater_than(rhs),
                LogicalOperator::GreaterThanEqual => lhs.greater_than_equal(rhs),
                LogicalOperator::LesserThan => lhs.lesser_than(rhs),
                LogicalOperator::LesserThanEqual => lhs.lesser_than_equal(rhs),
            },
            Operator::Arithmetic(op) => match op {
                ArithmeticOperator::Plus => lhs.add(rhs),
                ArithmeticOperator::Minus => lhs.subtract(rhs),
                ArithmeticOperator::Multiply => lhs.multiply(rhs),
                ArithmeticOperator::Divide => lhs.divide(rhs),
                ArithmeticOperator::Modulo => lhs.modulo(rhs),
                ArithmeticOperator::Power => lhs.power(rhs),
            },
            _ => error!(format!("Not implemented {:#?}", operator))
        };

        match result {
            Ok(object) => Ok(object),
            Err(err) => return Err(err.into())
        }

    }
}

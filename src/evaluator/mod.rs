
use std::error::Error;

use crate::{create_error_list, error, parser::ast::{ArithmeticOperator, Assignment, BinaryExpression, BlockStatement, Expression, FunctionCallExpression, FunctionDeclareExpression, Identifier, IfStatement, Literal, LogicalOperator, Node, Operator, ReturnStatement, UnaryExpression, WhileStatement}};

use self::{environment::Environment, object::{Object, ObjectType}};

pub mod environment;
pub mod object;
pub mod yaipl_std;

create_error_list!(EvaluatorErrors, {});
pub type EvaluatorResult<T> = Result<T, Box<dyn Error>>;
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

    pub fn eval(&'a mut self) -> Result<Object, EvaluatorErrors> {
        let mut result: Option<(Object, bool)> = None;
        
        for node in self.ast {
            result = Some(self.eval_statement(node)?);
        }

        Ok(result.unwrap_or((Object::void(), false)).0)
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
            _ => error!(format!("Not implemented statement {:#?}", node))
        }
    }

    fn eval_while(&mut self, statement: &'a WhileStatement) -> StatementResult<Object> {
        let WhileStatement(condition, block) = statement;
        let mut result = (Object::void(), false);

        let mut evaluator = self.new_scope();
        while evaluator.eval_expression(condition)?.as_boolean().expect("Couldn't take as boolean") {
            result = evaluator.eval_block(block)?;
            if result.1 {
                break;
            }
        }
        self.destroy_scope(evaluator);

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
            if condition.as_boolean().expect("Couldn't take as boolean") {
                return self.eval_block(block);
            }

            if let Some(elif) = elif {
                return self.eval_statement(&elif);
            }
        }

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

    fn new_scope(&self) -> Evaluator {
        Evaluator::with_env(self.ast, self.env.clone())
    }

    fn destroy_scope(&self, evaluator: Evaluator) {
        std::mem::drop(evaluator)
    }

    fn eval_func_call_expression(&mut self, expression: &'a FunctionCallExpression) -> EvaluatorResult<Object> {
        let FunctionCallExpression(identifier, args) = expression;
        let object = self.env.get(&identifier.0);
        
        if let Some(object) = object {
            let object = object.to_owned();
            
            let result = match object.get_type() {
                ObjectType::Function => {
                    let function = object.as_function().expect("Couldn't take as function");
                    let mut built_args: Vec<Object> = Vec::new();
                    for arg in args {
                        built_args.push(self.eval_expression(arg)?);
                    }
            
                    let mut scope = self.new_scope();
                    for (index, arg) in built_args.iter().enumerate() {
                        let identifier = &function.1[index];
                        scope.env.set(&identifier.0, arg.to_owned());
                    }
                    
                    let result = scope.eval_block(&function.2)?;
                    self.destroy_scope(scope);
                    result
                },
                ObjectType::NativeFunction => {
                    let function = object.as_native_function().expect("Couldn't take as natve function");
                    let mut built_args: Vec<Object> = Vec::new();
                    for arg in args {
                        built_args.push(self.eval_expression(arg)?);
                    }
                    
                    ((function.2)(built_args), false)
                },
                _ => error!("not function :(")
            };

            return Ok(result.0);
        }
        
        error!("Couldn't find fucntion")
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
            None => error!("Unidentified variable")
        }
    }

    fn eval_assignment_expression(&mut self, expression: &'a Assignment) -> EvaluatorResult<Object> {
        let Assignment(identifier, literal) = expression;

        let value = self.eval_expression(&literal)?;
        self.env.set(&identifier.0, value);
        Ok(Object::void())
    }

    fn eval_literal(&self, expression: &Literal) -> EvaluatorResult<Object> {
        Ok(match expression {
            Literal::Integer(num) => Object::integer(num.0),
            Literal::Boolean(bool) => Object::boolean(if bool.0 <= 0 { false } else { true }),
            Literal::Float(num) => Object::float(num.0),
            Literal::String(str) => Object::string(&str.0),
        })
    }

    fn eval_unary_expression(&mut self, expression: &'a UnaryExpression) -> EvaluatorResult<Object> {
        let UnaryExpression(operator, expr) = expression;

        let object = self.eval_expression(expr)?;
        if operator == &Operator::Logical(LogicalOperator::Not) {
            if object.is(ObjectType::Boolean) {
                return Ok(Object::boolean(!object.as_boolean().expect("Couldn't take as boolean")));
            }
        }

        if operator == &Operator::Arithmetic(ArithmeticOperator::Minus) {
            return Ok(match object.get_type() {
                ObjectType::Integer => Object::integer(-object.as_integer().expect("Couldn't take as integer")),
                ObjectType::Float => Object::float(-object.as_f32().expect("Couldn't take as float")),
                _ => error!("Not implemented")
            });
        }

        error!("Not implemented")
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
            Err(err) => error!(format!("{:?}", err))
        }

    }
}

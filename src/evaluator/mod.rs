use std::error::Error;

use crate::{create_error_list, error, parser::ast::{ArithmeticOperator, Assignment, BinaryExpression, Expression, FunctionCallExpression, FunctionDeclareExpression, Identifier, IfStatement, LogicalExpression, LogicalOperator, Node, Operator, Program, ProgramTree, ReturnStatement, WhileStatement}};

use self::{environment::Environment, object::{FunctionObject, ObjectValue}};

pub mod environment;
pub mod object;
pub mod yaipl_std;

pub struct Evaluator {
    global_functions: Vec<ObjectValue>,
    global_env: Environment,
    env_stack: Vec<Environment>,
    ast: ProgramTree,
}

create_error_list!(EvaluatorErrors, {});

type EvaluatorResult<T> = Result<T, Box<dyn Error>>; 
impl Evaluator {
    pub fn new(ast: Program) -> Self {
        let ast = match ast {
            Node::Program(ast) => ast,
            _ => panic!("Invalid AST")
        };

        Self {
            global_env: Environment::new(),
            global_functions: yaipl_std::initialize(),
            env_stack: Vec::new(),
            ast,
        }
    }

    pub fn eval(&mut self) -> Result<ObjectValue, EvaluatorErrors> {
        let mut result = (ObjectValue::Void, false);
        for statement in &self.ast.to_owned() {
            result = self.eval_statement(statement)?;
            if result.1 {
                break;
            }
        }

        Ok(result.0)
    }

    fn get_env(&self) -> &Environment {
        match self.env_stack.last() {
            Some(env) => env,
            None => &self.global_env
        }
    }

    fn get_env_mut(&mut self) -> &mut Environment {
        match self.env_stack.last_mut() {
            Some(env) => env,
            None => &mut self.global_env
        }
    }

    fn start_env(&mut self) {
        self.env_stack.push(Environment::with_parent(Box::new(self.get_env().clone())));
    }

    fn end_env(&mut self) {
        if let Some(mut env) = self.env_stack.pop() {
            env.destroy();
        }
        
    }

    fn eval_statement(&mut self, statement: &Node) -> EvaluatorResult<(ObjectValue, bool)> {
        Ok(match statement {
            Node::EmptyStatement(_) => (ObjectValue::Void, false),
            Node::ExpressionStatement(expr) => (self.eval_expression(&expr.0)?, false),
            Node::ReturnStatement(expr) => (self.eval_return(&expr)?, true),
            Node::WhileStatement(expr) => self.eval_while(&expr)?,
            Node::IfStatement(expr) => self.eval_if(&expr)?,
            _ => error!(format!("Not implemented eval_statement for {:?}", statement))
        })
    }

    fn eval_if(&mut self, expr: &IfStatement) -> EvaluatorResult<(ObjectValue, bool)> {
        let IfStatement(condition, block) = expr;
        
        let condition = self.eval_expression(condition)?;
        if condition == ObjectValue::Boolean(1) {
            self.start_env();
            let result = self.eval_block(&block.0)?;
            self.end_env();
            return Ok(result);
        }

        Ok((ObjectValue::Void, false))
    }

    fn eval_while(&mut self, expr: &WhileStatement) -> EvaluatorResult<(ObjectValue, bool)> {
        let WhileStatement(condition, block) = expr;
        let mut result = (ObjectValue::Void, false);

        self.start_env();
        while self.eval_expression(condition)? == ObjectValue::Boolean(1) {
            // result = self.eval_block(&block.0)?;
            let eval_result = self.eval_block(&block.0)?;
            if eval_result.1 {
                result = eval_result;
                break;
            }
        }
        self.end_env();

        Ok(result)
    }

    fn eval_return(&mut self, expr: &ReturnStatement) -> EvaluatorResult<ObjectValue> {
        if let Some(expr) = &expr.0 {
            self.eval_expression(expr)
        } else {
            Ok(ObjectValue::Void)
        }
    }

    fn eval_expression(&mut self, expr: &Expression) -> EvaluatorResult<ObjectValue> {
        Ok(match expr {
            Expression::IdentifierExpr(identifier) => self.eval_identifier(identifier)?,
            Expression::LiteralExpr(literal) => ObjectValue::from(literal.to_owned()),
            Expression::GroupExpr(group) => self.eval_group_expr(group)?,
            Expression::BinaryExpr(expr) => self.eval_binary_expr(expr)?,
            Expression::LogicalExpr(expr) => self.eval_logical_expr(expr)?,
            
            Expression::AssignmentExpr(assignment) => self.eval_assignment_expr(assignment)?,
            Expression::FunctionDeclareExpr(func) => self.eval_function_declare(func)?,
            Expression::FunctionCallExpr(func) => self.eval_call(func)?,
            _ => error!(format!("Not implemented eval_expression for {:?}", expr))
        })
    }

    fn eval_call(&mut self, func: &FunctionCallExpression) -> EvaluatorResult<ObjectValue> {
        let FunctionCallExpression(
            identifier, 
            call_params
        ) = func;

        let func = match self.get_env().get_func(&identifier.0) {
            Some(func) => &func.value,
            None => {
                match yaipl_std::get_native_function(&self.global_functions, &identifier.0) {
                    Some(func) => func,
                    None => error!(format!("Function '{}' not found", identifier.0))
                }
            }
        };

        let result = match func.to_owned() {
            ObjectValue::Function(func) => {
                let block = func.body;

                self.start_env();

                for (param, value) in func.params.iter().zip(call_params.iter()) {
                    let value = self.eval_expression(value)?;
                    self.get_env_mut().set(param.to_owned(), value);
                }

                let (result, _) = self.eval_block(unsafe {
                    &block.as_ref().take().unwrap().0
                })?;
                self.end_env();
                result
            },
            ObjectValue::NativeFunction(func) => {
                let mut objects: Vec<ObjectValue> = Vec::new();

                for param in call_params {
                    let value = self.eval_expression(param)?;
                    objects.push(value);
                }

                (func.body)(objects)
            }
            _ => error!("Invalid function")
        };

        Ok(result)
    }

    fn eval_block(&mut self, block: &Vec<Node>) -> EvaluatorResult<(ObjectValue, bool)> {
        let mut result = (ObjectValue::Void, false);

        for statement in block {
            result = self.eval_statement(&statement)?;
            if result.1 {
                // println!("Early exit");
                break;
            }
        }

        Ok(result)
    }

    fn eval_function_declare(&mut self, func: &FunctionDeclareExpression) -> EvaluatorResult<ObjectValue> {
        let crate::parser::ast::FunctionDeclareExpression(
            identifier, 
            params, 
            block
        ) = func;

        let params: Vec<String> = params.to_owned().into_iter().map(|param| param.0).collect();
        let object = ObjectValue::Function(FunctionObject::new(params, &*block));
        self.get_env_mut().set(identifier.0.to_owned(), object);

        Ok(ObjectValue::Void)
    }

    fn eval_identifier(&mut self, identifier: &Identifier) -> EvaluatorResult<ObjectValue> {
        let object = self.get_env().get_result(&identifier.0)?;
        Ok(object.value.to_owned())
    }

    fn eval_assignment_expr(&mut self, expr: &Assignment) -> EvaluatorResult<ObjectValue> {
        let Assignment(
            identifier, 
            value
        ) = expr;

        let value = self.eval_expression(value)?;
        self.get_env_mut().set(identifier.0.to_owned(), value.to_owned());
        Ok(value)
    }

    fn eval_group_expr(&mut self, expr: &Expression) -> EvaluatorResult<ObjectValue> {
        self.eval_expression(expr)
    }

    fn eval_binary_expr(&mut self, expr: &BinaryExpression) -> EvaluatorResult<ObjectValue> {
        let BinaryExpression(
            left, 
            op, 
            right
        ) = expr;
        
        let op = match op {
            Operator::Arithmetic(op) => op,
            Operator::Logical(op) => {
                return self.eval_logical_expr(&LogicalExpression(left.to_owned(), op.to_owned(), right.to_owned()));
            },
            _ => error!("Invalid operator")
        };

        let left = self.eval_expression(left)?;
        let right = self.eval_expression(right)?;

        macro_rules! operator_impl {
            ($op:tt $(, $pat:pat => $result:expr)*) => {
                match (left, right) {
                    (ObjectValue::Integer(l), ObjectValue::Integer(r)) => ObjectValue::Integer(l $op r),
                    (ObjectValue::Integer(l), ObjectValue::Float(r)) => ObjectValue::Float((l as f32) $op r),
                    (ObjectValue::Float(l), ObjectValue::Integer(r)) => ObjectValue::Float(l $op (r as f32)),
                    (ObjectValue::Float(l), ObjectValue::Float(r)) => ObjectValue::Float(l $op r),
                    $($pat => $result,)*
                    _ => error!(format!("Invalid types for operator {:?}", op))
                }
            };
        }

        Ok(match op {
            ArithmeticOperator::Plus => operator_impl!(+, 
                (ObjectValue::String(l), r) => ObjectValue::String(format!("{}{}", l, r.to_string())),
                (l, ObjectValue::String(r)) => ObjectValue::String(format!("{}{}", l.to_string(), r))
            ),
            ArithmeticOperator::Minus => operator_impl!(-),
            ArithmeticOperator::Multiply => operator_impl!(*),
            ArithmeticOperator::Divide => operator_impl!(/),
            ArithmeticOperator::Modulo => operator_impl!(%),
            ArithmeticOperator::Power => match (left, right) {
                (ObjectValue::Integer(l), ObjectValue::Integer(r)) => ObjectValue::Integer(l.pow(r.try_into()?)),
                (ObjectValue::Integer(l), ObjectValue::Float(r)) => ObjectValue::Float((l as f32).powf(r)),
                (ObjectValue::Float(l), ObjectValue::Integer(r)) => ObjectValue::Float(l.powf(r as f32)),
                (ObjectValue::Float(l), ObjectValue::Float(r)) => ObjectValue::Float(l.powf(r)),
                _ => error!(format!("Invalid types for operator {:?}", op))
            },
        })
    }

    fn eval_logical_expr(&mut self, expr: &LogicalExpression) -> EvaluatorResult<ObjectValue> {
        let LogicalExpression(
            left, 
            op, 
            right
        ) = expr;

        let left = self.eval_expression(left)?;
        let right = self.eval_expression(right)?;

        macro_rules! comparison {
            ($op:tt $(, $pat:pat => $result:expr)*) => {
                match (left, right) {
                    (ObjectValue::Integer(l), ObjectValue::Integer(r)) => ObjectValue::Boolean(if l $op r { 1 } else { 0 }),
                    (ObjectValue::Float(l), ObjectValue::Float(r)) => ObjectValue::Boolean(if l $op r { 1 } else { 0 }),
                    (ObjectValue::Integer(l), ObjectValue::Float(r)) => ObjectValue::Boolean(if (l as f32) $op r { 1 } else { 0 }),
                    (ObjectValue::Float(l), ObjectValue::Integer(r)) => ObjectValue::Boolean(if l $op (r as f32) { 1 } else { 0 }),
                    $($pat => $result,)*
                    _ => error!("Invalid types for operator <")
                }
            };
        }

        Ok(match op {
            LogicalOperator::Equal => {
                match (left, right) {
                    (ObjectValue::Integer(l), ObjectValue::Integer(r)) => ObjectValue::Boolean(if l == r { 1 } else { 0 }),
                    (ObjectValue::Float(l), ObjectValue::Float(r)) => ObjectValue::Boolean(if l == r { 1 } else { 0 }),
                    (ObjectValue::Boolean(l), ObjectValue::Boolean(r)) => ObjectValue::Boolean(if l == r { 1 } else { 0 }),
                    (ObjectValue::String(l), ObjectValue::String(r)) => ObjectValue::Boolean(if l == r { 1 } else { 0 }),
                    _ => error!("Invalid types for operator ==")
                }
            },
            LogicalOperator::NotEqual => {
                match (left, right) {
                    (ObjectValue::Integer(l), ObjectValue::Integer(r)) => ObjectValue::Boolean(if l != r { 1 } else { 0 }),
                    (ObjectValue::Float(l), ObjectValue::Float(r)) => ObjectValue::Boolean(if l != r { 1 } else { 0 }),
                    (ObjectValue::Boolean(l), ObjectValue::Boolean(r)) => ObjectValue::Boolean(if l != r { 1 } else { 0 }),
                    (ObjectValue::String(l), ObjectValue::String(r)) => ObjectValue::Boolean(if l != r { 1 } else { 0 }),
                    _ => error!("Invalid types for operator !=")
                }
            },
            LogicalOperator::And => {
                match (left, right) {
                    (ObjectValue::Boolean(l), ObjectValue::Boolean(r)) => ObjectValue::Boolean(if l == 1 && r == 1 { 1 } else { 0 }),
                    _ => error!("Invalid types for operator &&")
                }
            },
            LogicalOperator::Or => {
                match (left, right) {
                    (ObjectValue::Boolean(l), ObjectValue::Boolean(r)) => ObjectValue::Boolean(if l == 1 || r == 1 { 1 } else { 0 }),
                    _ => error!("Invalid types for operator ||")
                }
            },
            LogicalOperator::Not => {
                match left {
                    ObjectValue::Boolean(l) => ObjectValue::Boolean(if l == 1 { 0 } else { 1 }),
                    _ => error!("Invalid types for operator !")
                }
            },
            LogicalOperator::LesserThan => comparison!(<),
            LogicalOperator::LesserThanEqual => comparison!(<=),
            LogicalOperator::GreaterThan => comparison!(>),
            LogicalOperator::GreaterThanEqual => comparison!(>=),
        })
    }

}
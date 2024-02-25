use std::error::Error;

use crate::{create_error_list, error, parser::ast::{ArithmeticOperator, Assignment, BinaryExpression, Expression, FunctionCallExpression, FunctionDeclareExpression, Identifier, Node, Operator, Program, ProgramTree}};

use self::{environment::Environment, object::{FunctionObject, ObjectValue}};

pub mod environment;
pub mod object;
pub mod yaipl_std;

pub struct Evaluator {
    global_env: Environment,
    env_stack: Vec<Environment>,
    ast: ProgramTree
}

create_error_list!(EvaluatorErrors, {});

type EvaluatorResult<T> = Result<T, Box<dyn Error>>; 
impl Evaluator {
    pub fn new(ast: Program) -> Self {
        let ast = match ast {
            Node::Program(ast) => ast,
            _ => panic!("Invalid AST")
        };

        let mut global_env = Environment::new();
        yaipl_std::initialize(&mut global_env);

        Self {
            env_stack: Vec::new(),
            global_env,
            ast
        }
    }

    pub fn eval(&mut self) -> Result<ObjectValue, EvaluatorErrors> {
        let mut result = ObjectValue::Void;
        for statement in &self.ast.to_owned() {
            result = self.eval_statement(statement)?;
        }

        Ok(result)
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
        self.env_stack.pop();
    }

    fn eval_statement(&mut self, statement: &Node) -> EvaluatorResult<ObjectValue> {
        Ok(match statement {
            Node::ExpressionStatement(expr) => self.eval_expression(&expr.0)?,
            _ => ObjectValue::Void
        })
    }

    fn eval_expression(&mut self, expr: &Expression) -> EvaluatorResult<ObjectValue> {
        Ok(match expr {
            Expression::IdentifierExpr(identifier) => self.eval_identifier(identifier)?,
            Expression::LiteralExpr(literal) => ObjectValue::from(literal.to_owned()),
            Expression::GroupExpr(group) => self.eval_group_expr(group)?,
            Expression::BinaryExpr(expr) => self.eval_binary_expr(expr)?,
            
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

        let function = self.get_env().get_function_err(identifier.0.to_owned())?.to_owned();

        let mut result = ObjectValue::Void;
        match &function.value {
            ObjectValue::Function(func) => {
                self.start_env();
                let func = func.to_owned();

                for (param, value) in func.params.iter().zip(call_params.iter()) {
                    let value = self.eval_expression(value)?;
                    self.get_env_mut().set_var(param.to_owned(), value.to_owned());
                }
        
                for statement in func.body.0 {
                    result = self.eval_statement(&statement)?;
                }
                self.end_env();
            },
            ObjectValue::NativeFunction(func) => {
                let mut objects: Vec<ObjectValue> = Vec::new();
                for param in call_params {
                    let value = self.eval_expression(param)?;
                    objects.push(value);
                }

                result = (func.body)(objects);
            }
            _ => error!("Invalid function")
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
        let object = ObjectValue::Function(FunctionObject::new(params.to_owned(), *block.to_owned()));
        self.get_env_mut().set_function(identifier.0.to_owned(), object);

        Ok(ObjectValue::Void)
    }

    fn eval_identifier(&mut self, identifier: &Identifier) -> EvaluatorResult<ObjectValue> {
        let object = self.get_env().get_var_err(identifier.0.to_owned())?;
        Ok(object.value.to_owned())
    }

    fn eval_assignment_expr(&mut self, expr: &Assignment) -> EvaluatorResult<ObjectValue> {
        let Assignment(
            identifier, 
            value
        ) = expr;

        let value = self.eval_expression(value)?;
        self.get_env_mut().set_var(identifier.0.to_owned(), value.to_owned());
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
            Operator::Arithmetic(op) => match op {
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
            }
            _ => error!("Not implemented")
        })
    }

}

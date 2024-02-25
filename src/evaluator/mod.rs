use std::error::Error;

use crate::{create_error_list, error, parser::ast::{self, ArithmeticOperator, BinaryExpression, Expression, ExpressionStatement, Node, Operator, Program, ProgramTree}};

use self::{environment::Environment, object::ObjectValue};

pub mod environment;
pub mod object;

pub struct Evaluator<'a> {
    pub env: Environment<'a>,
    ast: ProgramTree
}

create_error_list!(EvaluatorErrors, {});

type EvaluatorResult<T> = Result<T, Box<dyn Error>>; 
impl Evaluator<'_> {
    pub fn new<'a>(ast: Program) -> Evaluator<'a> {
        let ast = match ast {
            Node::Program(ast) => ast,
            _ => panic!("Invalid AST")
        };

        Evaluator {
            env: Environment::new(),
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

    fn eval_statement(&mut self, statement: &Node) -> EvaluatorResult<ObjectValue> {
        let mut result = ObjectValue::Void;
        
        match statement {
            Node::ExpressionStatement(expr) => {
                result = self.eval_expression(&expr.0)?;
            },
            _ => {}
        }

        Ok(result)
    }

    fn eval_expression(&mut self, expr: &Expression) -> EvaluatorResult<ObjectValue> {
        Ok(match expr {
            Expression::LiteralExpr(literal) => ObjectValue::from(literal.to_owned()),
            Expression::GroupExpr(group) => self.eval_group_expr(&group)?,
            Expression::BinaryExpr(expr) => self.eval_binary_expr(expr)?,
            _ => error!(format!("Not implemented eval_expression for {:?}", expr))
        })
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

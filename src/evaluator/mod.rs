
use std::error::Error;

use crate::{create_error_list, error, parser::ast::{ArithmeticOperator, BinaryExpression, Expression, ExpressionStatement, Node, Operator, Program, ProgramTree}};

use self::{environment::Environment, object::Object};

pub mod environment;
pub mod object;
pub mod yaipl_std;

create_error_list!(EvaluatorErrors, {});
pub type EvaluatorResult<T> = Result<T, Box<dyn Error>>;

pub struct Evaluator<'a> {
    env: Environment<'a>,
    ast: &'a Vec<Node>,
}

impl<'a> Evaluator<'a> {
    pub fn new(ast: &'a Vec<Node>) -> Self {
        Self { 
            env: Environment::new(),
            ast: &ast,
        }
    }

    pub fn eval(&'a mut self) -> EvaluatorResult<Object> {
        let mut result: Option<Object> = None;
        
        for node in self.ast {
            result = Some(self.eval_statement(node)?);
        }

        Ok(result.unwrap_or(Object::void()))
    } 

    fn eval_statement(&mut self, node: &Node) -> EvaluatorResult<Object> {
        match node {
            Node::ExpressionStatement(expr) => self.eval_expression(&expr.0),
            _ => Ok(Object::void())
        }
    }

    fn eval_expression(&mut self, expression: &Expression) -> EvaluatorResult<Object> {
        Ok(match expression {
            Expression::BinaryExpr(expression) => self.eval_binary_expression(expression)?,
            _ => error!(format!("Not implemented {:#?}", expression))
        })
    }

    fn eval_binary_expression(&mut self, expression: &BinaryExpression) -> EvaluatorResult<Object> {
        let BinaryExpression(left, operator, right) = expression;

        let lhs = self.eval_expression(left)?;
        let rhs = self.eval_expression(right)?;

        match operator {
            Operator::Arithmetic(op) => match op {
                ArithmeticOperator::Plus => lhs.add(rhs),
                _ => error!(format!("Not implemented {:#?}", op))
            },
            _ => error!(format!("Not implemented {:#?}", operator))
        }
    }
}

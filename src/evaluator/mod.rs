
use std::error::Error;

use crate::{create_error_list, error, parser::ast::{ArithmeticOperator, Assignment, BinaryExpression, BlockStatement, Expression, FunctionCallExpression, FunctionDeclareExpression, Identifier, Literal, Node, Operator}};

use self::{environment::Environment, object::{Object, ObjectType}};

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
    pub fn with_env(ast: &'a Vec<Node>, env: Environment<'a>) -> Self {
        Self {
            env,
            ast
        }
    }

    pub fn new(ast: &'a Vec<Node>) -> Self {
        let env = Environment::new();
        
        for func in yaipl_std::initialize() {
            if let 
        }

        Self::with_env(ast, )
    }

    pub fn eval(&'a mut self) -> Result<Object, EvaluatorErrors> {
        let mut result: Option<Object> = None;
        
        for node in self.ast {
            result = Some(self.eval_statement(node)?);
        }

        Ok(result.unwrap_or(Object::void()))
    } 

    fn eval_statement(&mut self, node: &'a Node) -> EvaluatorResult<Object> {
        match node {
            Node::EmptyStatement(_) => Ok(Object::void()),
            Node::ExpressionStatement(expr) => self.eval_expression(&expr.0),
            _ => error!(format!("Not implemented statement {:?}", node))
        }
    }

    fn eval_expression(&mut self, expression: &'a Expression) -> EvaluatorResult<Object> {
        Ok(match expression {
            Expression::AssignmentExpr(expression) => self.eval_assignment_expression(expression)?,
            Expression::IdentifierExpr(expression) => self.eval_identifier(expression)?,
            Expression::LiteralExpr(expression) => self.eval_literal(expression)?,
            Expression::BinaryExpr(expression) => self.eval_binary_expression(expression)?,
            Expression::FunctionDeclareExpr(expression) => self.eval_func_declare_expression(expression)?,
            Expression::FunctionCallExpr(expression) => self.eval_func_call_expression(expression)?,
            _ => error!(format!("Not implemented {:#?}", expression))
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
        let mut result = Object::void();
        let object = self.env.get(&identifier.0);

        if let Some(object) = object {
            let object = object.to_owned();
            
            match object.get_type() {
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
                    
                    result = scope.eval_block(&function.2)?;
                    self.destroy_scope(scope);
                },
                ObjectType::NativeFunction => {
                    let function = object.as_native_function().expect("Couldn't take as natve function");
                    let mut built_args: Vec<Object> = Vec::new();
                    for arg in args {
                        built_args.push(self.eval_expression(arg)?);
                    }
                    
                    (function.2)(built_args);
                },
                _ => error!("not function :(")
            }
        }
        
        Ok(result)
    }

    fn eval_block(&mut self, expression: &'a BlockStatement) -> EvaluatorResult<Object> {
        let mut result = Object::void();
        
        for statement in &expression.0 {
            result = self.eval_statement(statement)?;
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

    fn eval_binary_expression(&mut self, expression: &'a BinaryExpression) -> EvaluatorResult<Object> {
        let BinaryExpression(left, operator, right) = expression;

        let lhs = self.eval_expression(left)?;
        let rhs = self.eval_expression(right)?;

        let result = match operator {
            Operator::Arithmetic(op) => match op {
                ArithmeticOperator::Plus => lhs.add(rhs),
                ArithmeticOperator::Minus => lhs.subtract(rhs),
                ArithmeticOperator::Multiply => lhs.subtract(rhs),
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

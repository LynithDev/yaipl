use std::{error::Error, fmt::Display};

use crate::{lexer::token::Token, parser::ast::{Operator, Program}};
use self::ast::{Expression, Literal, Statement};

pub mod ast;

macro_rules! create_error {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name {
            pub err: String
        }

        impl $name {
            pub fn from_str(err: &str) -> Self {
                Self {
                    err: err.to_string()
                }
            }

            pub fn from(err: String) -> Self {
                Self {
                    err
                }
            }
        }

        impl Error for $name {}
        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(format!("{}: {}", stringify!($name), self.err).as_str())
            }
        }
    };
}

create_error!(ASTError);

macro_rules! error {
    ($error_type:expr) => {
        return Err($error_type.into())
    };
}

pub fn parse_tokens(tokens: &mut Vec<Token>) -> Result<Program, Box<dyn Error>> {
    let mut root: Program = Program::new();
    let mut expression_builder: Vec<Expression> = Vec::new();

    macro_rules! clear_builder {
        () => {
            expression_builder.clear();
        };
    }

    let mut index: usize = 0;

    while index < tokens.len() {
        macro_rules! next {
            () => {
                {
                    index += 1;
                    continue;
                }
            };
        }

        let token = match tokens.get(index) {
            Some(token) => token,
            None => error!(ASTError::from_str("Couldn't get current index token"))
        };

        let expression = match token.to_owned() {
            Token::EndOfLine => {
                clear_builder!();
                next!()
            },
            Token::Plus => {
                let operator = Operator::Plus;
                let prefix: Option<ast::Prefix> = None;

                let expr = match prev_el(&expression_builder, index) {
                    Some(prev) => {
                        if let Some(next) = peek_el(&tokens, index) {
                            // Infix because both previous and next expression exists
                            match parse_token_expr(next.to_owned()) {
                                None => next!(),
                                Some(next) => {
                                    index += 1; // Skip the index
                                    Expression::Infix(operator, Box::new(prev.to_owned()), Box::new(next.to_owned()))
                                }
                            }
                        } else {
                            if let Some(prefix) = prefix {
                                // Prefix because previous doesn't exist, next does exist
                                Expression::Prefix(prefix, Box::new(prev.to_owned()))
                            } else {
                                error!(ASTError::from(format!("Operator '{:?}' doesn't support being prefixed!", operator)))
                            }
                        }
                    },
                    None => {
                        let ret = match &root.last() {
                            Some(el) => {
                                match el {
                                    Statement::Expr(Expression::Infix(op, left, right)) => {
                                        match peek_el(&tokens, index) {
                                            Some(next) => {
                                                match parse_token_expr(next.to_owned()) {
                                                    Some(expr) => {
                                                        index += 1;
                                                        Expression::Infix(op.to_owned(), Box::new(Expression::Infix(op.to_owned(), left.to_owned(), right.to_owned())), Box::new(expr))
                                                    }
                                                    None => error!("Could parse")
                                                }
                                            },
                                            None => error!("No next expr")
                                        }
                                    },
                                    _ => error!(ASTError::from_str("handle this"))
                                }
                            }
                            None => error!(ASTError::from_str("idk")),
                        };
                        root.pop();
                        ret
                    }
                };

                root.push(Statement::Expr(expr));
                clear_builder!();
                next!()
            }
            _ => {
                if let Some(expr) = parse_token_expr(token.to_owned()) {
                    expr
                } else {
                    next!();
                }
            }
        };
        
        expression_builder.push(expression);
        index += 1;
    }
    
    Ok(root)
}

fn parse_token_expr(token: Token) -> Option<Expression> {
    match token {
        Token::Integer(num) => Some(Expression::Literal(Literal::Integer(num))),
        _ => None
    }
}

fn prev_el<T>(list: &Vec<T>, index: usize) -> Option<&T> {
    if index - 1 > list.len() {
        return None;
    }

    list.get(index - 1)
}

fn peek_el<T>(list: &Vec<T>, index: usize) -> Option<&T> {
    if index + 1 > list.len() {
        return None;
    }

    list.get(index + 1)
}
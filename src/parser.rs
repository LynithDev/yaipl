use std::error::Error;

use crate::{token::{tokenize, Token}, TokenError};

pub fn parse(input: &str) -> Result<Token, Box<dyn Error>> {
    let mut tokens = tokenize(input)?;

    match eval(&mut tokens) {
        Ok(token) => Ok(token),
        Err(err) => Err(Box::new(err))
    }
}

fn eval(tokens: &mut Vec<Token>) -> Result<Token, TokenError> {
    let mut token: Token = match tokens.get(0) {
        Some(token) => token,
        None => return Ok(Token::EOL) // No tokens
    }.to_owned();

    let mut index: usize = 0; 

    while tokens.len() > 1 {
        token = match token {
            Token::Integer(_) | Token::Float(_) | Token::Boolean(_) | Token::EOL => token,
            Token::Operator(_) => eval_operator(tokens, token.to_owned(), &mut index)?,
            _ => return Err(TokenError::from(Some(token.to_owned()), "Could not determine how to evaluate token"))
        };

        index += 1;
        if let Some(next) = tokens.get(index) {
            token = next.to_owned();
        };
    };

    Ok(match tokens.first() {
        Some(token) => token.to_owned(),
        None => Token::EOL
    })
}

fn eval_operator(tokens: &mut Vec<Token>, token: Token, index: &mut usize) -> Result<Token, TokenError> {
    let operator = match &token {
        Token::Operator(operator) => operator,
        _ => return Err(TokenError::from(Some(token), "Token is not valid operator!"))
    };

    let left_token = match tokens.get(*index - 1) {
        Some(token) => token,
        None => return Err(TokenError::from(Some(token), "Missing left expression for operator"))
    };

    let right_token = match tokens.get(*index + 1) {
        Some(token) => token,
        None => return Err(TokenError::from(Some(token), "Missing right expression for operator"))
    };

    macro_rules! eval_op {
        ($operator:tt) => {
            Ok(match (left_token.to_owned(), right_token.to_owned()) {
                (Token::Integer(l), Token::Integer(r)) => Token::Integer(l $operator r),
                (Token::Integer(l), Token::Float(r)) => Token::Float(l as f32 $operator r),
                (Token::Float(l), Token::Float(r)) => Token::Float(l $operator r),
                (Token::Float(l), Token::Integer(r)) => Token::Float(l $operator r as f32),
                (Token::Float(_), _) | (Token::Integer(_), _) => return Err(TokenError::from(Some(token.to_owned()), format!("Invalid right hand type '{}' for '{}' operation", right_token.get_name(), stringify!($operator)).as_str())),
                (_, Token::Float(_)) | (_, Token::Integer(_)) => return Err(TokenError::from(Some(token.to_owned()), format!("Invalid left hand type '{}' for '{}' operation", left_token.get_name(), stringify!($operator)).as_str())),
                _ => return Err(TokenError::from(Some(token.to_owned()), format!("Invalid types for '{}' operation", stringify!($operator)).as_str())),
            })
        };
    }

    let token = match operator.as_str() {
        "+" => eval_op!(+),
        "-" => eval_op!(-),
        "/" => eval_op!(/),
        "*" => eval_op!(*),
        _ => Err(TokenError::from(Some(token.to_owned()), format!("Unsupported operator '{}'", operator).as_str()))
    }?;

    tokens.drain(0..3);
    tokens.insert(0, token.to_owned());
    *index -= 1;

    Ok(token)
}
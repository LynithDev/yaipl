use std::{error::Error, fmt::{Debug, Display}};

use crate::{evaluator::object::ObjectType, lexer::token::{Position, TokenType}};

pub type DynamicError = Box<dyn std::error::Error>;

#[macro_export]
macro_rules! error {
    ($arg:expr) => {
        return Err($arg.into())
    };
}

macro_rules! fmt_pos {
    ($pos:expr) => {
        format!("'&_&c{{{{path}}}}:{}:{}&-&r'", $pos.line, $pos.col)
    };
}

macro_rules! fmt_token {
    ($token:expr) => {
        format!("&g&*{:?}&-&r", $token)
    };
}

// --- Evaluator Errors ---
#[derive(Debug, Clone)]
pub enum EvaluatorError {
    ObjectNotFound { name: String },
    InvalidExpression {
        expected: String
    },
    InvalidType {
        expected: Vec<ObjectType>,
        found: ObjectType,
    },
}

impl Error for EvaluatorError {}
impl Display for EvaluatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluatorError::ObjectNotFound { name } => 
                write!(f, "Object '&g&*{}&-&r' not found in current scope", name),
            EvaluatorError::InvalidType { expected, found } => 
                write!(f, "Invalid type, expected {}, found {:?}", fmt_token!(expected), fmt_token!(found)),
            EvaluatorError::InvalidExpression { expected } =>
                write!(f, "Invalid expression, expected '{:?}'", expected),
        }
    }
}


// --- Parser Errors ---
#[derive(Debug, Clone)]
pub enum ParserError {
    UnexpectedToken {
        found: TokenType,
        pos: Position
    },
    
    InvalidToken {
        expected: Vec<TokenType>,
        found: TokenType,
        pos: Position
    },

    InvalidStatement,
    OutOfBounds { index: String },
}

impl Error for ParserError {}
impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::UnexpectedToken { found, pos } => 
                write!(f, "Unexpected token {} at {}", fmt_token!(found), fmt_pos!(pos)),

            ParserError::InvalidToken { expected, found, pos } => 
                write!(f, "Token {} was found at {}, expected {}", fmt_token!(found), fmt_pos!(pos), fmt_token!(expected)),
                
            ParserError::OutOfBounds { index } => 
                write!(f, "Out of bounds for index &c{}", index),

            ParserError::InvalidStatement =>
                write!(f, "Invalid statement"),
        }
    }
}


// --- Lexer Errors ---
#[derive(Debug, Clone)]
pub enum LexerError {
    OutOfBounds { index: String },
    InvalidCharacter {
        character: char,
        pos: Position
    },
}

impl Error for LexerError {}
impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::OutOfBounds { index } => 
                write!(f, "Out of bounds at index {}", index),
            LexerError::InvalidCharacter { character, pos } => 
                write!(f, "Invalid character '{}' at {:?}", character, fmt_pos!(pos)),
        }
    }
}

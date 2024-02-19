use std::{error::Error, fmt::Display};

use token::Token;

pub mod token;
pub mod parser;

#[derive(Debug)]
pub struct TokenError {
    pub err: String,
    pub token: Option<Token>
}

impl TokenError {
    pub fn from(token: Option<Token>, err: &str) -> Self {
        Self {
            token,
            err: err.to_string()
        }
    }
}

impl Error for TokenError {}
impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token_value = match &self.token {
            Some(token) => format!("{}", token.to_string()),
            None => format!("none-provided")
        };

        f.write_str(format!("A token error has occurred for token '{}'\n{}", token_value, self.err).as_str())
    }
}

#[derive(Debug)]
pub struct SyntaxError {
    pub err: String,
    pub line: u16, // Hopefully nobody has a file which is over 65535 lines long .-.
    pub col: u16,
}

impl SyntaxError {
    pub fn from(col: u16, line: u16, err: &str) -> Self {
        Self { 
            err: err.to_string(), 
            line, 
            col 
        }
    }
}

impl Error for SyntaxError {}
impl Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("A syntax error has occurred at {}:{}\nMessage: {}", self.line, self.col, self.err).as_str())
    }
}

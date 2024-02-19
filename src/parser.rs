use std::{error::Error, fmt::Display};

use crate::token::{tokenize, Token};

#[derive(Debug)]
pub struct SyntaxError {
    pub err: String,
    pub line: u16, // Hopefully nobody has a file which is over 65535 lines long .-.
    pub col: u16,
}

impl SyntaxError {
    fn from(col: u16, line: u16, err: &str) -> Self {
        Self { 
            err: err.to_string(), 
            line, 
            col 
        }
    }
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("A syntax error has occurred at {}:{}\nMessage: {}", self.line, self.col, self.err).as_str())
    }
}

impl Error for SyntaxError {}

pub fn parse(input: &str) -> Result<(), Box<dyn Error>> {
    let mut tokens = tokenize(input)?;

    parse_list(&mut tokens)?;

    Ok(())
}

fn parse_list(tokens: &mut Vec<Token>) -> Result<(), SyntaxError> {
    Err(SyntaxError::from(5, 5, "aaa"))
}
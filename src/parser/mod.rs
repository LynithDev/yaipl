use std::error::Error;

use crate::{create_error, error, lexer::token::{Position, TokenType, Tokens}};

use self::ast::Program;

pub mod ast;

create_error!(ParserError, {
    token_type: TokenType,
    pos: Position
});

pub fn parse_tokens(tokens: &Tokens) -> Result<Program, Box<dyn Error>> {
    

    error!("");
}
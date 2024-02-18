use std::error::Error;

use crate::token::{tokenize, Token};

pub fn parse(input: &str) -> Result<(), Box<dyn Error>> {
    let tokens = tokenize(input)?;
    
    
    
    Ok(())
}

fn parse_list(tokens: &mut Vec<Token>) -> Result<(), Box<dyn Error>> {
    
    
    Ok(())
}
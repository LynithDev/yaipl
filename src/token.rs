use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Operator(String),
    Symbol(String),
    Integer(i32),
    Float(f32),
    EOL,
    LParen,
    RParen,
    Scope(Vec<Token>)
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Token::*;
        f.write_str(
            (match self {
                Integer(v) => format!("{}", v),
                Float(v) => format!("{}", v),

                LParen => format!("("),
                RParen => format!(")"),
                EOL => format!("eol"),
                
                Scope(tokens) => format!("SCOPE {:#?} ESCOPE", tokens),

                Operator(v) 
                | Symbol(v) 
                | Keyword(v) => format!("{}", v),
            }).as_str()
        )
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().collect::<Vec<char>>();

    if chars.is_empty() {
        return Ok(tokens);
    }

    while !chars.is_empty() {
        let mut char = chars.remove(0);

        if check_stuff(&mut tokens, &mut chars, char)? {
            continue;
        }

        fn check_word_loop(char: char) -> bool {
            !char.is_whitespace() 
            && !is_comment(char) 
            && is_parenthesis(char).is_none() 
            && is_scope_gate(char).is_none()
            && is_operator_basic(char).is_none()
        }
        
        // Collecting words
        let mut word = String::new();
        while chars.len() > 0 && check_word_loop(char) {
            word.push(char);
            char = chars.remove(0);
        }
        
        if check_word_loop(char) {
            word.push(char);
        }

        // Adding our tokens !!
        if !word.is_empty() {
            tokens.push(
                if let Ok(num) = word.replace("_", "").parse::<i32>() {
                    Token::Integer(num)
                } else if let Ok(num) = word.replace("_", "").parse::<f32>() {
                    Token::Float(num)
                } else {

                    match word.as_str() {
                        input if is_keyword(input) => Token::Keyword(word),
                        _ => Token::Symbol(word)
                    }
                }
            );
        }

        check_stuff(&mut tokens, &mut chars, char)?;
    }

    push_eol(&mut tokens);

    Ok(tokens)
}


// ------------------
// TOKEN LIST HELPERS
// ------------------

fn push_eol(tokens: &mut Vec<Token>) {
    if let Some(token) = tokens.last() {
        if token.to_owned() != Token::EOL {
            tokens.push(Token::EOL)
        }
    }
}

// the value of bool here tells the program whether to continue the loop or not
fn check_stuff(mut tokens: &mut Vec<Token>, chars: &mut Vec<char>, mut char: char) -> Result<bool, Box<dyn Error>> {
    if is_comment(char) {
        // Remove all characters up until EOL
        while chars.len() > 0 && char != '\n' {
            char = chars.remove(0);
        }
        
        push_eol(&mut tokens);
        return Ok(true);
    }

    if is_eol(char).is_some() {
        push_eol(&mut tokens);
        return Ok(true);
    }

    if is_operator_arithmetic(char).is_some() {
        tokens.push(Token::Operator(char.to_string()));
        return Ok(true);
    }

    if let Some(char) = is_operator_equation(char) {
        match char {
            '=' => {
                if let Some(prev) = tokens.last() {
                    match prev.to_owned() {
                        Token::Operator(prev_value) => {
                            if prev_value.len() == 1 {
                                if let Ok(parsed) = prev_value.parse::<char>() {
                                    if is_operator_arithmetic(parsed).is_some() {
                                        tokens.pop();
                                        tokens.push(Token::Operator(format!("{}=", parsed)));
                                        return Ok(true);
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
        tokens.push(Token::Operator(char.to_string()));
        return Ok(true);
    }

    if let Some(paren) = is_parenthesis(char) {
        match paren {
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            _ => {}
        };

        return Ok(true);
    }

    if char == '{' {
        char = chars.remove(0);
        let mut scope_source = String::new();

        while !chars.is_empty() && char != '}' {
            scope_source.push(char);
            char = chars.remove(0);
        }
        
        let list: Vec<Token> = tokenize(scope_source.as_str())?;
        tokens.push(Token::Scope(list));
        return Ok(true);
    }

    return Ok(false);
}


// -----
// UTILS
// -----
pub fn is_keyword(input: &str) -> bool {
    match input {
        "if" | "for" | "while" | "return" => true,
        _ => false,
    }
}

pub fn is_operator(chars: &mut Vec<char>) -> Option<&str> {
    let mut cloned = chars.clone();
    let char = cloned.remove(0);
    let mut word = String::new();

    while !cloned.is_empty() && !char.is_whitespace() {
        word.push(char);
        cloned.remove(0);
    }

    None
}

pub fn is_operator_basic(input: char) -> Option<char> {
    match input {
        char if is_operator_arithmetic(char).is_some() || is_operator_equation(char).is_some() => Some(input),
        _ => None
    }
}

pub fn is_operator_equation(input: char) -> Option<char> {
    match input {
        '=' | '<' | '>' => Some(input),
        _ => None
    }
}

pub fn is_operator_arithmetic(input: char) -> Option<char> {
    match input {
        '+' | '-' | '*' | '/' | '%' => Some(input),
        _ => None
    }
}

pub fn is_parenthesis(input: char) -> Option<char> {
    if input == '(' || input == ')' {
        return Some(input)
    }

    None
}

pub fn is_comment(input: char) -> bool {
    input == '#'
}

pub fn is_eol(input: char) -> Option<char> {
    if input == '\n' || input == ';' {
        return Some(input)
    }

    None
} 

pub fn is_scope_gate(input: char) -> Option<char> {
    if input == '{' || input == '}' {
        return Some(input)
    }

    None
}
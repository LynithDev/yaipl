use std::error::Error;
use token::Token;

pub mod token;

pub fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().collect::<Vec<char>>();
    
    while !chars.is_empty() {
        let mut char = chars.remove(0);

        let matched_tokens: Vec<Token> = match match_char(&mut chars, char) {
            Some(token) => vec![token],

            None => {
                let mut ret: Vec<Token> = Vec::new();
                let mut word = String::new();
        
                while !chars.is_empty() && !char.is_whitespace() && match_char(&mut chars, char).is_none() {
                    word.push(char);
                    char = chars.remove(0);
                }
        
                match match_char(&mut chars, char) {
                    Some(token) => {
                        ret.push(token);
                    }
                    None => {
                        if char != ' ' {
                            word.push(char)
                        }
                    }
                }
                
                if word.is_empty() {
                    continue;
                }
        
                ret.push(if let Ok(num) = word.replace("_", "").parse::<i32>() {
                    Token::Integer(num)
                } else if let Ok(num) = word.replace("_", "").parse::<f32>() {
                    Token::Float(num)
                } else {
                    match word.as_str() {
                        "true" => Token::Boolean(1),
                        "false" => Token::Boolean(0),

                        // Keywords
                        "if" => Token::If,
                        "while" => Token::While,
                        "for" => Token::For,
                        "return" => Token::Return,

                        _ => Token::Symbol(word)
                    }
                });

                ret
            }
        };
        
        for token in matched_tokens.iter().rev() {
            // Remove duplicate EOL
            if let Some(last) = tokens.last() {
                match last {
                    Token::EndOfLine => {
                        if token == last {
                            continue;
                        }
                    },
                    _ => {}
                };
            }

            tokens.push(token.to_owned());
        }
    }

    if tokens.starts_with(&[Token::EndOfLine]) {
        tokens.remove(0);
    }

    if !tokens.ends_with(&[Token::EndOfLine]) {
        tokens.push(Token::EndOfLine);
    }
    
    Ok(tokens)
}

fn peek_eq(chars: &mut Vec<char>, char: char) -> bool {
    if chars.len() < 2 {
        return false;
    }

    if let Some(next_char) = chars.get(0) {
        if next_char.to_owned() == char {
            chars.remove(0);
            return true;
        }
    }

    false
}

fn peek_eq_ret<T>(chars: &mut Vec<char>, peek_char: char, if_true: T, if_false: T) -> T {
    if peek_eq(chars, peek_char) {
        if_true
    } else {
        if_false
    }
}

fn match_char(chars: &mut Vec<char>, char: char) -> Option<Token> {
    Some(match char {
        '(' => Token::LeftParen,
        ')' => Token::RightParen,
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,

        '+' => peek_eq_ret(chars, '=', Token::PlusAssign, Token::Plus),
        '-' => peek_eq_ret(chars, '=', Token::MinusAssign, Token::Minus),
        '*' => peek_eq_ret(chars, '=', Token::MultiplyAssign, Token::Multiply),
        '/' => peek_eq_ret(chars, '=', Token::DivideAssign, Token::Divide),
        '%' => peek_eq_ret(chars, '=', Token::ModuloAssign, Token::Modulo),
        '^' => peek_eq_ret(chars, '=', Token::PowerAssign, Token::Power),
        '=' => peek_eq_ret(chars, '=', Token::Equal, Token::Assign),

        '<' => peek_eq_ret(chars, '=', Token::LesserThanEqual, Token::LesserThan),
        '>' => peek_eq_ret(chars, '=', Token::GreaterThanEqual, Token::GreaterThan),
        
        '!' => peek_eq_ret(chars, '=', Token::NotEqual, Token::Not),

        '\n' | ';' => Token::EndOfLine,
        _ => return None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let input = "
            test = () {
                var = 5
                var += 2

                if var == 7 {
                    print(1)
                    return var
                }

                print(0)
                return var
            }

            test() + 5
        ";

        let result = tokenize(input).unwrap();
        
        use Token::*;
        let expected: Vec<Token> = vec![
            Symbol("test".to_string()),
            Assign,
            LeftParen,
            RightParen,
            LeftBrace,
            EndOfLine,

            Symbol("var".to_string()),
            Assign,
            Integer(5),
            EndOfLine,

            Symbol("var".to_string()),
            PlusAssign,
            Integer(2),
            EndOfLine,

            If,
            Symbol("var".to_string()),
            Equal,
            Integer(7),
            LeftBrace,
            EndOfLine,

            Symbol("print".to_string()),
            LeftParen,
            Integer(1),
            RightParen,
            EndOfLine,

            Return,
            Symbol("var".to_string()),
            EndOfLine,

            RightBrace,
            EndOfLine,

            Symbol("print".to_string()),
            LeftParen,
            Integer(0),
            RightParen,
            EndOfLine,

            Return,
            Symbol("var".to_string()),
            EndOfLine,

            RightBrace,
            EndOfLine,

            Symbol("test".to_string()),
            LeftParen,
            RightParen,
            Plus,
            Integer(5),
            EndOfLine
        ];
        
        assert_eq!(result, expected)
    }
}
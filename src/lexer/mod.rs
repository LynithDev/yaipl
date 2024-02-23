use crate::{create_error, create_error_list, error};
use self::token::{Position, Token, TokenType, Tokens};

pub mod token;

/* ERRORS */
create_error!(LexerError, {
    token_type: Option<TokenType>,
    pos: Position
});

create_error!(LexerOutOfBounds, {});

create_error_list!(LexerErrors, {
    LexerError,
    LexerOutOfBounds
});
/* END ERRORS */

// TODO: Implement position
#[allow(dead_code)]
pub struct Lexer {
    pub tokens: Tokens,
    chars: Vec<char>,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn from(input: &str) -> Lexer {
        Lexer {
            tokens: Vec::new(),
            chars: input.chars().collect::<Vec<char>>(),
            line: 0,
            col: 0
        }
    }

    pub fn tokens_to_string(tokens: &Tokens) -> String {
        let mut builder: String = String::new();

        for token in tokens {
            builder += format!("    {:?},\n", token.token_type.to_owned()).as_str()
        };

        format!("[\n{}]", builder)
    }

    fn remove_char(&mut self, index: usize) -> Result<char, LexerErrors> {
        if index > self.chars.len() {
            error!("Index out of bounds")
        }

        let char = self.chars.remove(0);
        // TODO: set line and col variables
        Ok(char)
    }

    fn parse_word(&mut self, char: &mut char) -> Result<String, LexerErrors> {
        let mut word = String::new();
        
        while !self.chars.is_empty() && !char.is_whitespace() && match_char(&mut self.chars, char.to_owned()).is_none() {
            word.push(char.to_owned());
            *char = self.remove_char(0)?;
        };

        Ok(word)
    }

    pub fn tokenize(&mut self) -> Result<&Tokens, LexerErrors> {
        while !self.chars.is_empty() {
            let mut char = self.remove_char(0)?;
    
            let matched_tokens: Tokens = match match_char(&mut self.chars, char) {
                Some(token) => vec![Token::from(token, (0, 0), (0, 0))],
    
                None => {
                    let mut ret: Tokens = Vec::new();
                    let mut word = self.parse_word(&mut char)?;

                    if let Some(token) = match_char(&mut self.chars, char) {
                        ret.push(Token::from(token, (0, 0), (0, 0)));
                    } else if char != ' ' {
                        word.push(char)
                    }
                    
                    if word.is_empty() {
                        continue;
                    }
            
                    ret.push(if let Ok(num) = word.replace("_", "").parse::<i32>() {
                        Token::from(TokenType::Integer(num), (0, 0), (0, 0))
                    } else if let Ok(num) = word.replace("_", "").parse::<f32>() {
                        Token::from(TokenType::Float(num), (0, 0), (0, 0))
                    } else {
                        match word.as_str() {
                            "true" => Token::from(TokenType::Boolean(1), (0, 0), (0, 0)),
                            "false" => Token::from(TokenType::Boolean(0), (0, 0), (0, 0)),
    
                            // Keywords
                            "if" => Token::from(TokenType::If, (0, 0), (0, 0)),
                            "while" => Token::from(TokenType::While, (0, 0), (0, 0)),
                            "for" => Token::from(TokenType::For, (0, 0), (0, 0)),
                            "return" => Token::from(TokenType::Return, (0, 0), (0, 0)),
    
                            _ => Token::from(TokenType::Symbol(word), (0, 0), (0, 0))
                        }
                    });
    
                    ret
                }
            };
            
            for token in matched_tokens.iter().rev() {
                // Remove duplicate EOL
                if let Some(last) = self.tokens.last() {
                    match last.token_type {
                        TokenType::EndOfLine => {
                            if token == last {
                                continue;
                            }
                        },
                        _ => {}
                    };
                }
    
                self.tokens.push(token.to_owned());
            }
        }
    
        if let Some(first) = self.tokens.first() {
            match first.token_type {
                TokenType::EndOfLine => {
                    self.tokens.remove(0);
                },
                _ => {}
            }
        }
    
        if let Some(last) = self.tokens.last() {
            match last.token_type {
                TokenType::EndOfLine => {},
                _ => {
                    self.tokens.push(Token::from(TokenType::EndOfLine, (0, 0), (0, 0)))
                },
            }
        }

        self.tokens.push(Token::from(TokenType::EndOfFile, (0, 0), (0, 0)));
        
        Ok(&self.tokens)
    }

}

fn accept_eq(chars: &mut Vec<char>, char: char) -> bool {
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

fn accept_eq_ret<T>(chars: &mut Vec<char>, peek_char: char, if_true: T, if_false: T) -> T {
    if accept_eq(chars, peek_char) {
        if_true
    } else {
        if_false
    }
}

fn match_char(chars: &mut Vec<char>, char: char) -> Option<TokenType> {
    Some(match char {
        '(' => TokenType::LeftParen,
        ')' => TokenType::RightParen,
        '{' => TokenType::LeftBrace,
        '}' => TokenType::RightBrace,

        '+' => accept_eq_ret(chars, '=', TokenType::PlusAssign, TokenType::Plus),
        '-' => accept_eq_ret(chars, '=', TokenType::MinusAssign, TokenType::Minus),
        '*' => accept_eq_ret(chars, '=', TokenType::MultiplyAssign, TokenType::Multiply),
        '/' => accept_eq_ret(chars, '=', TokenType::DivideAssign, TokenType::Divide),
        '%' => accept_eq_ret(chars, '=', TokenType::ModuloAssign, TokenType::Modulo),
        '^' => accept_eq_ret(chars, '=', TokenType::PowerAssign, TokenType::Power),
        '=' => accept_eq_ret(chars, '=', TokenType::Equal, TokenType::Assign),

        '<' => accept_eq_ret(chars, '=', TokenType::LesserThanEqual, TokenType::LesserThan),
        '>' => accept_eq_ret(chars, '=', TokenType::GreaterThanEqual, TokenType::GreaterThan),
        
        '!' => accept_eq_ret(chars, '=', TokenType::NotEqual, TokenType::Not),

        '&' => {
            if accept_eq(chars, '&') {
                TokenType::And
            } else {
                return None
            }
        }

        '|' => {
            if accept_eq(chars, '|') {
                TokenType::Or
            } else {
                return None
            }
        }

        '\n' | ';' => TokenType::EndOfLine,
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

        let mut lexer = Lexer::from(input);
        let result = lexer.tokenize().unwrap();
        let mut token_type_result: Vec<TokenType> = Vec::new();

        for token in result {
            token_type_result.push(token.token_type.to_owned());
        }
        
        use TokenType::*;
        let expected: Vec<TokenType> = vec![
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
        
        assert_eq!(token_type_result, expected)
    }
}
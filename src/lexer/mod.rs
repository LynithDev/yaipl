use crate::{error, errors::{DynamicError, LexerError}};
use self::token::{Position, Token, TokenLiteral, TokenType, Tokens};

pub mod token;

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
            line: 1,
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

    pub fn tokenize(&mut self) -> Result<&Tokens, DynamicError> {
        while !self.chars.is_empty() {
            let mut char = self.remove_char(0)?;

            if char == '\n' {
                continue;
            }
    
            let matched_tokens: Tokens = match self.match_char(char) {
                Some((token, len)) => vec![
                    Token::from_pos(
                        token, 
                        self.get_pos(), 
                        self.get_pos_offset(len as usize)
                    )
                ],
    
                None => {
                    let mut ret: Tokens = Vec::new();
                    let start: Position = self.get_pos();

                    if char == '"' {
                        let str = self.parse_string(&mut char)?;

                        ret.push(Token::from_value_pos(
                            TokenType::String, 
                            start, 
                            self.get_pos_offset(str.chars().count()),
                            Some(TokenLiteral::String(str))
                        ));
                    } else {
                        let mut word = self.parse_word(&mut char)?;
                        
                        if self.is_comment(&char) {
                            while !self.chars.is_empty() && char != '\n' {
                                char = self.remove_char(0)?;
                            }
                            
                            self.tokens.push(Token::from_pos(
                                TokenType::EndOfLine, 
                                self.get_pos(), 
                                self.get_pos_offset(1)
                            ));
                            continue;
                        }
                        
                        if let Some((token, len)) = self.match_char(char) {
                            ret.push(Token::from_pos(
                                token, 
                                self.get_pos(), 
                                self.get_pos_offset(len as usize))
                            );
                        } else if char != ' ' && char != '\n' {
                            word.push(char)
                        }
                        
                        if word.is_empty() {
                            continue;
                        }

                        let end: Position = self.get_pos();
            
                        ret.push(if let Ok(num) = word.replace("_", "").parse::<i32>() {
                            Token::from_value_pos(
                                TokenType::Integer, 
                                start,
                                self.get_pos_offset(num.to_string().chars().count()),
                                Some(TokenLiteral::Integer(num))
                            )
                        } else if let Ok(num) = word.replace("_", "").parse::<f32>() {
                            Token::from_value_pos(
                                TokenType::Float, 
                                start, 
                                self.get_pos_offset(num.to_string().chars().count()),
                                Some(TokenLiteral::Float(num))
                            )
                        } else {
                            let (token_type, value) = match word.as_str() {
                                "true" => (TokenType::Boolean, Some(TokenLiteral::Boolean(true))),
                                "false" => (TokenType::Boolean, Some(TokenLiteral::Boolean(false))),
                                "null" => (TokenType::Null, None),
        
                                // Keywords
                                "if" => (TokenType::If, None),
                                "elif" => (TokenType::ElIf, None),
                                "else" => (TokenType::Else, None),
                                "while" => (TokenType::While, None),
                                "for" => (TokenType::For, None),
                                "return" => (TokenType::Return, None),
                                "break" => (TokenType::Break, None),
                                "continue" => (TokenType::Continue, None),
        
                                _ => (TokenType::Symbol, Some(TokenLiteral::String(word)))
                            };

                            Token::from_value_pos(token_type, start, end, value)
                        });
                    };
    
                    ret
                }
            };
            
            for token in matched_tokens.iter().rev() {
                // Remove duplicate end of lines
                if let Some(last) = self.tokens.last() {
                    if last.token_type == TokenType::EndOfLine && token.token_type == TokenType::EndOfLine {
                        continue;
                    }
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
                    self.tokens.push(
                        Token::from_pos(
                            TokenType::EndOfLine, 
                            self.get_pos(),
                            self.get_pos_offset(1)
                        )
                    )
                },
            }
        }

        self.tokens.push(
            Token::from_pos(TokenType::EndOfFile, self.get_pos(), self.get_pos_offset(1))
        );

        Ok(&self.tokens)
    }

    fn remove_char(&mut self, index: usize) -> Result<char, DynamicError> {
        if index > self.chars.len() {
            error!(LexerError::OutOfBounds { index: index.to_string() })
        }

        self.pos_advance(1);
        let char = self.chars.remove(0);
        
        Ok(char)
    }

    fn parse_word(&mut self, char: &mut char) -> Result<String, DynamicError> {
        let mut word = String::new();

        while !self.chars.is_empty() && !char.is_whitespace() && !self.is_comment(&char) && self.match_char(char.to_owned()).is_none() {
            word.push(char.to_owned());
            *char = self.remove_char(0)?;
        };

        Ok(word)
    }

    fn is_comment(&self, char: &char) -> bool {
        char == &'#'
    }

    fn parse_string(&mut self, char: &mut char) -> Result<String, DynamicError> {
        let mut builder = String::new();
        *char = self.remove_char(0)?;

        while !self.chars.is_empty() {
            if !builder.ends_with("\\") && char == &'"' {
                break;
            }

            if char == &'\\' {
                *char = self.remove_char(0)?;
                match char {
                    'b' => builder.push('\u{0008}'),
                    'f' => builder.push('\u{000C}'),
                    'n' => builder.push('\n'),
                    't' => builder.push('\t'),
                    'r' => builder.push('\r'),
                    '\'' => builder.push('\''),
                    '\"' => builder.push('\"'),
                    '\\' => builder.push('\\'),
                    'u' => {
                        let mut hex = String::new();
                        
                        for _ in 0..4 {
                            hex.push(self.remove_char(0)?);
                        }
                        
                        let unicode = match u32::from_str_radix(&hex, 16) {
                            Ok(unicode) => unicode,
                            Err(_) => error!(LexerError::InvalidCharacter { 
                                character: char.to_owned(), 
                                pos: self.get_pos()
                            })
                        };

                        builder.push(std::char::from_u32(unicode).unwrap());
                    },
                    _ => builder.push(char.to_owned())
                }
                *char = self.remove_char(0)?;
                continue;
            }

            builder.push(char.to_owned());
            *char = self.remove_char(0)?;
        };

        Ok(builder)
    }

    fn get_pos(&self) -> Position {
        Position::from(self.line, self.col)
    }

    fn get_pos_offset(&self, amount: usize) -> Position {
        let mut pos = self.get_pos();
        pos.col += amount;
        pos
    }

    fn pos_advance(&mut self, amount: usize) {
        for _ in 0..amount {
            if self.chars.is_empty() {
                break;
            }
            
            let char = self.chars.get(0).unwrap_or(&' ');

            if char == &'\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
        }
    }

    fn accept_eq(&mut self, char: char) -> bool {
        if self.chars.len() < 2 {
            return false;
        }
    
        if let Some(next_char) = self.chars.get(0) {
            if next_char.to_owned() == char {
                self.chars.remove(0);
                return true;
            }
        }
    
        false
    }
    
    fn match_char(&mut self, char: char) -> Option<(TokenType, u8)> {
        macro_rules! accept_eq_ret {
            ($sym_b:literal, $tru:expr, $fal:expr) => {
                if self.accept_eq($sym_b) {
                    return Some(($tru, 2));
                } else {
                    return Some(($fal, 1));
                }
            };
        }

        Some(match char {
            ',' => (TokenType::Comma, 1),
            '(' => (TokenType::LeftParen, 1),
            '{' => (TokenType::LeftBrace, 1),
            '[' => (TokenType::LeftBracket, 1),
            ')' => (TokenType::RightParen, 1),
            '}' => (TokenType::RightBrace, 1),
            ']' => (TokenType::RightBracket, 1),
    
            '+' => accept_eq_ret!('=', TokenType::PlusAssign, TokenType::Plus),
            '-' => accept_eq_ret!('=', TokenType::MinusAssign, TokenType::Minus),
            '*' | '×' => accept_eq_ret!('=', TokenType::MultiplyAssign, TokenType::Multiply),
            '/' | '÷' => accept_eq_ret!('=', TokenType::DivideAssign, TokenType::Divide),
            '%' => accept_eq_ret!('=', TokenType::ModuloAssign, TokenType::Modulo),
            '^' => accept_eq_ret!('=', TokenType::PowerAssign, TokenType::Power),
            '=' => accept_eq_ret!('=', TokenType::Equal, TokenType::Assign),
    
            '<' => accept_eq_ret!('=', TokenType::LesserThanEqual, TokenType::LesserThan),
            '>' => accept_eq_ret!('=', TokenType::GreaterThanEqual, TokenType::GreaterThan),
            
            '!' => accept_eq_ret!('=', TokenType::NotEqual, TokenType::Not),
    
            '&' => {
                if self.accept_eq('&') {
                    (TokenType::And, 2)
                } else {
                    return None
                }
            }
    
            '|' => {
                if self.accept_eq('|') {
                    (TokenType::Or, 2)
                } else {
                    return None
                }
            }
    
            ';' => (TokenType::EndOfLine, 1),
            _ => return None
        })
    }
}
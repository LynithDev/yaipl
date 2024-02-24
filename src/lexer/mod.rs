use crate::{create_error, create_error_list, error, errors::ErrorWithPosition};
use self::token::{Position, Token, TokenLiteral, TokenType, Tokens};

pub mod token;

/* ERRORS */
create_error!(LexerError, {
    token_type: Option<TokenType>,
    pos: Position
});

impl ErrorWithPosition for LexerError {
    fn position(&self) -> Position {
        self.pos.to_owned()
    }
}

create_error!(LexerOutOfBounds, {});

create_error_list!(LexerErrors, {
    LexerError,
    LexerOutOfBounds
});
/* END ERRORS */

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
            col: 1
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
        self.pos_advance(1);
        
        Ok(char)
    }

    fn parse_word(&mut self, char: &mut char) -> Result<String, LexerErrors> {
        let mut word = String::new();
        
        while !self.chars.is_empty() && !char.is_whitespace() && self.match_char(char.to_owned()).is_none() {
            word.push(char.to_owned());
            *char = self.remove_char(0)?;
        };

        Ok(word)
    }

    fn get_pos(&self) -> Position {
        Position::from(self.line, self.col)
    }

    fn get_pos_offset(&self, amount: usize) -> Position {
        let mut line = self.line;
        let mut col = self.col;

        for _ in 0..amount {
            let char = match self.chars.get(amount) {
                Some(char) => char,
                None => { return Position::from(line, col); }
            };

            if char == &'\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        Position::from(line, col)
    }

    fn pos_advance(&mut self, amount: usize) {
        for _ in 0..amount {
            let char = match self.chars.get(amount) {
                Some(char) => char,
                None => { return; }
            };

            if char == &'\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<&Tokens, LexerErrors> {
        while !self.chars.is_empty() {
            let mut char = self.remove_char(0)?;
    
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
                    let mut word = self.parse_word(&mut char)?;

                    if let Some((token, len)) = self.match_char(char) {
                        ret.push(Token::from_pos(
                            token, 
                            self.get_pos(), 
                            self.get_pos_offset(len as usize))
                        );
                    } else if char != ' ' {
                        word.push(char)
                    }
                    
                    if word.is_empty() {
                        continue;
                    }
            
                    ret.push(if let Ok(num) = word.replace("_", "").parse::<i32>() {
                        Token::from_value_pos(
                            TokenType::Integer, 
                            self.get_pos(), 
                            self.get_pos_offset(num.to_string().chars().count()),
                            Some(TokenLiteral::Integer(num))
                        )
                    } else if let Ok(num) = word.replace("_", "").parse::<f32>() {
                        Token::from_value_pos(
                            TokenType::Float, 
                            self.get_pos(), 
                            self.get_pos_offset(num.to_string().chars().count()),
                            Some(TokenLiteral::Float(num))
                        )
                    } else {
                        let (token_type, len, value) = match word.as_str() {
                            "true" => (TokenType::Boolean, 4, Some(TokenLiteral::Boolean(1))),
                            "false" => (TokenType::Boolean, 5, Some(TokenLiteral::Boolean(0))),
    
                            // Keywords
                            "if" => (TokenType::If, 2, None),
                            "while" => (TokenType::While, 5, None),
                            "for" => (TokenType::For, 3, None),
                            "return" => (TokenType::Return, 6, None),
    
                            _ => (TokenType::Symbol, word.chars().count(), Some(TokenLiteral::String(word)))
                        };

                        Token::from_value_pos(token_type, self.get_pos(), self.get_pos_offset(len), value)
                    });
    
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
            ')' => (TokenType::RightParen, 1),
            '{' => (TokenType::LeftBrace, 1),
            '}' => (TokenType::RightBrace, 1),
    
            '+' => accept_eq_ret!('=', TokenType::PlusAssign, TokenType::Plus),
            '-' => accept_eq_ret!('=', TokenType::MinusAssign, TokenType::Minus),
            '*' => accept_eq_ret!('=', TokenType::MultiplyAssign, TokenType::Multiply),
            '/' => accept_eq_ret!('=', TokenType::DivideAssign, TokenType::Divide),
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
    
            '\n' | ';' => (TokenType::EndOfLine, 1),
            _ => return None
        })
    }

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
        
        // TODO: Implement token comparison

        println!("{:#?}", result);
    }
}
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl Position {
    pub fn from(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    pub fn to_tuple(&self) -> (usize, usize) {
        (self.line, self.col)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenLiteral {
    Float(f32),
    Integer(i32),
    Boolean(i8),
    String(String),
}

impl TokenLiteral {
    pub fn get_value(&self) -> String {
        match self {
            TokenLiteral::Float(value) => value.to_string(),
            TokenLiteral::Integer(value) => value.to_string(),
            TokenLiteral::Boolean(value) => value.to_string(),
            TokenLiteral::String(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub start: Position,
    pub end: Position,
    pub value: Option<TokenLiteral>
}

impl Token {
    pub fn from_value_pos(token: TokenType, start: Position, end: Position, value: Option<TokenLiteral>) -> Self {
        Self {
            token_type: token,
            start,
            end,
            value
        }
    }

    pub fn from_pos(token: TokenType, start: Position, end: Position) -> Self {
        Token::from_value_pos(token, start, end, None)
    }

    pub fn from_value(token: TokenType, value: Option<TokenLiteral>) -> Self {
        Self::from_value_pos(token, Position::from(0, 0), Position::from(0, 0), value)
    }

    pub fn from(token: TokenType) -> Self {
        Self::from_value_pos(token, Position::from(0, 0), Position::from(0, 0), None)
    }

    pub fn pos_range(&self) -> (Position, Position) {
        (self.start.to_owned(), self.end.to_owned())
    }
}

pub type Tokens = Vec<Token>;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Types
    Integer,
    Float,
    Boolean,
    String,
    Null,
    
    // Operators
    Plus,
    Minus,
    Divide,
    Multiply,
    Modulo,
    Power,
    
    PlusAssign,
    MinusAssign,
    DivideAssign,
    MultiplyAssign,
    ModuloAssign,
    PowerAssign,
    Assign,

    Or,
    And,
    Not,
    Equal,
    NotEqual,
    LesserThan,
    GreaterThan,
    LesserThanEqual,
    GreaterThanEqual,
    // EOF Operators

    // keywords
    If,
    ElIf,
    Else,
    While,
    For,
    Return,
    Break,
    Continue,

    // Other
    LeftParen,
    LeftBrace,
    RightParen,
    RightBrace,
    EndOfLine,
    EndOfFile,
    Symbol,
    Comma,

    Unknown
}
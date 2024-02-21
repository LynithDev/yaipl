#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub fn from(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub start: Position,
    pub end: Position,
}

impl Token {
    pub fn from_pos(token: TokenType, start: Position, end: Position) -> Self {
        Self {
            token_type: token,
            start,
            end
        }
    }

    pub fn from(token: TokenType, start: (usize, usize), end: (usize, usize)) -> Self {
        Self {
            token_type: token,
            start: Position::from(start.0, start.1),
            end: Position::from(end.0, end.1)
        }
    }
}

pub type Tokens = Vec<Token>;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Types
    Integer(i32),
    Float(f32),
    Boolean(i8),
    String(String),
    List(Vec<TokenType>),
    
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
    While,
    For,
    Return,

    // Other
    LeftParen,
    LeftBrace,
    RightParen,
    RightBrace,
    EndOfLine,
    Symbol(String),

    Unknown
}

impl TokenType {
    pub fn len(&self) -> usize {
        use TokenType::*;
        match self {
            Integer(n) => n.to_string().len(),
            Float(n) => n.to_string().len(),
            Boolean(b) => b.to_string().len(),
            String(s) => s.chars().count(),
            List(list) => {
                let mut total: usize = 0;
                for token in list {
                    total += token.len();
                }
                total
            },

            Plus | Minus | Divide | Multiply | Modulo | Power
            | Not | Assign | LesserThan | GreaterThan => 1,

            PlusAssign | MinusAssign | DivideAssign 
            | MultiplyAssign | ModuloAssign 
            | PowerAssign | Equal | NotEqual
            | GreaterThanEqual | LesserThanEqual => 2,

            If => 2,
            While => 5,
            For => 3,
            Return => 6,

            LeftParen | LeftBrace | RightParen | RightBrace | EndOfLine => 1,
            
            Symbol(s) => s.chars().count(),
            Unknown => 0,
        }
    }
}
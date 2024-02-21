#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Types
    Integer(i32),
    Float(f32),
    Boolean(i8),
    String(String),
    List(Vec<Token>),
    
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

impl Token {
    pub fn len(&self) -> usize {
        use Token::*;
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
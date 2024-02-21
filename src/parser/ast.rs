#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    ExpressionStatement(Expression),
}

pub type Program = Vec<Statement>;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    IdentifierExpr(Identifier),
    LiteralExpr(Literal),
    BinaryExpr(Box<Expression>, Operator, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i32),
    Float(f32),
    Boolean(i8),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Divide,
    Multiply,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanEqual,
    LesserThan,
    LesserThanEqual
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier(pub String);

impl Identifier {
    pub fn from_str(name: &str) -> Self {
        Self(name.to_string())
    }

    pub fn from(name: String) -> Self {
        Self(name)
    }
}

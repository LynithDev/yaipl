#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Variable(Identifier, Expression),
    Expr(Expression),
    Return(Expression)
}

pub type Program = Vec<Statement>;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    Prefix(Prefix, Box<Expression>),
    Infix(Operator, Box<Expression>, Box<Expression>),
    List(Vec<Expression>),
    Function(Vec<Identifier>, Program),
    FunctionCall(Box<Expression>, Vec<Expression>),
    If(Box<Expression>, Program),
    While(Box<Expression>, Program),
    For(Box<Statement>, Box<Expression>, Box<Statement>, Program)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i32),
    Float(f32),
    Boolean(i8),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Prefix {
    Plus,
    Minus,
    Not
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
pub struct Identifier {
    pub name: String
}

impl Identifier {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string()
        }
    }

    pub fn from_string(name: String) -> Self {
        Self { 
            name
        }
    }
}

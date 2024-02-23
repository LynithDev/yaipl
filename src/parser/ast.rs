use crate::lexer::token::{Token, TokenType};

macro_rules! create_struct {
    ($name:ident $(, $field_type:ty)*) => {
        #[derive(Debug, PartialEq, Clone)]
        pub struct $name($(pub $field_type),*);
    };
}

create_struct!(StringLiteral, String);
create_struct!(IntegerLiteral, i32);
create_struct!(FloatLiteral, f32);
create_struct!(BooleanLiteral, i8);

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    String(StringLiteral),
    Integer(IntegerLiteral),
    Float(FloatLiteral),
    Boolean(BooleanLiteral)
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArithmeticOperator {
    Plus,
    Minus,
    Divide,
    Multiply,
    Modulo,
    Power,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentOperator {
    PlusAssign,
    MinusAssign,
    DivideAssign,
    MultiplyAssign,
    ModuloAssign,
    PowerAssign,
    Assign,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LogicalOperator {
    Or,
    And,
    Not,
    Equal,
    NotEqual,
    LesserThan,
    GreaterThan,
    LesserThanEqual,
    GreaterThanEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Assignment(AssignmentOperator),
    Arithmetic(ArithmeticOperator),
    Logical(LogicalOperator)
}

pub fn op_token_to_arithmetic(op: &Token) -> Option<ArithmeticOperator> {
    match op.token_type {
        TokenType::Plus => Some(ArithmeticOperator::Plus),
        TokenType::Minus => Some(ArithmeticOperator::Minus),
        TokenType::Divide => Some(ArithmeticOperator::Divide),
        TokenType::Multiply => Some(ArithmeticOperator::Multiply),
        TokenType::Modulo => Some(ArithmeticOperator::Modulo),
        TokenType::Power => Some(ArithmeticOperator::Power),
        _ => None
    }
}

pub fn op_token_to_assignment(op: &Token) -> Option<AssignmentOperator> {
    match op.token_type {
        TokenType::PlusAssign => Some(AssignmentOperator::PlusAssign),
        TokenType::MinusAssign => Some(AssignmentOperator::MinusAssign),
        TokenType::DivideAssign => Some(AssignmentOperator::DivideAssign),
        TokenType::MultiplyAssign => Some(AssignmentOperator::MultiplyAssign),
        TokenType::ModuloAssign => Some(AssignmentOperator::ModuloAssign),
        TokenType::PowerAssign => Some(AssignmentOperator::PowerAssign),
        TokenType::Assign => Some(AssignmentOperator::Assign),
        _ => None
    }
}

pub fn op_token_to_logical(op: &Token) -> Option<LogicalOperator> {
    match op.token_type {
        TokenType::Or => Some(LogicalOperator::Or),
        TokenType::And => Some(LogicalOperator::And),
        TokenType::Not => Some(LogicalOperator::Not),
        TokenType::Equal => Some(LogicalOperator::Equal),
        TokenType::NotEqual => Some(LogicalOperator::NotEqual),
        TokenType::LesserThan => Some(LogicalOperator::LesserThan),
        TokenType::GreaterThan => Some(LogicalOperator::GreaterThan),
        TokenType::LesserThanEqual => Some(LogicalOperator::LesserThanEqual),
        TokenType::GreaterThanEqual => Some(LogicalOperator::GreaterThanEqual),
        _ => None
    }
}

create_struct!(Variable, Identifier, Box<Expression>);
create_struct!(Assignment, Variable, Box<Expression>);
create_struct!(Identifier, String);
create_struct!(BinaryExpression, Box<Expression>, Operator, Box<Expression>);
create_struct!(LogicalExpression, Box<Expression>, LogicalOperator, Box<Expression>);
create_struct!(UnaryExpression, Operator, Box<Expression>);
create_struct!(CallExpression, Box<Expression>, Vec<Expression>);

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    VariableExpr(Variable),
    AssignmentExpr(Assignment),
    LiteralExpr(Literal),
    IdentifierExpr(Identifier),
    BinaryExpr(BinaryExpression),
    LogicalExpr(LogicalExpression),
    UnaryExpr(UnaryExpression),
    CallExpr(CallExpression)
}

pub type Program = Vec<Node>;
create_struct!(BlockStatement, Vec<Node>);
create_struct!(EmptyStatement);
create_struct!(ExpressionStatement, Expression);
create_struct!(ReturnStatement, Option<Expression>);

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Program(Program),
    BlockStatement(BlockStatement),
    ExpressionStatement(ExpressionStatement),
    EmptyStatement(EmptyStatement),
    ReturnStatement(ReturnStatement)
}

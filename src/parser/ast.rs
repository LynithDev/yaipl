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
create_struct!(Null);

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    String(StringLiteral),
    Integer(IntegerLiteral),
    Float(FloatLiteral),
    Boolean(BooleanLiteral),
    Null
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

pub fn assignment_to_arithmetic(op: &AssignmentOperator) -> Option<ArithmeticOperator> {
    match op {
        AssignmentOperator::PlusAssign => Some(ArithmeticOperator::Plus),
        AssignmentOperator::MinusAssign => Some(ArithmeticOperator::Minus),
        AssignmentOperator::DivideAssign => Some(ArithmeticOperator::Divide),
        AssignmentOperator::MultiplyAssign => Some(ArithmeticOperator::Multiply),
        AssignmentOperator::ModuloAssign => Some(ArithmeticOperator::Modulo),
        AssignmentOperator::PowerAssign => Some(ArithmeticOperator::Power),
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

create_struct!(Assignment, Identifier, Box<Expression>);
create_struct!(Identifier, String);
create_struct!(BinaryExpression, Box<Expression>, Operator, Box<Expression>);
create_struct!(UnaryExpression, Operator, Box<Expression>);
create_struct!(FunctionCallExpression, Identifier, Vec<Expression>);
create_struct!(FunctionDeclareExpression, Identifier, Vec<Identifier>, Box<BlockStatement>);

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    AssignmentExpr(Assignment),
    LiteralExpr(Literal),
    IdentifierExpr(Identifier),
    BinaryExpr(BinaryExpression),
    UnaryExpr(UnaryExpression),
    GroupExpr(Box<Expression>),
    BlockExpr(BlockStatement),
    FunctionCallExpr(FunctionCallExpression),
    FunctionDeclareExpr(FunctionDeclareExpression),
}

pub type ProgramTree = Vec<Node>;
pub type Program = Node;
create_struct!(BlockStatement, Vec<Node>);
create_struct!(EmptyStatement);
create_struct!(ContinueStatement);
create_struct!(BreakStatement);
create_struct!(ExpressionStatement, Expression);
create_struct!(ReturnStatement, Option<Expression>);
create_struct!(IfStatement, Expression, Box<BlockStatement>, Option<Box<Node>>);
create_struct!(ElseStatement, Box<BlockStatement>);
create_struct!(WhileStatement, Expression, Box<BlockStatement>);
create_struct!(ForStatement, Expression, Expression, Expression, Box<BlockStatement>);

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Program(ProgramTree),
    BlockStatement(BlockStatement),
    ExpressionStatement(ExpressionStatement),
    EmptyStatement(EmptyStatement),
    ReturnStatement(ReturnStatement),
    ContinueStatement(ContinueStatement),
    BreakStatement(BreakStatement),
    IfStatement(IfStatement),
    ElseStatement(ElseStatement),
    WhileStatement(WhileStatement),
    ForStatement(ForStatement),
}

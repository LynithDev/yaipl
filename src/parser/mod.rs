use std::{error::Error, vec};

use crate::{error, errors::{DynamicError, ParserError}, evaluator::object::FUNCTION_PREFIX, lexer::token::{Token, TokenLiteral, TokenType, Tokens}, parser::ast::Literal, utils::unwrap_result};

use self::ast::{assignment_to_arithmetic, op_token_to_arithmetic, op_token_to_assignment, op_token_to_logical, BlockStatement, EmptyStatement, Expression, ExpressionStatement, Identifier, Node, Program};

pub mod ast;

type ParserResult<T> = Result<T, DynamicError>;

pub struct Parser<'a> {
    pub tokens: &'a Tokens,
    warnings: Vec<Box<dyn Error>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn from(tokens: &'a Tokens) -> Self {
        Self {
            tokens,
            current: 0,
            warnings: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Program, DynamicError> {
        Ok(Node::Program(self.parse_statements()?))
    }

    fn parse_statements(&mut self) -> ParserResult<Vec<Node>> {
        let mut statements: Vec<Node> = Vec::new();

        while !self.is_at_end() {
            let statement = self.declaration()?;
            statements.push(statement);
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> ParserResult<Node> {
        if self.check(TokenType::Symbol) {
            if unwrap_result(self.lookahead())?.token_type == TokenType::Assign {
                return Ok(self.var_declaration()?)
            }
        }

        Ok(self.statement()?)
    }

    fn var_declaration(&mut self) -> ParserResult<Node> {
        let symbol = self.consume(TokenType::Symbol)?;
        let name = match unwrap_result(symbol.value)? {
            TokenLiteral::String(name) => name,
            _ => error!(ParserError::InvalidToken { 
                expected: vec![TokenType::Symbol],
                found: symbol.token_type,
                pos: symbol.start,
            }),
        };
        
        self.consume(TokenType::Assign)?;
        let old_current = self.current;
        
        // Attempt to collect parameters for function declaration
        if self.matches(TokenType::LeftParen) {
            match self.collect_parameters() {
                Ok(parameters) => {
                    if self.check(TokenType::LeftBrace) {
                        return self.func_declaration(Identifier(name), parameters);
                    }
                },
                Err(_) => {} // Collecting parameters failed
            };
        }
        self.current = old_current; // Reset current to before the failed attempt

        let initializer = self.statement()?;

        Ok(Node::ExpressionStatement(ExpressionStatement(
            Expression::AssignmentExpr(ast::Assignment(
                ast::Identifier(name),
                Box::from(initializer),
            ))
        )))
    }
    
    fn collect_parameters(&mut self) -> ParserResult<Vec<Identifier>> {
        let mut arguments: Vec<Identifier> = Vec::new();
        
        loop {
            if self.matches(TokenType::RightParen) {
                break;
            }
            
            let symbol = self.consume(TokenType::Symbol)?;
            let name = match unwrap_result(symbol.value)? {
                TokenLiteral::String(name) => name,
                _ => error!(ParserError::InvalidToken {
                    expected: vec![TokenType::Symbol],
                    found: symbol.token_type,
                    pos: symbol.start,
                }),
            };
            
            arguments.push(Identifier(name));
            
            if !self.matches(TokenType::Comma) {
                if self.matches(TokenType::RightParen) {
                    break;
                }
            }
        }

        Ok(arguments)
    }

    fn func_declaration(&mut self, mut identifier: Identifier, parameters: Vec<Identifier>) -> ParserResult<Node> {
        let body = self.block()?;

        identifier.0 = format!("{}{}", FUNCTION_PREFIX, identifier.0);

        Ok(Node::ExpressionStatement(ExpressionStatement(
            Expression::FunctionDeclareExpr(
                ast::FunctionDeclareExpression(
                    identifier,
                    parameters,
                    Box::from(body)
                )
            )
        )))
    }

    fn block(&mut self) -> ParserResult<BlockStatement> {
        self.consume(TokenType::LeftBrace)?;
        let mut statements: Vec<Node> = Vec::new();

        while !self.is_at_end() && !self.check(TokenType::RightBrace) {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace)?;
        let _ = self.consume(TokenType::EndOfLine);

        Ok(ast::BlockStatement(statements))
    }

    fn statement(&mut self) -> ParserResult<Node> {
        if self.matches(TokenType::EndOfLine) {
            return Ok(Node::EmptyStatement(EmptyStatement()));
        }

        if self.matches(TokenType::If) {
            return self.if_statement();
        }

        if self.matches(TokenType::While) {
            return self.while_statement();
        }

        if self.matches(TokenType::For) {
            return self.for_statement();
        }

        if self.matches(TokenType::Break) {
            return Ok(Node::BreakStatement(ast::BreakStatement()));
        }

        if self.matches(TokenType::Continue) {
            return Ok(Node::ContinueStatement(ast::ContinueStatement()));
        }

        if self.matches(TokenType::Return) {
            return self.return_statement();
        }

        Ok(Node::ExpressionStatement(self.expression_statement()?))
    }

    fn else_statement(&mut self) -> ParserResult<Node> {
        let body = self.block()?;
        Ok(Node::ElseStatement(ast::ElseStatement(Box::from(body))))
    }

    fn if_statement(&mut self) -> ParserResult<Node> {
        let condition = self.expression()?;
        let body = self.block()?;

        let maybe_else = if let Some(token) = self.peek() {
            match token.token_type {
                TokenType::ElIf => {
                    self.consume(TokenType::ElIf)?;
                    Some(Box::from(self.if_statement()?))
                },
                TokenType::Else => {
                    self.consume(TokenType::Else)?;
                    Some(Box::from(self.else_statement()?))
                },
                _ => None
            }
        } else {
            None
        };

        Ok(Node::IfStatement(
            ast::IfStatement(
                condition,
                Box::from(body),
                maybe_else,
            )
        ))
    }

    fn while_statement(&mut self) -> ParserResult<Node> {
        let condition = self.expression()?;
        let body = self.block()?;

        Ok(Node::WhileStatement(
            ast::WhileStatement(
                condition,
                Box::from(body),
            )
        ))
    }

    fn for_statement(&mut self) -> ParserResult<Node> {
        let _ = self.consume(TokenType::LeftParen);
        let variable = self.var_declaration()?;

        let condition = self.or()?;
        self.consume(TokenType::EndOfLine)?;
        
        let assignment = self.assignment()?;
        let _ = self.consume(TokenType::RightParen);

        let body = self.block()?;

        let variable = match variable {
            Node::ExpressionStatement(ExpressionStatement(assignment)) => assignment,
            _ => error!(ParserError::InvalidStatement)
        };

        Ok(Node::ForStatement(
            ast::ForStatement(
                variable,
                condition,
                assignment,
                Box::from(body),
            )
        ))
    }

    fn return_statement(&mut self) -> ParserResult<Node> {
        let return_value = if !self.matches(TokenType::EndOfLine) {
            Some(self.expression()?)
        } else {
            None
        };

        if return_value.is_some() {
            self.consume(TokenType::EndOfLine)?;
        }

        Ok(Node::ReturnStatement(ast::ReturnStatement(
            return_value
        )))
    }

    fn expression_statement(&mut self) -> ParserResult<ExpressionStatement> {
        let expression = self.expression()?;
        if let Some(token) = self.previous() {
            if token.token_type != TokenType::RightBrace {
                self.consume(TokenType::EndOfLine)?;
            }
        }
        Ok(ExpressionStatement(expression))
    }

    fn expression(&mut self) -> ParserResult<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expression> {
        let expression = self.or()?;

        if self.match_one_of(vec![
            TokenType::PlusAssign,
            TokenType::MinusAssign,
            TokenType::MultiplyAssign,
            TokenType::DivideAssign,
            TokenType::ModuloAssign,
        ]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let value = self.assignment()?;

            if let Expression::IdentifierExpr(identifier) = &expression {
                let ast_op = unwrap_result(op_token_to_assignment(&operator))?;
                let arithmetic_op = unwrap_result(assignment_to_arithmetic(&ast_op))?;

                return Ok(Expression::AssignmentExpr(ast::Assignment(
                    identifier.to_owned(),
                    Box::from(Node::ExpressionStatement(
                        ExpressionStatement(
                            Expression::BinaryExpr(ast::BinaryExpression(
                                Box::new(expression),
                                ast::Operator::Arithmetic(arithmetic_op),
                                Box::new(value),
                            ))
                        )
                    )),
                )))
            }
        }

        Ok(expression)
    }

    fn or(&mut self) -> ParserResult<Expression> {
        let mut expression = self.and()?;

        while self.matches(TokenType::Or) {
            let right = self.and()?;
            expression = Expression::BinaryExpr(ast::BinaryExpression(
                Box::new(expression), 
                ast::Operator::Logical(ast::LogicalOperator::Or), 
                Box::new(right)
            ));
        }

        Ok(expression)
    }

    fn and(&mut self) -> ParserResult<Expression> {
        let mut expression = self.equality()?;

        while self.matches(TokenType::And) {
            let right = self.equality()?;
            expression = Expression::BinaryExpr(ast::BinaryExpression(
                Box::new(expression), 
                ast::Operator::Logical(ast::LogicalOperator::And), 
                Box::new(right)
            ));
        }

        Ok(expression)
    }

    fn equality(&mut self) -> ParserResult<Expression> {
        let mut expression = self.comparison()?;

        while self.match_one_of(vec![TokenType::Equal, TokenType::NotEqual]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.comparison()?;
        
            match op_token_to_logical(&operator) {
                None => error!(ParserError::InvalidToken {
                    expected: vec![TokenType::Equal, TokenType::NotEqual],
                    found: operator.token_type,
                    pos: operator.start,
                }),
                Some(op) => {
                    expression = Expression::BinaryExpr(ast::BinaryExpression(
                        Box::new(expression),
                        ast::Operator::Logical(op),
                        Box::new(right),
                    ))
                }
            }
        }

        Ok(expression)
    }

    fn comparison(&mut self) -> ParserResult<Expression> {
        let mut expression = self.addition()?;

        while self.match_one_of(vec![
            TokenType::LesserThan,
            TokenType::GreaterThan,
            TokenType::LesserThanEqual,
            TokenType::GreaterThanEqual,
        ]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.addition()?;

            let comparison_operator = unwrap_result(op_token_to_logical(&operator))?;

            expression = Expression::BinaryExpr(ast::BinaryExpression(
                Box::new(expression), 
                ast::Operator::Logical(comparison_operator), 
                Box::new(right)
            ));
        }

        Ok(expression)
    }

    fn addition(&mut self) -> ParserResult<Expression> {
        let mut expression = self.multiplication()?;

        while self.match_one_of(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.multiplication()?;

            let arithmetic_operator = unwrap_result(op_token_to_arithmetic(&operator))?;

            expression = Expression::BinaryExpr(ast::BinaryExpression(
                Box::new(expression),
                ast::Operator::Arithmetic(arithmetic_operator),
                Box::new(right),
            ));
        }

        Ok(expression)
    }

    fn multiplication(&mut self) -> ParserResult<Expression> {
        let mut expression = self.exponent()?;

        while self.match_one_of(vec![TokenType::Multiply, TokenType::Divide, TokenType::Modulo]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.exponent()?;

            let arithmetic_operator = unwrap_result(op_token_to_arithmetic(&operator))?;

            expression = Expression::BinaryExpr(ast::BinaryExpression(
                Box::new(expression),
                ast::Operator::Arithmetic(arithmetic_operator),
                Box::new(right),
            ));
        }

        Ok(expression)
    }

    fn exponent(&mut self) -> ParserResult<Expression> {
        let mut expression = self.unary()?;

        if self.matches(TokenType::Power) {
            let right = self.unary()?;

            expression = Expression::BinaryExpr(ast::BinaryExpression(
                Box::new(expression),
                ast::Operator::Arithmetic(ast::ArithmeticOperator::Power),
                Box::new(right),
            ));
        }

        Ok(expression)
    }

    fn unary(&mut self) -> ParserResult<Expression> {
        if self.match_one_of(vec![TokenType::Minus, TokenType::Not]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.unary()?;

            let unary_operator = match operator.token_type {
                TokenType::Minus => ast::Operator::Arithmetic(ast::ArithmeticOperator::Minus),
                TokenType::Not => ast::Operator::Logical(ast::LogicalOperator::Not),
                _ => error!(ParserError::InvalidToken {
                    expected: vec![TokenType::Minus, TokenType::Not],
                    found: operator.token_type.to_owned(),
                    pos: operator.start,
                })
            };

            return match unary_operator.to_owned() {
                ast::Operator::Arithmetic(_) => Ok(Expression::UnaryExpr(
                    ast::UnaryExpression(
                        unary_operator,
                        Box::new(right),
                    )
                )),
                ast::Operator::Logical(_) => Ok(Expression::UnaryExpr(
                    ast::UnaryExpression(
                        unary_operator,
                        Box::new(right),
                    )
                )),
                _ => error!(ParserError::InvalidToken {
                    expected: vec![TokenType::Minus, TokenType::Not],
                    found: operator.token_type.to_owned(),
                    pos: operator.start,
                })
            };
        }

        self.call()
    }

    fn call(&mut self) -> ParserResult<Expression> {
        let identifier = unwrap_result(self.peek())?.to_owned();
        if self.matches_all_in_order(vec![TokenType::Symbol, TokenType::LeftParen]) {
            return Ok(self.finish_call(identifier.to_owned())?);
        }
        
        Ok(self.primary()?)
    }

    fn finish_call(&mut self, identifier: Token) -> ParserResult<Expression> {
        let name = match unwrap_result(identifier.value)? {
            TokenLiteral::String(name) => format!("{}{}", FUNCTION_PREFIX, name),
            _ => error!(ParserError::InvalidToken {
                expected: vec![TokenType::String],
                found: identifier.token_type.to_owned(),
                pos: identifier.start,
            })
        };

        let mut arguments: Vec<Expression> = Vec::new();

        loop {
            if self.matches(TokenType::RightParen) {
                break;
            }

            arguments.push(self.expression()?);

            if !self.matches(TokenType::Comma) {
                self.consume(TokenType::RightParen)?;
                break;
            }
        }

        Ok(Expression::FunctionCallExpr(ast::FunctionCallExpression(
            ast::Identifier(name),
            arguments
        )))
    }

    fn primary(&mut self) -> ParserResult<Expression> {
        let token = unwrap_result(self.peek())?.to_owned();
        let value = token.value;

        let result = match token.token_type {
            TokenType::LeftBrace => {
                let block = self.block()?;
                return Ok(Expression::BlockExpr(block));
            },
            TokenType::Null => Expression::LiteralExpr(Literal::Null),
            TokenType::Integer => {
                let value = unwrap_result(value)?.get_value().parse::<i32>()?;
                Expression::LiteralExpr(Literal::Integer(ast::IntegerLiteral(value)))
            },
            TokenType::Float => {
                let value = unwrap_result(value)?.get_value().parse::<f32>()?;
                Expression::LiteralExpr(Literal::Float(ast::FloatLiteral(value)))
            },
            TokenType::Boolean => {
                let value = unwrap_result(value)?.get_value().parse::<i8>()?;
                Expression::LiteralExpr(Literal::Boolean(ast::BooleanLiteral(value)))
            },
            TokenType::String => {
                let value = unwrap_result(value)?.get_value();
                Expression::LiteralExpr(Literal::String(ast::StringLiteral(value)))
            },
            TokenType::Symbol => {
                let value = unwrap_result(value)?.get_value();
                Expression::IdentifierExpr(ast::Identifier(value))
            }
            TokenType::LeftParen => {
                self.advance();
                let expression = self.expression()?;
                self.consume(TokenType::RightParen)?;
                return Ok(Expression::GroupExpr(Box::from(expression)));
            },
            _ => error!(ParserError::InvalidToken {
                expected: vec![TokenType::Integer, TokenType::Float, TokenType::Boolean, TokenType::String, TokenType::Symbol, TokenType::LeftParen],
                found: token.token_type,
                pos: token.start,
            })
        };
        
        self.advance();
        Ok(result)
    }

    fn consume(&mut self, token: TokenType) -> ParserResult<Token> {
        if self.check(token.to_owned()) {
            return Ok(unwrap_result(self.advance())?.to_owned())
        }

        let found = unwrap_result(self.peek())?.to_owned();
        error!(ParserError::InvalidToken {
            expected: vec![token],
            found: found.token_type,
            pos: found.start,
        })
    }

    fn matches_all_in_order(&mut self, tokens: Vec<TokenType>) -> bool {
        for (index, token) in tokens.iter().enumerate() {
            if !self.check(token.to_owned()) {
                self.current -= index;
                return false;
            }

            self.advance();
        }

        true
    }

    fn match_one_of(&mut self, tokens: Vec<TokenType>) -> bool {
        for token in tokens {
            if self.matches(token) {
                return true;
            }
        }

        false
    }

    fn matches(&mut self, token: TokenType) -> bool {
        if self.check(token) {
            self.advance();
            return true;
        }

        false
    }

    fn check(&self, token: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        match self.peek() {
            Some(peek) => peek.token_type == token,
            None => false,
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        self.advance_amt(1)
    }

    fn advance_amt(&mut self, amount: usize) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += amount;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        match unwrap_result(self.peek()) {
            Ok(result) => {
                result.token_type == TokenType::EndOfFile
            },
            Err(_) => true,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn lookahead(&self) -> Option<&Token> {
        if self.current + 1 >= self.tokens.len() {
            return None;
        }

        self.tokens.get(self.current + 1)
    }

    fn previous(&mut self) -> Option<&Token> {
        if self.current == 0 {
            self.warnings.push(ParserError::OutOfBounds { index: String::from("-1") }.into());
            return None;
        }
        
        self.tokens.get(self.current - 1)
    }
}
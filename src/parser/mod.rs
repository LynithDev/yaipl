use std::error::Error;

use crate::{create_error, create_error_list, error, lexer::token::{Token, TokenLiteral, TokenType, Tokens}, parser::ast::Literal, utils::unwrap_result};

use self::ast::{op_token_to_arithmetic, op_token_to_logical, EmptyStatement, Expression, ExpressionStatement, Node};

pub mod ast;

create_error!(TokenMismatch, {
    expected: Vec<TokenType>,
    found: TokenType,
});

create_error_list!(ParserErrors, {
    TokenMismatch,
});

type ParserResult<T> = Result<T, Box<dyn Error>>;

pub struct Parser<'a> {
    pub tokens: &'a Tokens,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn from(tokens: &'a Tokens) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Node, ParserErrors> {
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
        if self.matches(TokenType::Symbol) {
            let symbol = unwrap_result(self.previous())?.to_owned();
            let is_declaration = unwrap_result(self.peek())?.token_type == TokenType::Assign;

            if is_declaration {
                self.advance();
                return Ok(self.var_declaration(symbol)?)
            }
        }

        Ok(self.statement()?)
    }

    fn var_declaration(&mut self, symbol: Token) -> ParserResult<Node> {
        let name = match unwrap_result(symbol.value)? {
            TokenLiteral::String(name) => name,
            _ => error!("Expected identifier, found something else"),
        };

        let initializer = self.expression()?;
        self.consume(TokenType::EndOfLine)?;

        Ok(Node::ExpressionStatement(ExpressionStatement(
            Expression::VariableExpr(ast::Variable(
                ast::Identifier(name),
                Box::from(initializer),
            ))
        )))
    }

    fn statement(&mut self) -> ParserResult<Node> {
        if self.matches(TokenType::EndOfLine) {
            return Ok(Node::EmptyStatement(EmptyStatement()));
        }

        if self.matches(TokenType::Return) {
            return self.return_statement();
        }

        Ok(Node::ExpressionStatement(self.expression_statement()?))
    }

    fn return_statement(&mut self) -> ParserResult<Node> {
        let return_value = if !self.matches(TokenType::EndOfLine) {
            Some(self.expression()?)
        } else {
            None
        };

        if return_value.is_some() {
            // self.consume(TokenType::EndOfLine)?;
        }

        Ok(Node::ReturnStatement(ast::ReturnStatement(
            return_value
        )))
    }

    fn expression_statement(&mut self) -> ParserResult<ExpressionStatement> {
        let expression = self.expression()?;
        self.consume(TokenType::EndOfLine)?;
        Ok(ExpressionStatement(expression))
    }

    fn expression(&mut self) -> ParserResult<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expression> {
        let expression = self.or()?;

        if self.matches(TokenType::Equal) {
            let value = self.assignment()?;

            if let Expression::VariableExpr(variable) = &expression {
                return Ok(Expression::AssignmentExpr(ast::Assignment(
                    variable.to_owned(),
                    Box::new(value),
                )));
            }

            error!("Invalid assignment target");
        }

        Ok(expression)
    }

    fn or(&mut self) -> ParserResult<Expression> {
        let mut expression = self.and()?;

        while self.matches(TokenType::Or) {
            let right = self.and()?;
            expression = Expression::LogicalExpr(ast::LogicalExpression(
                Box::new(expression), 
                ast::LogicalOperator::Or, 
                Box::new(right)
            ));
        }

        Ok(expression)
    }

    fn and(&mut self) -> ParserResult<Expression> {
        let mut expression = self.equality()?;

        while self.matches(TokenType::And) {
            let right = self.equality()?;
            expression = Expression::LogicalExpr(ast::LogicalExpression(
                Box::new(expression), 
                ast::LogicalOperator::And, 
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
        
            match op_token_to_arithmetic(&operator) {
                None => error!(TokenMismatch {
                    err: format!("Expected token of type '{:?}' or '{:?}', found {:?}", TokenType::Equal, TokenType::NotEqual, operator.token_type),
                    expected: vec![TokenType::Equal, TokenType::NotEqual],
                    found: operator.token_type,
                }),
                Some(op) => {
                    expression = Expression::BinaryExpr(ast::BinaryExpression(
                        Box::new(expression),
                        ast::Operator::Arithmetic(op),
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

            expression = Expression::LogicalExpr(ast::LogicalExpression(
                Box::new(expression),
                comparison_operator,
                Box::new(right),
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
        let mut expression = self.unary()?;

        while self.match_one_of(vec![TokenType::Multiply, TokenType::Divide, TokenType::Modulo]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.unary()?;

            let arithmetic_operator = unwrap_result(op_token_to_arithmetic(&operator))?;

            expression = Expression::BinaryExpr(ast::BinaryExpression(
                Box::new(expression),
                ast::Operator::Arithmetic(arithmetic_operator),
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
                _ => error!(TokenMismatch {
                    err: "Expected token of type Minus or Not".to_owned(),
                    expected: vec![TokenType::Minus, TokenType::Not],
                    found: operator.token_type.to_owned(),
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
                _ => error!(TokenMismatch {
                    err: "Expected token of type Arithmetic or Logical".to_owned(),
                    expected: vec![TokenType::Minus, TokenType::Not],
                    found: operator.token_type.to_owned(),
                })
            };
        }

        self.call()
    }

    fn call(&mut self) -> ParserResult<Expression> {
        let mut expression = self.primary()?;

        loop {
            if self.matches(TokenType::LeftParen) {
                expression = self.finish_call(expression)?;
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn finish_call(&mut self, callee: Expression) -> ParserResult<Expression> {
        let mut arguments: Vec<Expression> = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.matches(TokenType::Comma) {
                    break;
                }
            }
        }

        match self.consume(TokenType::RightParen) {
            Ok(token) => token,
            Err(_) => error!(TokenMismatch {
                err: "Expected ) after arguments".to_owned(),
                expected: vec![TokenType::RightParen],
                found: unwrap_result(self.peek())?.token_type.to_owned(),
            }),
        };

        Ok(Expression::FunctionCallExpr(
            ast::FunctionCallExpression(
                Box::new(callee),
                arguments,
            )
        ))
    }

    fn primary(&mut self) -> ParserResult<Expression> {
        let token = unwrap_result(self.peek())?.to_owned();
        let value = token.value;

        let result = match token.token_type {
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
            TokenType::LeftParen => {
                self.advance();
                let expression = self.expression()?;
                self.consume(TokenType::RightParen)?;
                return Ok(Expression::GroupExpr(Box::from(expression)));
            }
            TokenType::Symbol => {
                let value = unwrap_result(value)?.get_value();
                Expression::IdentifierExpr(ast::Identifier(value))
            },
            _ => error!(format!("Expected expression, received '{:?}'", token.token_type)),
        };
        
        self.advance();
        Ok(result)
    }

    fn consume(&mut self, token: TokenType) -> ParserResult<Token> {
        if self.check(token.to_owned()) {
            return Ok(unwrap_result(self.advance())?.to_owned())
        }

        let found = unwrap_result(self.peek())?.to_owned();
        error!(TokenMismatch {
            err: format!("Expected token of type {:?}, found {:?}", token, found.token_type),
            expected: vec![token],
            found: found.token_type,
        })
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
        if !self.is_at_end() {
            self.current += 1;
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

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }
}
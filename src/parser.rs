use std::rc::Rc;

use crate::{
    errors,
    expr::{Binary, Expression, Grouping, Literal, Unary},
    token::{Token, TokenLiteral},
    token_type::TokenType,
};

#[derive(Debug)]
struct ParseError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Expression> {
        if let Ok(expr) = self.expression() {
            Some(expr)
        } else {
            None
        }
    }

    // Basically equality with extra steps
    fn expression(&mut self) -> Result<Expression, ParseError> {
        let expr = self.equality();
        expr
    }

    // Translation of equality rule into syntax tree, if it never encounters an
    // equality expression, it'll call and return comparison() which will match
    // anything with a higher precedence than equality.
    fn equality(&mut self) -> Result<Expression, ParseError> {
        self.comparison().map(|mut expr| {
            while self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
                // Parse an equality expression
                let operator = self.previous();
                if let Ok(right) = self.comparison() {
                    expr = Binary::new(expr, operator, right);
                }
            }
            expr
        })
    }

    // Matches anything with a higher precedence than equality.
    fn comparison(&mut self) -> Result<Expression, ParseError> {
        self.term().map(|mut expr| {
            while self.matches(&[
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ]) {
                let operator = self.previous();
                if let Ok(right) = self.term() {
                    expr = Binary::new(expr, operator, right);
                }
            }

            expr
        })
    }

    // Addition and subtraction
    fn term(&mut self) -> Result<Expression, ParseError> {
        self.factor().map(|mut expr| {
            while self.matches(&[TokenType::Plus, TokenType::Minus]) {
                let operator = self.previous();
                if let Ok(right) = self.factor() {
                    expr = Binary::new(expr, operator, right);
                }
            }
            expr
        })
    }

    // Multiplication and division
    fn factor(&mut self) -> Result<Expression, ParseError> {
        self.unary().map(|mut expr| {
            while self.matches(&[TokenType::Slash, TokenType::Star]) {
                let operator = self.previous();
                if let Ok(right) = self.unary() {
                    expr = Binary::new(expr, operator, right);
                }
            }

            expr
        })
    }

    // Binary operators
    fn unary(&mut self) -> Result<Expression, ParseError> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            if let Ok(right) = self.unary() {
                return Ok(Unary::new(operator, right));
            }
        }

        self.primary()
    }

    // Primary Expressions (highest level of precedence)
    fn primary(&mut self) -> Result<Expression, ParseError> {
        if self.matches(&[TokenType::False]) {
            return Ok(Literal::new(TokenLiteral::False));
        }
        if self.matches(&[TokenType::True]) {
            return Ok(Literal::new(TokenLiteral::True));
        }
        if self.matches(&[TokenType::Nil]) {
            return Ok(Literal::new(TokenLiteral::None));
        }
        if self.matches(&[TokenType::Number, TokenType::String]) {
            return Ok(Literal::new(self.previous().literal));
        }
        if self.matches(&[TokenType::LeftParen]) {
            // Try to end an expression. If we can't end it, we'll end up returning
            // an error.
            if let Ok(expression) = self.expression() {
                if let Ok(_right_paren) =
                    self.consume(TokenType::RightParen, "Expect ')' after expression.")
                {
                    return Ok(Grouping::new(expression));
                } else {
                    return Err(ParseError);
                }
            }
        }

        // If we have a valid token literal, return it as a literal token,
        // otherwise, return error.
        let err_token = self.peek();
        Err(self.error(err_token, "Expect expression."))
    }

    // Checks to see if the current token has any of the passed types
    fn matches(&mut self, types: &[TokenType]) -> bool {
        for ty in types {
            if self.check(ty) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, ty: TokenType, message: impl Into<String>) -> Result<Token, ParseError> {
        if self.check(&ty) {
            return Ok(self.advance());
        }

        let err_token = self.peek();

        // Enter panic mode...
        Err(self.error(err_token, message))
    }

    // Checks if the current token is equal to the passed type
    fn check(&mut self, ty: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return &self.peek().ty == ty;
    }

    // Consumes the current token and returns it.
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current = self.current + 1;
        }
        self.previous()
    }

    // Returns the current token
    fn peek(&mut self) -> Token {
        unsafe { self.tokens.get_unchecked(self.current).clone() }
    }

    // Returns the previously consumed token
    fn previous(&mut self) -> Token {
        unsafe { self.tokens.get_unchecked(self.current - 1).clone() }
    }

    fn error(&mut self, token: Token, message: impl Into<String>) -> ParseError {
        if token.ty == TokenType::Eof {
            errors::report(token.line, " at end", message);
        } else {
            errors::report(token.line, format!("at \"{}\"", token.lexeme), message);
        }

        ParseError
    }

    // Synchronization mechanism for error recovery. Discards tokens until we
    // think we have found a statement boundary.
    #[allow(dead_code)]
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().ty == TokenType::Semicolon {
                return;
            }

            match self.peek().ty {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    // Whether we're at the end of the token stream
    fn is_at_end(&mut self) -> bool {
        self.peek().ty == TokenType::Eof
    }
}

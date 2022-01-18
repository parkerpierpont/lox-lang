use crate::{
    errors,
    expr::{Assign, Binary, Call, Expression, Grouping, Literal, Logical, Unary, Variable},
    object::LoxNil,
    stmt::{
        BlockStmt, ExprStmt, FunStmt, IfStmt, PrintStmt, ReturnStmt, Statement, VariableStmt,
        WhileStmt,
    },
    token::{Token, TokenLiteral},
    token_type::TokenType,
};

#[derive(Debug)]
pub struct ParseError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Some(decl) => {
                    statements.push(decl);
                }
                // When we run into a parse error, we don't return anything.
                // Instead, we call synchronize() to try to recover.
                None => {}
            }
        }

        statements
    }

    // Parse an expression
    fn expression(&mut self) -> Result<Expression, ParseError> {
        self.assignment()
    }

    // Parse a declaration, and try to recover if possible using synchronize in
    // the case that we have a ParseError.
    fn declaration(&mut self) -> Option<Statement> {
        let result = if self.matches(&[TokenType::Fun]) {
            self.function("function")
        } else if self.matches(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match result {
            // Parsing succeeded
            Ok(statement) => Some(statement),
            Err(_parse_error) => {
                self.synchronize();
                None
            }
        }
    }

    // Parse a variable declaration statement.
    fn var_declaration(&mut self) -> Result<Statement, ParseError> {
        let name = match self.consume(TokenType::Identifier, "Expect variable name.") {
            Ok(name) => name,
            Err(parse_error) => return Err(parse_error),
        };

        // An initializer statement is optional.
        let mut initializer = None;
        if self.matches(&[TokenType::Equal]) {
            match self.expression() {
                Ok(expr) => {
                    initializer = Some(expr);
                }
                Err(parse_error) => return Err(parse_error),
            };
        }

        if let Err(parse_error) = self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        ) {
            return Err(parse_error);
        }

        Ok(VariableStmt::new(name, initializer))
    }

    // Parse a while statement
    fn while_statement(&mut self) -> Result<Statement, ParseError> {
        match self.consume(TokenType::LeftParen, "Expect '(' after 'while'.") {
            Err(parse_error) => return Err(parse_error),
            _ => {}
        };
        let condition = match self.expression() {
            Ok(cond_expr) => cond_expr,
            Err(parse_error) => return Err(parse_error),
        };
        match self.consume(TokenType::RightParen, "Expect ')' after condition.") {
            Err(parse_error) => return Err(parse_error),
            _ => {}
        };
        let body = match self.statement() {
            Ok(body_stmt) => body_stmt,
            Err(parse_error) => return Err(parse_error),
        };

        Ok(WhileStmt::new(condition, body))
    }

    // Parse a statement
    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.matches(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.matches(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.matches(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.matches(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.matches(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.matches(&[TokenType::LeftBrace]) {
            return match self.block() {
                Ok(statements) => Ok(BlockStmt::new(statements)),
                Err(parse_error) => Err(parse_error),
            };
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Statement, ParseError> {
        match self.consume(TokenType::LeftParen, "Expect '(' after 'for'.") {
            Err(parse_error) => return Err(parse_error),
            _ => {}
        };

        // The initializer – can be None, a Variable decl., or an expression.
        let initializer = if self.matches(&[TokenType::Semicolon]) {
            // Branch when the initializer has been omitted
            None
        } else if self.matches(&[TokenType::Var]) {
            // Branch when the initializer is a variable declaration
            match self.var_declaration() {
                Ok(var_stmt) => Some(var_stmt),
                Err(parse_error) => return Err(parse_error),
            }
        } else {
            // Branch when the initializer is just a statement.
            match self.expression_statement() {
                Ok(expr_statement) => Some(expr_statement),
                Err(parse_error) => return Err(parse_error),
            }
        };

        let condition = match self.check(&TokenType::Semicolon) {
            // If it's a semicolon, we have no condition.
            true => None,
            // If it's not a semicolon, we have a condition statement.
            false => match self.expression() {
                Ok(condition_expr) => Some(condition_expr),
                Err(parse_error) => return Err(parse_error),
            },
        };

        // Make sure a semicolon exists after the loop condition.
        match self.consume(TokenType::Semicolon, "Expect ';' after loop condition.") {
            Err(parse_error) => return Err(parse_error),
            _ => {}
        };

        let increment = match self.check(&TokenType::RightParen) {
            // If no right paren, we have an increment expression.
            false => match self.expression() {
                Ok(increment_expr) => Some(increment_expr),
                Err(parse_error) => return Err(parse_error),
            },
            // If we have a right paren, we don't have an increment expression.
            true => None,
        };

        // Make sure a right paren exists after the increment's position.
        match self.consume(TokenType::RightParen, "Expect ')' after for clauses.") {
            Err(parse_error) => return Err(parse_error),
            _ => {}
        };

        let mut body = match self.statement() {
            Ok(body_stmt) => body_stmt,
            Err(parse_error) => return Err(parse_error),
        };

        //
        // Desugaring process (into a while-loop)
        //

        // If we have a increment statement, it should execute after the body.
        if let Some(increment) = increment {
            body = BlockStmt::new(vec![body, ExprStmt::new(increment)])
        };

        // Set the body to a while statement. If no condition was specified, set
        // the condition to 'True'.
        if let Some(condition) = condition {
            body = WhileStmt::new(condition, body);
        } else {
            body = WhileStmt::new(Literal::new(TokenLiteral::True), body);
        };

        // If we have an initializer, it should run once before the entire loop.
        if let Some(initializer) = initializer {
            body = BlockStmt::new(vec![initializer, body]);
        }

        Ok(body)
    }

    // Parse an 'if' statement
    fn if_statement(&mut self) -> Result<Statement, ParseError> {
        if let Err(parse_error) = self.consume(TokenType::LeftParen, "Expect '(' after 'if'.") {
            return Err(parse_error);
        }
        let condition = match self.expression() {
            Ok(expr) => expr,
            Err(parse_error) => return Err(parse_error),
        };
        if let Err(parse_error) =
            self.consume(TokenType::RightParen, "Expect ')' after 'if' condition.")
        {
            return Err(parse_error);
        }
        let then_branch = match self.statement() {
            Ok(stmt) => stmt,
            Err(parse_error) => return Err(parse_error),
        };
        let else_branch = match self.matches(&[TokenType::Else]) {
            true => match self.statement() {
                Ok(stmt) => Some(stmt),
                Err(parse_error) => return Err(parse_error),
            },
            false => None,
        };

        Ok(IfStmt::new(condition, then_branch, else_branch))
    }

    // Parse a print statement
    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        let value = self.expression();
        let consume_semi = self.consume(TokenType::Semicolon, "Expect ';' after value.");
        match (value, consume_semi) {
            // Continue with no error
            (Ok(value), Ok(_)) => Ok(PrintStmt::new(value)),
            // Take the semi error
            (Ok(_), Err(err))
            // Take the value error
            | (Err(err), Ok(_))
            // Take the value error
            | (Err(err), Err(_)) => Err(err),
        }
    }

    // Parse a return statement
    fn return_statement(&mut self) -> Result<Statement, ParseError> {
        let keyword = self.previous();
        // We know that there's a value if there's no semicolon (since
        // expressions can't start with semicolons).
        let value = if !self.check(&TokenType::Semicolon) {
            match self.expression() {
                Ok(expr) => expr,
                Err(parse_error) => return Err(parse_error),
            }
        } else {
            // If there's no return value, return Nil.
            Literal::new(TokenLiteral::None)
        };

        match self.consume(TokenType::Semicolon, "Expect ';' after return value.") {
            _ => Ok(ReturnStmt::new(keyword, value)),
            Err(parse_error) => return Err(parse_error),
        }
    }

    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression();
        let consume_semi = self.consume(TokenType::Semicolon, "Expect ';' after value.");
        match (expr, consume_semi) {
            // Continue with no error
            (Ok(expr), Ok(_)) => Ok(ExprStmt::new(expr)),
            // Take the semi error
            (Ok(_), Err(err))
            // Take the value error
            | (Err(err), Ok(_))
            // Take the value error
            | (Err(err), Err(_)) => Err(err),
        }
    }

    fn function(&mut self, kind: impl Into<String>) -> Result<Statement, ParseError> {
        let kind: String = kind.into();
        let name = match self.consume(TokenType::Identifier, format!("Expect {} name.", kind)) {
            Ok(ident_token) => ident_token,
            Err(parse_error) => return Err(parse_error),
        };

        if let Err(parse_error) = self.consume(
            TokenType::LeftParen,
            format!("Expect '(' after {} name.", kind),
        ) {
            return Err(parse_error);
        }

        let mut parameters = vec![];
        if !self.check(&TokenType::RightParen) {
            if parameters.len() >= 255 {
                let next_token = self.peek();
                return Err(self.error(next_token, "Can't have more than 255 parameters."));
            }

            loop {
                match self.consume(TokenType::Identifier, "Expect parameter name.") {
                    Ok(param) => parameters.push(param),
                    Err(parse_error) => return Err(parse_error),
                }

                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        if let Err(parse_error) =
            self.consume(TokenType::RightParen, "Expect ')' after parameters.")
        {
            return Err(parse_error);
        }

        if let Err(parse_error) = self.consume(
            TokenType::LeftBrace,
            format!("Expect '{}' before {} body.", "{", kind),
        ) {
            return Err(parse_error);
        }

        match self.block() {
            Ok(body) => Ok(FunStmt::new(name, parameters, body)),
            Err(parse_error) => return Err(parse_error),
        }
    }

    fn block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = vec![];

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if let Some(declaration) = self.declaration() {
                statements.push(declaration);
            }
        }

        if let Err(parse_error) = self.consume(TokenType::RightBrace, "Expect '}' after block.") {
            return Err(parse_error);
        }

        Ok(statements)
    }

    // Parsing an assignment expression with only single-token lookahead.
    fn assignment(&mut self) -> Result<Expression, ParseError> {
        // Parse next expression (could be left-hand-side of an assignment)
        let expr = match self.or() {
            // Try to match equality
            Ok(expr) => expr,
            // If equality doesn't work, try to parse short-circut
            Err(parse_error) => return Err(parse_error),
        };

        // If equals is the next token, 'expr' is the left-hand-side of an
        // assignment.
        if self.matches(&[TokenType::Equal]) {
            // The assignment.
            let equals = self.previous();
            // For the assignment's right-hand expr (value), we need to recurse.
            let value = match self.assignment() {
                Ok(val) => val,
                Err(parse_error) => return Err(parse_error),
            };

            // If the left-hand-side is indeed a variable, we can proceed.
            //
            // NOTE: Maybe we should check this earlier (like at the top
            // if-statement?)
            if expr.name() == "Variable" {
                if let Ok(variable) = expr.downcast_rc::<Variable>() {
                    return Ok(Assign::new(variable.name.clone(), value));
                }
            }

            // Otherwise, bail out.
            return Err(self.error(equals, "Invalid assignment target."));
        }

        // Return the equality expression if the next token isn't
        // 'TokenType::Equal'.
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expression, ParseError> {
        let mut expr = match self.and() {
            Ok(expr) => expr,
            Err(parse_error) => return Err(parse_error),
        };

        while self.matches(&[TokenType::Or]) {
            let operator = self.previous();
            let right = match self.and() {
                Ok(expr) => expr,
                Err(parse_error) => return Err(parse_error),
            };
            expr = Logical::new(expr, operator, right);
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expression, ParseError> {
        let mut expr = match self.equality() {
            Ok(expr) => expr,
            Err(parse_error) => return Err(parse_error),
        };

        while self.matches(&[TokenType::And]) {
            let operator = self.previous();
            let right = match self.equality() {
                Ok(expr) => expr,
                Err(parse_error) => return Err(parse_error),
            };
            expr = Logical::new(expr, operator, right);
        }

        Ok(expr)
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

        self.call()
    }

    // Parsing an arguments list between parens, with zero-or-more arguments.
    fn finish_call(&mut self, callee: Expression) -> Result<Expression, ParseError> {
        let mut arguments = vec![];

        // If we have a right paren, this is never run (zero-argument fn).
        if !self.check(&TokenType::RightParen) {
            loop {
                // If we don't have a right-paren, we expect at least one argument.
                match self.expression() {
                    Ok(expr) => arguments.push(expr),
                    Err(parse_error) => return Err(parse_error),
                };

                // If we don't match a comma being right after the argument
                // expression, we are done parsing the arguments list.
                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        match self.consume(TokenType::RightParen, "Expect ')' after arguments.") {
            // If we have a right-paren, we can returna new Call expression.
            Ok(paren) => Ok(Call::new(callee, paren, arguments)),
            // Otherwise we have to return a ParseError.
            Err(parse_error) => return Err(parse_error),
        }
    }

    fn call(&mut self) -> Result<Expression, ParseError> {
        let mut expr = match self.primary() {
            Ok(primary_expr) => primary_expr,
            Err(parse_error) => return Err(parse_error),
        };

        loop {
            if self.matches(&[TokenType::LeftParen]) {
                match self.finish_call(expr) {
                    Ok(finish_call_expr) => {
                        expr = finish_call_expr;
                    }
                    Err(parse_error) => return Err(parse_error),
                };
            } else {
                break;
            }
        }

        Ok(expr)
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
        if self.matches(&[TokenType::Identifier]) {
            return Ok(Variable::new(self.previous()));
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

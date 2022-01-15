use crate::{
    errors,
    shared_traits::{CharAt, CharLen, Substring},
    token::{Token, TokenLiteral},
    token_type::TokenType,
};

#[derive(Debug, Clone)]
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: &String) -> Self {
        let source = source.clone();
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            "",
            TokenLiteral::None,
            self.line,
        ));

        self.tokens()
    }

    fn scan_token(&mut self) {
        let advance = self.advance();
        match advance {
            '(' => self.add_etoken(TokenType::LeftParen),
            ')' => self.add_etoken(TokenType::RightParen),
            '{' => self.add_etoken(TokenType::LeftBrace),
            '}' => self.add_etoken(TokenType::RightBrace),
            ',' => self.add_etoken(TokenType::Comma),
            '.' => self.add_etoken(TokenType::Dot),
            '-' => self.add_etoken(TokenType::Minus),
            '+' => self.add_etoken(TokenType::Plus),
            ';' => self.add_etoken(TokenType::Semicolon),
            '*' => self.add_etoken(TokenType::Star),
            '!' => {
                if self.matches('=') {
                    self.add_etoken(TokenType::BangEqual)
                } else {
                    self.add_etoken(TokenType::Bang)
                }
            }
            '=' => {
                if self.matches('=') {
                    self.add_etoken(TokenType::EqualEqual)
                } else {
                    self.add_etoken(TokenType::Equal)
                }
            }
            '<' => {
                if self.matches('=') {
                    self.add_etoken(TokenType::LessEqual)
                } else {
                    self.add_etoken(TokenType::Less)
                }
            }
            '>' => {
                if self.matches('=') {
                    self.add_etoken(TokenType::GreaterEqual)
                } else {
                    self.add_etoken(TokenType::Greater)
                }
            }
            '/' => {
                if self.matches('/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_etoken(TokenType::Slash);
                }
            }
            // Whitespace – No op
            ' ' | '\r' | '\t' => {}
            // Whitespace (newline) – increment the current line count
            '\n' => self.increment_line(),
            // The start of a string
            '"' => self.string(),
            c => match c {
                c if Self::is_digit(c) => self.number(),
                c if Self::is_alpha(c) => self.identifier(),
                _ => errors::error(self.line, format!("Unexpected character \"{:?}\".", c)),
            },
        }
    }

    fn identifier(&mut self) {
        while Self::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let value = self.source.substring(self.start, self.current).to_string();
        if let Some(reserved_token_type) = Self::get_reserved_token_type(value.clone()) {
            self.add_token(reserved_token_type, value);
        } else {
            self.add_token(TokenType::Identifier, value);
        }
    }

    fn number(&mut self) {
        // Advance until we're out of numbers
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        let value: f64;

        // Look for a fractional part.
        if self.peek() == '.' && Self::is_digit(self.peek_next()) {
            // Consume the '.'.
            self.advance();
            // Advance until the numbers end
            while Self::is_digit(self.peek()) {
                self.advance();
            }

            value = self
                .source
                .substring(self.start, self.current)
                .parse::<f64>()
                .unwrap();
        } else {
            value = self
                .source
                .substring(self.start, self.current)
                .parse::<i64>()
                .unwrap() as f64
        }

        self.add_token(TokenType::Number, value);
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.increment_line();
            }
            self.advance();
        }

        if self.is_at_end() {
            return;
        }

        // The closeing '"'.
        self.advance();

        // Trim the surrounding quotes.
        let value = self
            .source
            .substring(self.start + 1, self.current - 1)
            .to_string();

        self.add_token(TokenType::String, value);
    }

    // Simpler 'is_numeric()'
    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    // Simpler 'is_alphabetic()'
    fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    // Simpler 'is_alphanumeric()'
    fn is_alphanumeric(c: char) -> bool {
        Self::is_alpha(c) || Self::is_digit(c)
    }

    // Like advance, but it doesn't consume the character. (Lookahead)
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.char_at(self.current).unwrap()
        }
    }

    // Like peek, but checks next-next character
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.char_length() {
            '\0'
        } else {
            self.source.char_at(self.current + 1).unwrap()
        }
    }

    // Conditional advance, only consumes if the expected character matches
    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source.char_at(self.current) != Some(expected) {
            return false;
        }
        // Advance only if it matches
        self.current = self.current + 1;
        true
    }

    // Consumes the next character in the source file and returns it.
    fn advance(&mut self) -> char {
        let current = self.source.char_at(self.current).unwrap();
        self.current = self.current + 1;
        current
    }

    // Increments the line number
    fn increment_line(&mut self) {
        self.line = self.line + 1;
    }

    // Adds a new token to our tokens list (without an associated literal)
    fn add_etoken(&mut self, ty: impl Into<TokenType>) {
        self.add_token(ty, TokenLiteral::None);
    }

    // Adds a new token to our tokens list
    fn add_token(&mut self, ty: impl Into<TokenType>, literal: impl Into<TokenLiteral>) {
        let text = self.source.substring(self.start, self.current);
        self.tokens
            .push(Token::new(ty, text, literal.into(), self.line));
    }

    // Whether we've consumed all of the characters or not.
    fn is_at_end(&self) -> bool {
        self.current > self.source.char_length()
    }

    // If the identifier passed in has the value as a reserved word, then we
    // pass back the tokentype for that reserved word
    fn get_reserved_token_type(name: String) -> Option<TokenType> {
        match name.as_str() {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }

    /// Consumes self and returns a list of tokens.
    fn tokens(self) -> Vec<Token> {
        self.tokens
    }
}

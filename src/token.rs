use crate::token_type::TokenType;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TokenLiteral {
    String(String),
    Number(f64),
    False,
    True,
    None,
}

impl ToString for TokenLiteral {
    fn to_string(&self) -> String {
        match self {
            TokenLiteral::None => "None".to_string(),
            TokenLiteral::String(v) => v.clone(),
            TokenLiteral::Number(v) => v.to_string(),
            TokenLiteral::False => "False".to_string(),
            TokenLiteral::True => "True".to_string(),
        }
    }
}

impl Into<TokenLiteral> for String {
    fn into(self) -> TokenLiteral {
        TokenLiteral::String(self)
    }
}

impl Into<TokenLiteral> for f64 {
    fn into(self) -> TokenLiteral {
        TokenLiteral::Number(self)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub lexeme: String,
    pub literal: TokenLiteral,
    pub line: usize,
}

impl Token {
    pub fn new(
        ty: impl Into<TokenType>,
        lexeme: impl Into<String>,
        literal: impl Into<TokenLiteral>,
        line: impl Into<usize>,
    ) -> Self {
        Self {
            ty: ty.into(),
            lexeme: lexeme.into(),
            literal: literal.into(),
            line: line.into(),
        }
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        self.ty.to_string() + " " + self.lexeme.as_str() + " " + self.literal.to_string().as_str()
    }
}

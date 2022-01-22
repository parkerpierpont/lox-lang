#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // End of file
    Eof,
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::LeftParen => "LeftParen".to_string(),
            TokenType::RightParen => "RightParen".to_string(),
            TokenType::LeftBrace => "LeftBrace".to_string(),
            TokenType::RightBrace => "RightBrace".to_string(),
            TokenType::Comma => "Comma".to_string(),
            TokenType::Dot => "Dot".to_string(),
            TokenType::Minus => "Minus".to_string(),
            TokenType::Plus => "Plus".to_string(),
            TokenType::Semicolon => "Semicolon".to_string(),
            TokenType::Slash => "Slash".to_string(),
            TokenType::Star => "Star".to_string(),
            TokenType::Bang => "Bang".to_string(),
            TokenType::BangEqual => "BangEqual".to_string(),
            TokenType::Equal => "Equal".to_string(),
            TokenType::EqualEqual => "EqualEqual".to_string(),
            TokenType::Greater => "Greater".to_string(),
            TokenType::GreaterEqual => "GreaterEqual".to_string(),
            TokenType::Less => "Less".to_string(),
            TokenType::LessEqual => "LessEqual".to_string(),
            TokenType::Identifier => "Identifier".to_string(),
            TokenType::String => "String".to_string(),
            TokenType::Number => "Number".to_string(),
            TokenType::And => "And".to_string(),
            TokenType::Class => "Class".to_string(),
            TokenType::Else => "Else".to_string(),
            TokenType::False => "False".to_string(),
            TokenType::Fun => "Fun".to_string(),
            TokenType::For => "For".to_string(),
            TokenType::If => "If".to_string(),
            TokenType::Nil => "Nil".to_string(),
            TokenType::Or => "Or".to_string(),
            TokenType::Print => "Print".to_string(),
            TokenType::Return => "Return".to_string(),
            TokenType::Super => "Super".to_string(),
            TokenType::This => "This".to_string(),
            TokenType::True => "True".to_string(),
            TokenType::Var => "Var".to_string(),
            TokenType::While => "While".to_string(),
            TokenType::Eof => "Eof".to_string(),
        }
    }
}

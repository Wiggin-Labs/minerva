use vm::Value;

use string_interner::Symbol;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    Comment(String),
    BlockComment(String),
    LeftParen,
    RightParen,
    Dot,
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplice,
    Pound,
    String(String),
    Integer(i32),
    Float(f64),
    //ComplexExact(Option<String>, Option<String>),
    //ComplexFloating(Option<String>, Option<String>),
    Symbol(Symbol),
}

impl Token {
    pub fn is_primitive(&self) -> bool {
        matches!(self, Token::String(_) | Token::Integer(_) | Token::Float(_))
    }

    pub fn to_primitive(&self) -> Value {
        match self {
            Token::String(s) => Value::String(s.to_string()),
            Token::Integer(i) => Value::Integer(*i),
            Token::Float(i) => Value::Float(*i),
            _ => unreachable!(),
        }
    }

    pub fn is_left_paren(&self) -> bool {
        matches!(self, Token::LeftParen)
    }

    pub fn is_right_paren(&self) -> bool {
        matches!(self, Token::RightParen)
    }
}

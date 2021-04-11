use vm::Value;

use string_interner::Symbol;

#[derive(Debug, Clone, PartialEq, PartialOrd, is_enum_variant)]
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
        match self {
            Token::String(_) | Token::Integer(_) | Token::Float(_) => true,
            _ => false,
        }
    }

    pub fn to_primitive(&self) -> Value {
        match self {
            Token::String(s) => Value::String(s.to_string()),
            Token::Integer(i) => Value::Integer(*i),
            Token::Float(i) => Value::Float(*i),
            _ => unreachable!(),
        }
    }
}

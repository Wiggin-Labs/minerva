use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParseError {
    EOF,
    InString,
    Input,
    Token,
    UnbalancedParen,
    BadQuote,
    UnexpectedCloseParen,
    IllegalUse,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::EOF => write!(f, "Unexpected end of input"),
            ParseError::InString => write!(f, "Unexpected end of input in string"),
            ParseError::Input => write!(f, "Unexpected input"),
            ParseError::Token => write!(f, "Unexpected token"),
            ParseError::UnbalancedParen => write!(f, "Expected a `)` to close `(`"),
            ParseError::BadQuote => write!(f, "Expected an element for quoting, found EOF"),
            ParseError::UnexpectedCloseParen => write!(f, "Unexpected `)`"),
            ParseError::IllegalUse => write!(f, "Illegal use of `.`"),
        }
    }
}

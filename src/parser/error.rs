use std::fmt::{self, Display, Formatter};
use std::option::NoneError;

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

impl From<NoneError> for ParseError {
    fn from(_e: NoneError) -> ParseError {
        ParseError::EOF
    }
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

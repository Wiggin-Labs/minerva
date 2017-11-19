use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParseError {
    EOF,
    Input,
    Token,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::EOF => write!(f, "Unexpected end of input"),
            ParseError::Input => write!(f, "Unexpected input"),
            ParseError::Token => write!(f, "Unexpected token"),
        }
    }
}

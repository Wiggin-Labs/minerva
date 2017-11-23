use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    UnboundVariable(String),
    PairExpected,
    NumberExpected,
    WrongArgs,
    ElseNotLast,
    UserDefined(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::UnboundVariable(v) => write!(f, "Unbound variable {}", v),
            Error::PairExpected => write!(f, "PAIR expected"),
            Error::NumberExpected => write!(f, "NUMBER expected"),
            Error::WrongArgs => write!(f, "Incorrect number of arguments passed to procedure"),
            Error::ElseNotLast => write!(f, "Else expression not last"),
            Error::UserDefined(e) => write!(f, "{}", e),
        }
    }
}

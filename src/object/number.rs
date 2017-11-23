use parser::Token;
use num::{BigInt, One, Zero};

use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub enum Number {
    Integer(BigInt),
    Rational,
    Real,
    Complex,
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::Number::*;
        match self {
            Integer(n) => write!(f, "{}", n),
            _ => unimplemented!(),
        }
    }
}

impl Number {
    pub fn from_token(t: &Token) -> Self {
        match t {
            Token::Number(s) => Number::Integer(s.parse().unwrap()),
            _ => panic!("compiler error"),
        }
    }

    pub fn zero() -> Self {
        Number::Integer(BigInt::zero())
    }

    pub fn one() -> Self {
        Number::Integer(BigInt::one())
    }

    pub fn is_integer(&self) -> bool {
        match self {
            Number::Integer(_) => true,
            _ => false,
        }
    }

    pub fn is_rational(&self) -> bool {
        match self {
            Number::Rational => true,
            _ => false,
        }
    }

    pub fn is_real(&self) -> bool {
        match self {
            Number::Real => true,
            _ => false,
        }
    }

    pub fn is_complex(&self) -> bool {
        match self {
            Number::Complex => true,
            _ => false,
        }
    }
}

impl Add for Number {
    type Output = Number;

    fn add(self, other: Number) -> Number {
        use self::Number::*;
        match (self, other) {
            (Integer(n1), Integer(n2)) => Integer(n1 + n2),
            _ => unimplemented!(),
        }
    }
}

impl Neg for Number {
    type Output = Number;

    fn neg(self) -> Number {
        use self::Number::*;
        match self {
            Integer(n) => Integer(-n),
            _ => unimplemented!(),
        }
    }
}

impl Sub for Number {
    type Output = Number;

    fn sub(self, other: Number) -> Number {
        use self::Number::*;
        match (self, other) {
            (Integer(n1), Integer(n2)) => Integer(n1 - n2),
            _ => unimplemented!(),
        }
    }
}

impl Mul for Number {
    type Output = Number;

    fn mul(self, other: Number) -> Number {
        use self::Number::*;
        match (self, other) {
            (Integer(n1), Integer(n2)) => Integer(n1 * n2),
            _ => unimplemented!(),
        }
    }
}

impl From<i64> for Number {
    fn from(n: i64) -> Number {
        Number::Integer(BigInt::from(n))
    }
}

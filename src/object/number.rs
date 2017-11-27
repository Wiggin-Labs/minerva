use parser::Token;
use num::{BigInt, BigRational, One, Zero};

use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Clone, PartialEq, PartialOrd, is_enum_variant)]
pub enum Number {
    Integer(BigInt),
    Rational(BigRational),
    Real(f64),
    Complex(Box<Complex>),
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::Number::*;
        match self {
            Integer(n) => write!(f, "{}", n),
            Rational(n) => write!(f, "{}", n),
            Real(n) => write!(f, "{}", n),
            Complex(n) => write!(f, "{}", n),
        }
    }
}

impl Number {
    pub fn from_token(t: &Token) -> Self {
        match t {
            Token::Integer(s) => Number::Integer(s.parse().unwrap()),
            Token::Rational(s) => unimplemented!(),
            Token::Real(s) => unimplemented!(),
            Token::ComplexInt(s) => unimplemented!(),
            Token::ComplexRat(s) => unimplemented!(),
            Token::ComplexReal(s) => unimplemented!(),
            _ => panic!("compiler error"),
        }
    }

    pub fn zero() -> Self {
        Number::Integer(BigInt::zero())
    }

    pub fn one() -> Self {
        Number::Integer(BigInt::one())
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
            Rational(n) => Rational(-n),
            Real(n) => Real(-n),
            Complex(n) => Complex(Box::new(-*n)),
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Complex {
    real: Number,
    imaginary: Number,
}

impl Display for Complex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}+{}i", self.real, self.imaginary)
    }
}

impl Neg for Complex {
    type Output = Complex;

    fn neg(mut self) -> Complex {
        self.real = -self.real;
        self.imaginary = -self.imaginary;
        self
    }
}

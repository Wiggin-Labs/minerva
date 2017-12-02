use super::{Number, Object};
use Error;

use std::fmt::{self, Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq)]
pub enum Arity {
    /// Normal procedure
    Exactly(usize),
    /// Variadic procedure
    AtLeast(usize),
}

impl Arity {
    pub fn as_usize(&self) -> usize {
        match self {
            Arity::Exactly(n) => *n,
            Arity::AtLeast(n) => *n,
        }
    }

    pub fn is_variadic(&self) -> bool {
        match self {
            Arity::AtLeast(_) => true,
            _ => false,
        }
    }

    pub fn correct_number_of_args(&self, args: usize) -> bool {
        match self {
            Arity::Exactly(n) => args == *n,
            Arity::AtLeast(n) => args >= *n,
        }
    }
}

impl Display for Arity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Arity::Exactly(n) => if *n > 0 {
                write!(f, "{}", n)
            } else {
                Ok(())
            },
            Arity::AtLeast(n) => write!(f, ">={}", n),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Primitive {
    pub name: String,
    pub args: Arity,
}

impl Primitive {
    pub fn new(name: String, args: Arity) -> Self {
        Primitive {
            name,
            args,
        }
    }

    pub fn run(self, args: Object) -> Object {
        let len = args.length();
        if !self.args.correct_number_of_args(len.as_usize()) {
            return Object::Error(Error::WrongArgs);
        }

        match self.name.as_str() {
            "cons" => {
                let car = args.car();
                let cdr = args.cadr();
                Object::cons(car, cdr)
            }
            "car" => {
                args.caar()
            }
            "cdr" => {
                args.cdar()
            }
            "set-car!" => {
                let pair = args.car();
                let car = args.cadr();
                pair.set_car(car)
            }
            "set-cdr!" => {
                let pair = args.car();
                let cdr = args.cadr();
                pair.set_cdr(cdr)
            }
            "=" => {
                if args.is_null() {
                    return Object::Bool(true);
                }
                let n = match args.car() {
                    Object::Number(n) => n,
                    _ => return Object::Error(Error::NumberExpected),
                };
                let mut args = args.cdr();
                while !args.is_null() {
                    match args.car() {
                        Object::Number(m) => if n != m {
                            return Object::Bool(false);
                        }
                        _ => return Object::Error(Error::NumberExpected),
                    }
                    args = args.cdr();
                }
                Object::Bool(true)
            }
            "+" => {
                let mut sum = Number::zero();
                let mut args = args;
                while !args.is_null() {
                    match args.car() {
                        Object::Number(n) => sum = sum + n,
                        _ => return Object::Error(Error::NumberExpected),
                    }
                    args = args.cdr();
                }
                Object::Number(sum)
            }
            "-" => {
                if len.as_usize() == 1 {
                    if let Object::Number(n) = args.car() {
                        Object::Number(-n)
                    } else {
                        Object::Error(Error::NumberExpected)
                    }
                } else {
                    let mut sum = if let Object::Number(n) = args.car() {
                        n
                    } else {
                        return Object::Error(Error::NumberExpected);
                    };

                    let mut args = args.cdr();
                    while !args.is_null() {
                        match args.car() {
                            Object::Number(n) => sum = sum - n,
                            _ => return Object::Error(Error::NumberExpected),
                        }
                        args = args.cdr();
                    }
                    Object::Number(sum)
                }
            }
            "*" => {
                let mut prod = Number::one();
                let mut args = args;
                while !args.is_null() {
                    match args.car() {
                        Object::Number(n) => prod = prod * n,
                        _ => return Object::Error(Error::NumberExpected),
                    }
                    args = args.cdr();
                }
                Object::Number(prod)
            }
            "/" => Object::Void,
            _ => Object::Error(Error::UnboundVariable(self.name)),
        }
    }
}

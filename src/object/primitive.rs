use super::Object;

use Error;

use num::{BigInt, One, Zero};

#[derive(Clone, Debug, PartialEq)]
pub struct Primitive {
    pub name: String,
    // None means that the procedure is a variadic
    pub args: Option<usize>,
}

impl Primitive {
    pub fn new(name: String, args: Option<usize>) -> Self {
        Primitive {
            name,
            args,
        }
    }

    pub fn run(self, args: Object) -> Option<Object> {
        let len = args.length();
        if let Some(n) = self.args {
            if len != n {
                //TODO
                //return Some(Object::Error("Wrong number of arguments passed to procedure".to_string()));
            }
        }

        match self.name.as_str() {
            "cons" => {
                let car = args.car();
                let cdr = args.cadr();
                Some(Object::cons(car, cdr))
            }
            "car" => {
                let arg = args.car();
                Some(arg.car())
            }
            "cdr" => {
                let arg = args.car();
                Some(arg.cdr())
            }
            "=" => {
                if args.is_null() {
                    return Some(Object::Bool(true));
                }
                let n = match args.car() {
                    Object::Number(n) => n,
                    _ => return Some(Object::Error(Error::NumberExpected)),
                };
                let mut args = args.cdr();
                while !args.is_null() {
                    match args.car() {
                        Object::Number(m) => if n != m {
                            return Some(Object::Bool(false));
                        }
                        _ => return Some(Object::Error(Error::NumberExpected)),
                    }
                    args = args.cdr();
                }
                Some(Object::Bool(true))
            }
            "+" => {
                let mut sum = BigInt::zero();
                let mut args = args;
                while !args.is_null() {
                    match args.car() {
                        Object::Number(n) => sum = sum + n,
                        _ => return Some(Object::Error(Error::NumberExpected)),
                    }
                    args = args.cdr();
                }
                Some(Object::Number(sum))
            }
            "-" => {
                if len < 1 {
                    // TODO
                    panic!("TODO");
                } else if len == 1 {
                    if let Object::Number(n) = args.car() {
                        Some(Object::Number(-n))
                    } else {
                        Some(Object::Error(Error::NumberExpected))
                    }
                } else {
                    let mut sum = args.car().unwrap_number();
                    let mut args = args.cdr();
                    while !args.is_null() {
                        match args.car() {
                            Object::Number(n) => sum = sum - n,
                            _ => return Some(Object::Error(Error::NumberExpected)),
                        }
                        args = args.cdr();
                    }
                    Some(Object::Number(sum))
                }
            }
            "*" => {
                let mut prod = BigInt::one();
                let mut args = args;
                while !args.is_null() {
                    match args.car() {
                        Object::Number(n) => prod = prod * n,
                        _ => return Some(Object::Error(Error::NumberExpected)),
                    }
                    args = args.cdr();
                }
                Some(Object::Number(prod))
            }
            "/" => None,
            _ => Some(Object::Error(Error::UnboundVariable(self.name))),
        }
    }
}

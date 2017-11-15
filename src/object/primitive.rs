use super::Object;

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
                return Some(Object::Error("Wrong number of arguments passed to procedure".to_string()));
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
            "+" => {
                let mut sum = 0;
                let mut args = args;
                while args != Object::Nil {
                    match args.car() {
                        Object::Number(n) => sum += n,
                        _ => return Some(Object::Error("NUMBER expected".to_string())),
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
                        Some(Object::Error("NUMBER expected".to_string()))
                    }
                } else {
                    let mut sum = args.car().unwrap_number();
                    let mut args = args.cdr();
                    while args != Object::Nil {
                        match args.car() {
                            Object::Number(n) => sum -= n,
                            _ => return Some(Object::Error("NUMBER expected".to_string())),
                        }
                        args = args.cdr();
                    }
                    Some(Object::Number(sum))
                }
            }
            "*" => {
                let mut prod = 1;
                let mut args = args;
                while args != Object::Nil {
                    match args.car() {
                        Object::Number(n) => prod *= n,
                        _ => return Some(Object::Error("NUMBER expected".to_string())),
                    }
                    args = args.cdr();
                }
                Some(Object::Number(prod))
            }
            "/" => None,
            _ => Some(Object::Error(format!("Unbound variable {}", self.name))),
        }
    }
}

use string_interner::{get_value, Symbol};
use vm::Value;

use std::fmt;

#[derive(Derivative, Debug, PartialEq)]
#[derivative(Clone(bound=""))]
pub enum IR<T> {
    Label(Symbol),
    Return(Symbol),
    Goto(Symbol),
    GotoIf(Symbol, Symbol),
    GotoIfNot(Symbol, Symbol),
    Phi(Symbol, Symbol, Symbol),
    Define(Symbol, Symbol, Symbol),
    Primitive(Symbol, Value<T>),
    Lookup(Symbol, Symbol),
    Copy(Symbol, Symbol),
    Param(Symbol),
    Call(Symbol, Symbol, usize),
    Fn(Symbol, Vec<Symbol>, Vec<IR<T>>),
}

impl<T> fmt::Display for IR<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IR::Primitive(s, v) => write!(f, "PRIMITIVE {}, {}", get_value(*s).unwrap(), v),
            IR::Define(t, s1, s2) => write!(f, "{}, DEFINE {}, {}", get_value(*t).unwrap(), get_value(*s1).unwrap(), get_value(*s2).unwrap()),
            IR::Lookup(t, s) => write!(f, "{}, LOOKUP {}", get_value(*t).unwrap(), get_value(*s).unwrap()),
            IR::Copy(t, s) => write!(f, "COPY {}, {}", get_value(*t).unwrap(), get_value(*s).unwrap()),
            IR::Param(s) => write!(f, "PARAM {}", get_value(*s).unwrap()),
            IR::Call(s, proc, args) => {
                write!(f, "{} CALL {}, {}", get_value(*s).unwrap(), get_value(*proc).unwrap(), args)
                //for arg in args {
                //    write!(f, "{}, ", get_value(*arg).unwrap())?;
                //}
                //write!(f, ")")
            }
            IR::Fn(s, args, ir) => {
                write!(f, "{}(", get_value(*s).unwrap())?;
                for arg in args {
                    write!(f, "{}, ", get_value(*arg).unwrap())?;
                }
                writeln!(f, ")")?;
                for i in ir {
                    writeln!(f, "\t{}", i)?;
                }
                Ok(())
            }
            IR::Phi(s1, s2, s3) => write!(f, "{} PHI {}, {}", get_value(*s1).unwrap(), get_value(*s2).unwrap(), get_value(*s3).unwrap()),
            IR::Label(s) => write!(f, "{}:", get_value(*s).unwrap()),
            IR::Goto(s) => write!(f, "GOTO {}", get_value(*s).unwrap()),
            IR::GotoIf(s1, s2) => write!(f, "GOTOIF {}, {}", get_value(*s1).unwrap(), get_value(*s2).unwrap()),
            IR::GotoIfNot(s1, s2) => write!(f, "GOTOIFNOT {}, {}", get_value(*s1).unwrap(), get_value(*s2).unwrap()),
            IR::Return(s) => write!(f, "RETURN {}", get_value(*s).unwrap()),
        }
    }
}

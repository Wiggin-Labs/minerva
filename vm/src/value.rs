use {Environment, Operation};

use string_interner::Sym;

use std::{cmp, ops};

/// Primitive values
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    #[doc(hidden)]
    /// Used in garbage collection for marking moved pairs
    BrokenHeart,
    Void,
    /// The empty list
    Nil,
    Label(usize),
    /// Integers
    Integer(i64),
    /// Booleans
    Bool(bool),
    /// Strings
    String(Box<String>),
    /// Symbols
    Symbol(Sym),
    Environment(Environment),
    Lambda(Box<Lambda>),
    // A pair is an index into ours cars and cdrs arrays
    /// Pairs
    Pair(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lambda {
    pub(crate) code: Vec<Operation>,
    pub(crate) consts: Vec<Value>,
    pub(crate) environment: Environment,
}

impl Lambda {
    pub fn new(code: Vec<Operation>, consts: Vec<Value>, environment: Environment) -> Self {
        Lambda {
            code,
            consts,
            environment,
        }
    }

    pub fn set_env(&mut self, env: Environment) {
        self.environment = env;
    }
}

impl cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use Value::*;
        match (self, other) {
            (Integer(a), Integer(b)) => Some(a.cmp(b)),
            _ => None
        }
    }
}

impl<'a> ops::Add for &'a Value {
    type Output = Value;
    fn add(self, other: &'a Value) -> Value {
        use Value::*;
        match (self, other) {
            (Integer(a1), Integer(a2)) => Integer(a1 + a2),
            _ => unimplemented!(),
        }
    }
}

impl<'a> ops::Sub for &'a Value {
    type Output = Value;
    fn sub(self, other: &'a Value) -> Value {
        use Value::*;
        match (self, other) {
            (Integer(a1), Integer(a2)) => Integer(a1 - a2),
            _ => unimplemented!(),
        }
    }
}

impl<'a> ops::Mul for &'a Value {
    type Output = Value;
    fn mul(self, other: &'a Value) -> Value {
        use Value::*;
        match (self, other) {
            (Integer(a1), Integer(a2)) => Integer(a1 * a2),
            _ => unimplemented!(),
        }
    }
}

impl Value {
    pub fn into_label(self) -> usize {
        match self {
            Value::Label(l) => l,
            _ => unreachable!(),
        }
    }

    pub fn symbol_to_string(self) -> Sym {
        match self {
            Value::Symbol(s) => s,
            _ => unreachable!(),
        }
    }

    pub fn is_true(&self) -> bool {
        match self {
            Value::Void => false,
            Value::Bool(false) => false,
            _ => true,
        }
    }

    pub fn pair_pointer(&self) -> usize {
        match self {
            Value::Pair(p) => *p,
            _ => unreachable!(),
        }
    }

    pub fn unwrap_lambda(self) -> Box<Lambda> {
        match self {
            Value::Lambda(l) => l,
            _ => panic!("Expected Lambda"),
        }
    }

    pub fn unwrap_symbol(&self) -> Sym {
        match self {
            Value::Symbol(s) => *s,
            _ => panic!("Expected Symbol"),
        }
    }

    pub fn unwrap_string(self) -> Box<String> {
        match self {
            Value::String(s) => s,
            _ => panic!("Expected String"),
        }
    }
}

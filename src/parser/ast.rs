use vm::Value;

use string_interner::Symbol;

#[derive(Debug)]
pub enum Ast<T> {
    Define {
        name: Symbol,
        value: Box<Ast<T>>,
    },
    Lambda {
        args: Vec<Symbol>,
        body: Vec<Ast<T>>,
    },
    If {
        predicate: Box<Ast<T>>,
        consequent: Box<Ast<T>>,
        alternative: Box<Ast<T>>,
    },
    Begin(Vec<Ast<T>>),
    Apply(Vec<Ast<T>>),
    Ident(Symbol),
    Primitive(Value<T>),
}

impl<T> Ast<T> {
    pub fn unwrap_define(self) -> (Symbol, Self) {
        match self {
            Ast::Define { name, value } => (name, *value),
            _ => unreachable!(),
        }
    }

    pub fn unwrap_if(self) -> (Self, Self, Self) {
        match self {
            Ast::If { predicate, consequent, alternative } =>
                (*predicate, *consequent, *alternative),
            _ => unreachable!(),
        }
    }

    pub fn unwrap_lambda(self) -> (Vec<Symbol>, Vec<Self>) {
        match self {
            Ast::Lambda { args, body } => (args, body),
            _ => unreachable!(),
        }
    }

    pub fn unwrap_begin(self) -> Vec<Self> {
        match self {
            Ast::Begin(b) => b,
            _ => unreachable!(),
        }
    }
}

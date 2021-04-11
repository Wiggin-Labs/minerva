use vm::Value;

use string_interner::Symbol;

#[derive(Debug)]
pub enum Ast {
    Define {
        name: Symbol,
        value: Box<Ast>,
    },
    Lambda {
        args: Vec<Symbol>,
        body: Vec<Ast>,
    },
    If {
        predicate: Box<Ast>,
        consequent: Box<Ast>,
        alternative: Box<Ast>,
    },
    Begin(Vec<Ast>),
    Apply(Vec<Ast>),
    Ident(Symbol),
    Primitive(Value),
}

impl Ast {
    pub fn unwrap_define(self) -> (Symbol, Ast) {
        match self {
            Ast::Define { name, value } => (name, *value),
            _ => unreachable!(),
        }
    }

    pub fn unwrap_if(self) -> (Ast, Ast, Ast) {
        match self {
            Ast::If { predicate, consequent, alternative } =>
                (*predicate, *consequent, *alternative),
            _ => unreachable!(),
        }
    }

    pub fn unwrap_lambda(self) -> (Vec<Symbol>, Vec<Ast>) {
        match self {
            Ast::Lambda { args, body } => (args, body),
            _ => unreachable!(),
        }
    }

    pub fn unwrap_begin(self) -> Vec<Ast> {
        match self {
            Ast::Begin(b) => b,
            _ => unreachable!(),
        }
    }
}

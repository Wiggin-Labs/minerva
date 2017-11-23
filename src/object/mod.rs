mod number;
mod primitive;

pub use self::number::Number;
pub use self::primitive::{Arity, Primitive};

use {Environment, Error, eval};

use std::cell::RefCell;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct Pair {
    pub car: Object,
    pub cdr: Object,
}

impl Pair {
    pub fn new(car: Object, cdr: Object) -> Self {
        Pair {
            car,
            cdr,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Lambda {
    pub parameters: Object,
    pub body: Object,
    pub env: Environment,
}

impl Lambda {
    pub fn new(parameters: Object, body: Object, env: Environment) -> Self {
        Lambda {
            parameters,
            body,
            env,
        }
    }
}

#[derive(Clone, Debug, PartialEq, is_enum_variant)]
pub enum Object {
    Void,
    #[is_enum_variant(name="is_null")]
    Nil,
    Bool(bool),
    Number(Number),
    String(String),
    Symbol(String),
    Pair(Rc<RefCell<Pair>>),
    #[is_enum_variant(skip)]
    Lambda(Rc<Lambda>),
    Primitive(Primitive),
    Error(Error),
}

impl Object {
    // -----------------------------------
    // Interpreter procedures
    // -----------------------------------

    pub(crate) fn is_self_evaluating(&self) -> bool {
        match self {
            Object::Nil | Object::Bool(_) | Object::Number(_) | Object::String(_) => true,
            _ => false,
        }
    }

    pub(crate) fn is_variable(&self) -> bool {
        match self {
            Object::Symbol(_) => true,
            _ => false,
        }
    }

    pub(crate) fn lookup_variable_value(self, env: &Environment) -> Object {
        let var = self.symbol_value();
        if let Some(value) = env.lookup_variable_value(&var) {
            value
        } else {
            Object::Error(Error::UnboundVariable(var))
        }
    }

    pub(crate) fn is_quoted(&self) -> bool {
        self.is_tagged_list("quote".to_string())
    }

    fn is_tagged_list(&self, tag: String) -> bool {
        match self {
            Object::Pair(pair) => pair.borrow().car == Object::Symbol(tag),
            _ => false,
        }
    }

    pub(crate) fn text_of_quotation(self) -> Object {
        self.cadr()
    }

    pub(crate) fn is_assignment(&self) -> bool {
        self.is_tagged_list("set!".to_string())
    }

    pub(crate) fn eval_assignment(self, env: &Environment) -> Object {
        let var = self.assignment_variable().symbol_value();
        let val = eval(self.assignment_value(), env);
        env.set_variable_value(var, val)
    }

    fn assignment_variable(&self) -> Object {
        self.cadr()
    }

    fn assignment_value(&self) -> Object {
        self.caddr()
    }

    pub(crate) fn is_definition(&self) -> bool {
        self.is_tagged_list("define".to_string())
    }

    pub(crate) fn eval_definition(self, env: &Environment) -> Object {
        let var = self.definition_variable().symbol_value();
        let val = eval(self.definition_value(), env);
        env.define_variable(var, val);
        Object::Void
    }

    fn definition_variable(&self) -> Object {
        let cadr = self.cadr();
        if cadr.is_symbol() {
            cadr
        } else {
            self.caadr()
        }
    }

    fn definition_value(&self) -> Object {
        if self.cadr().is_symbol() {
            self.caddr()
        } else {
            Object::make_lambda(self.cdadr(), self.cddr())
        }
    }

    fn make_lambda(parameters: Object, body: Object) -> Object {
        Object::cons(Object::Symbol("lambda".to_string()),
                   Object::cons(parameters, body))
    }

    pub(crate) fn is_if(&self) -> bool {
        self.is_tagged_list("if".to_string())
    }

    pub(crate) fn eval_if(self, env: &Environment) -> Object {
        if eval(self.if_predicate(), env).is_true() {
            eval(self.if_consequent(), env)
        } else {
            eval(self.if_alternative(), env)
        }
    }

    fn if_predicate(&self) -> Object {
        self.cadr()
    }

    fn if_consequent(&self) -> Object {
        self.caddr()
    }

    fn if_alternative(&self) -> Object {
        if !self.cdddr().is_null() {
            self.cadddr()
        } else {
            Object::Bool(false)
        }
    }

    pub(crate) fn is_lambda(&self) -> bool {
        self.is_tagged_list("lambda".to_string())
    }

    pub(crate) fn make_procedure(self, env: &Environment) -> Object {
        let parameters = self.lambda_parameters();
        let body = self.lambda_body();
        let env = env.extend();
        let procedure = Lambda::new(parameters, body, env);
        Object::cons(Object::Symbol("procedure".to_string()),
                     Object::Lambda(Rc::new(procedure)))
    }

    fn lambda_parameters(&self) -> Object {
        self.cadr()
    }

    fn lambda_body(&self) -> Object {
        self.cddr()
    }

    pub(crate) fn is_begin(&self) -> bool {
        self.is_tagged_list("begin".to_string())
    }

    pub(crate) fn eval_sequence(self, env: &Environment) -> Object {
        if self.is_last_exp() {
            eval(self.first_exp(), env)
        } else {
            eval(self.first_exp(), env);
            self.rest_exps().eval_sequence(env)
        }
    }

    fn is_last_exp(&self) -> bool {
        self.cdr().is_null()
    }

    fn first_exp(&self) -> Object {
        self.car()
    }

    fn rest_exps(&self) -> Object {
        self.cdr()
    }

    pub(crate) fn is_cond(&self) -> bool {
        self.is_tagged_list("cond".to_string())
    }

    pub(crate) fn cond_to_if(&self) -> Object {
        self.cond_clauses().expand_clauses()
    }

    fn cond_clauses(&self) -> Object {
        self.cdr()
    }

    fn expand_clauses(&self) -> Object {
        if self.is_null() {
            Object::Bool(false)
        } else {
            let first = self.car();
            let rest = self.cdr();
            if first.is_cond_else_clause() {
                if rest.is_null() {
                    first.cond_actions().sequence_to_exp()
                } else {
                    Object::Error(Error::ElseNotLast)
                }
            } else {
                let predicate = first.cond_predicate();
                let actions = first.cond_actions().sequence_to_exp();
                let rest = rest.expand_clauses();
                if rest.is_error() {
                    rest
                } else {
                    Object::make_if(predicate, actions, rest)
                }
            }
        }
    }

    fn make_if(predicate: Object, consequent: Object, alternative: Object) -> Object {
        Object::cons(Object::Symbol("if".to_string()),
                   Object::cons(predicate,
                              Object::cons(consequent,
                                         Object::cons(alternative, Object::Nil))))
    }

    fn is_cond_else_clause(&self) -> bool {
        self.cond_predicate() == Object::Symbol("else".to_string())
    }

    fn cond_actions(&self) -> Object {
        self.cdr()
    }

    fn sequence_to_exp(self) -> Object {
        if self.is_null() {
            self
        } else if self.is_last_exp() {
            self.first_exp()
        } else {
            self.make_begin()
        }
    }

    fn make_begin(self) -> Object {
        Object::cons(Object::Symbol("begin".to_string()), self)
    }

    fn cond_predicate(&self) -> Object {
        self.car()
    }

    pub(crate) fn is_application(&self) -> bool {
        self.is_pair()
    }

    pub(crate) fn operator(&self) -> Object {
        self.car()
    }

    pub(crate) fn operands(&self) -> Object {
        self.cdr()
    }

    pub(crate) fn list_of_values(self, env: &Environment) -> Object {
        if self.has_no_operands() {
            Object::Nil
        } else {
            let car = eval(self.first_operand(), env);
            if car.is_error() {
                return car;
            }
            let cdr = self.rest_operands().list_of_values(env);
            if cdr.is_error() {
                return cdr;
            }
            Object::cons(car, cdr)
        }
    }

    fn has_no_operands(&self) -> bool {
        self.is_null()
    }

    fn first_operand(&self) -> Object {
        self.car()
    }

    fn rest_operands(&self) -> Object {
        self.cdr()
    }

    pub(crate) fn is_primitive_procedure(&self) -> bool {
        self.is_tagged_list("primitive".to_string())
    }

    pub(crate) fn apply_primitive_procedure(self, args: Object) -> Object {
        let procedure = self.primitive_implementation();
        let primitive = match procedure {
            Object::Primitive(p) => p,
            _ => panic!("compiler error in apply_primitive_procedure"),
        };
        primitive.run(args)
    }

    fn primitive_implementation(&self) -> Object {
        self.cadr()
    }

    pub(crate) fn is_compound_procedure(&self) -> bool {
        self.is_tagged_list("procedure".to_string())
    }

    pub(crate) fn procedure_env(&self) -> Environment {
        match self.cdr() {
            Object::Lambda(procedure) => procedure.env.procedure_local(),
            _ => panic!("compiler error in procedure_env"),
        }
    }

    pub(crate) fn procedure_parameters(&self) -> Object {
        match self.cdr() {
            Object::Lambda(procedure) => procedure.parameters.clone(),
            _ => panic!("compiler error in procedure_parameters"),
        }
    }

    pub(crate) fn procedure_body(&self) -> Object {
        match self.cdr() {
            Object::Lambda(procedure) => procedure.body.clone(),
            _ => panic!("compiler error in procedure_body"),
        }
    }

    pub(crate) fn symbol_value(self) -> String {
        match self {
            Object::Symbol(s) => s,
            _ => panic!("compiler error in symbol_value"),
        }
    }

    // -----------------------------------
    // Primitive procedures
    // -----------------------------------

    pub fn cons(car: Object, cdr: Object) -> Object {
        let pair = Pair {
            car,
            cdr,
        };
        Object::Pair(Rc::new(RefCell::new(pair)))
    }

    pub fn car(&self) -> Object {
        match self {
            Object::Pair(pair) => pair.borrow().car.clone(),
            _ => Object::Error(Error::PairExpected),
        }
    }

    pub fn caar(&self) -> Object {
        self.car().car()
    }

    pub fn caaar(&self) -> Object {
        self.caar().car()
    }

    pub fn caaaar(&self) -> Object {
        self.caaar().car()
    }

    pub fn cdr(&self) -> Object {
        match self {
            Object::Pair(pair) => pair.borrow().cdr.clone(),
            _ => Object::Error(Error::PairExpected),
        }
    }

    pub fn cddr(&self) -> Object {
        self.cdr().cdr()
    }

    pub fn cdddr(&self) -> Object {
        self.cddr().cdr()
    }

    pub fn cddddr(&self) -> Object {
        self.cdddr().cdr()
    }

    pub fn cadr(&self) -> Object {
        self.cdr().car()
    }

    pub fn caadr(&self) -> Object {
        self.cadr().car()
    }

    pub fn caaadr(&self) -> Object {
        self.caadr().car()
    }

    pub fn caddr(&self) -> Object {
        self.cddr().car()
    }

    pub fn cdadr(&self) -> Object {
        self.cadr().cdr()
    }

    pub fn cadddr(&self) -> Object {
        self.cdddr().car()
    }

    pub fn push(&self, next: Object) -> Object {
        if self.is_null() {
            Object::cons(next, Object::Nil)
        } else {
            Object::cons(self.car(), self.cdr().push(next))
        }
    }

    pub fn length(&self) -> usize {
        let mut list = self.clone();
        let mut len = 0;
        while !list.is_null() {
            len += 1;
            list = list.cdr();
        }
        len
    }

    pub fn is_true(&self) -> bool {
        match *self {
            Object::Bool(b) => b,
            _ => true,
        }
    }

    pub fn is_false(&self) -> bool {
        match *self {
            Object::Bool(b) => !b,
            _ => false,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Object::Nil => write!(f, "()"),
            Object::Void => Ok(()),
            Object::Bool(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Object::Number(n) => write!(f, "{}", n),
            Object::String(s) => write!(f, "\"{}\"", s),
            Object::Symbol(s) => write!(f, "{}", s),
            Object::Pair(p) => {
                let p = p.borrow();
                write!(f, "({}", p.car)?;
                let mut pair = p.cdr.clone();
                while !pair.is_null() {
                    match pair {
                        Object::Pair(p) => {
                            let p = p.borrow();
                            write!(f, " {}", p.car)?;
                            pair = p.cdr.clone();
                        },
                        _ => return write!(f, " . {})", pair),
                    }
                }
                write!(f, ")")
            }
            // TODO: print proc name and number of args
            Object::Lambda(_l) => write!(f, "#<procedure TODO>"),
            Object::Primitive(l) => write!(f, "#<procedure {} {} args>", l.name, l.args),
            Object::Error(e) => write!(f, "ERROR: {}", e),
        }
    }
}

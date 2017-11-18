mod primitive;

pub use self::primitive::Primitive;

use {Environment, Error, eval};

use num::BigInt;

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

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Nil,
    Bool(bool),
    Number(BigInt),
    String(String),
    Symbol(String),
    Pair(Rc<RefCell<Pair>>),
    Lambda(Rc<Lambda>),
    Primitive(Primitive),
    Error(Error),
}

impl Object {
    pub fn make_procedure(self, env: &Environment) -> Option<Object> {
        let parameters = self.lambda_parameters();
        let body = self.lambda_body();
        let env = env.extend();
        let procedure = Lambda::new(parameters, body, env);
        Some(Object::cons(Object::Symbol("procedure".to_string()),
                          Object::Lambda(Rc::new(procedure))))
    }

    pub fn is_compound_procedure(&self) -> bool {
        self.is_tagged_list("procedure".to_string())
    }

    pub fn procedure_parameters(&self) -> Object {
        match self.cdr() {
            Object::Lambda(procedure) => procedure.parameters.clone(),
            _ => panic!("compiler error in procedure_parameters"),
        }
    }

    pub fn procedure_body(&self) -> Object {
        match self.cdr() {
            Object::Lambda(procedure) => procedure.body.clone(),
            _ => panic!("compiler error in procedure_body"),
        }
    }

    pub fn procedure_env(&self) -> Environment {
        match self.cdr() {
            Object::Lambda(procedure) => procedure.env.procedure_local(),
            _ => panic!("compiler error in procedure_env"),
        }
    }

    pub fn eval_sequence(self, env: &Environment) -> Option<Object> {
        if self.is_last_exp() {
            eval(self.first_exp(), env)
        } else {
            eval(self.first_exp(), env);
            self.rest_exps().eval_sequence(env)
        }
    }

    pub fn list_of_values(self, env: &Environment) -> Object {
        if self.has_no_operands() {
            Object::Nil
        } else {
            Object::cons(eval(self.first_operand(), env).unwrap(),
                         self.rest_operands().list_of_values(env))
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

    pub fn eval_if(self, env: &Environment) -> Option<Object> {
        if eval(self.if_predicate(), env).unwrap().is_true() {
            eval(self.if_consequent(), env)
        } else {
            eval(self.if_alternative(), env)
        }
    }

    pub fn symbol_value(self) -> String {
        match self {
            Object::Symbol(s) => s,
            _ => panic!("compiler error in symbol_value"),
        }
    }

    pub fn eval_assignment(self, env: &Environment) -> Option<Object> {
        let var = self.assignment_variable().symbol_value();
        let val = eval(self.assignment_value(), env).unwrap();
        env.set_variable_value(var, val)
    }

    pub fn eval_definition(self, env: &Environment) {
        let var = self.definition_variable().symbol_value();
        let val = eval(self.definition_value(), env).unwrap();
        env.define_variable(var, val);
    }

    pub fn is_self_evaluating(&self) -> bool {
        match self {
            Object::Number(_) | Object::String(_) => true,
            _ => false,
        }
    }

    pub fn is_variable(&self) -> bool {
        match self {
            Object::Symbol(_) => true,
            _ => false,
        }
    }

    pub fn is_quoted(&self) -> bool {
        self.is_tagged_list("quote".to_string())
    }

    pub fn text_of_quotation(self) -> Option<Object> {
        Some(self.cadr())
    }

    pub fn is_tagged_list(&self, tag: String) -> bool {
        match self {
            Object::Pair(pair) => pair.borrow().car == Object::Symbol(tag),
            _ => false,
        }
    }

    pub fn is_assignment(&self) -> bool {
        self.is_tagged_list("set!".to_string())
    }

    pub fn assignment_variable(&self) -> Object {
        self.cadr()
    }

    pub fn assignment_value(&self) -> Object {
        self.caddr()
    }

    pub fn is_definition(&self) -> bool {
        self.is_tagged_list("define".to_string())
    }

    pub fn car(&self) -> Object {
        match self {
            Object::Pair(pair) => pair.borrow().car.clone(),
            _ => Object::Error(Error::PairExpected),
        }
    }

    pub fn cdr(&self) -> Object {
        match self {
            Object::Pair(pair) => pair.borrow().cdr.clone(),
            _ => Object::Error(Error::PairExpected),
        }
    }

    pub fn cadr(&self) -> Object {
        self.cdr().car()
    }

    pub fn caddr(&self) -> Object {
        self.cddr().car()
    }

    pub fn caadr(&self) -> Object {
        self.cadr().car()
    }

    pub fn cddr(&self) -> Object {
        self.cdr().cdr()
    }

    pub fn cdadr(&self) -> Object {
        self.cadr().cdr()
    }

    pub fn is_symbol(&self) -> bool {
        match self {
            Object::Symbol(_) => true,
            _ => false,
        }
    }

    pub fn definition_variable(&self) -> Object {
        let cadr = self.cadr();
        if cadr.is_symbol() {
            cadr
        } else {
            self.caadr()
        }
    }

    pub fn definition_value(&self) -> Object {
        if self.cadr().is_symbol() {
            self.caddr()
        } else {
            Object::make_lambda(self.cdadr(), self.cddr())
        }
    }

    pub fn is_lambda(&self) -> bool {
        self.is_tagged_list("lambda".to_string())
    }

    pub fn lambda_parameters(&self) -> Object {
        self.cadr()
    }

    pub fn lambda_body(&self) -> Object {
        // TODO: maybe caddr here?
        self.cddr()
    }

    pub fn cons(car: Object, cdr: Object) -> Object {
        let pair = Pair {
            car,
            cdr,
        };
        Object::Pair(Rc::new(RefCell::new(pair)))
    }

    pub fn make_lambda(parameters: Object, body: Object) -> Object {
        Object::cons(Object::Symbol("lambda".to_string()),
                   Object::cons(parameters, body))
    }

    pub fn is_if(&self) -> bool {
        self.is_tagged_list("if".to_string())
    }

    pub fn if_predicate(&self) -> Object {
        self.cadr()
    }

    pub fn if_consequent(&self) -> Object {
        self.caddr()
    }

    pub fn cdddr(&self) -> Object {
        self.cddr().cdr()
    }

    pub fn cadddr(&self) -> Object {
        self.cdddr().car()
    }

    pub fn if_alternative(&self) -> Object {
        if !self.cdddr().is_null() {
            self.cadddr()
        } else {
            Object::Bool(false)
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Object::Nil => true,
            _ => false,
        }
    }

    pub fn make_if(predicate: Object, consequent: Object, alternative: Object) -> Object {
        Object::cons(Object::Symbol("if".to_string()),
                   Object::cons(predicate,
                              Object::cons(consequent,
                                         Object::cons(alternative, Object::Nil))))
    }

    pub fn is_begin(&self) -> bool {
        self.is_tagged_list("begin".to_string())
    }

    pub fn begin_actions(&self) -> Object {
        self.cdr()
    }

    pub fn is_last_exp(&self) -> bool {
        self.cdr().is_null()
    }

    pub fn first_exp(&self) -> Object {
        self.car()
    }

    pub fn rest_exps(&self) -> Object {
        self.cdr()
    }

    pub fn sequence_to_exp(self) -> Object {
        if self.is_null() {
            self
        } else if self.is_last_exp() {
            self.first_exp()
        } else {
            self.make_begin()
        }
    }

    pub fn make_begin(self) -> Object {
        Object::cons(Object::Symbol("begin".to_string()), self)
    }

    pub fn is_application(&self) -> bool {
        self.is_pair()
    }

    pub fn is_pair(&self) -> bool {
        match self {
            Object::Pair(_) => true,
            _ => false,
        }
    }

    pub fn operator(&self) -> Object {
        self.car()
    }

    pub fn operands(&self) -> Object {
        self.cdr()
    }

    pub fn has_no_operands(&self) -> bool {
        self.is_null()
    }

    pub fn first_operand(&self) -> Object {
        self.car()
    }

    pub fn rest_operands(&self) -> Object {
        self.cdr()
    }

    pub fn is_cond(&self) -> bool {
        self.is_tagged_list("cond".to_string())
    }

    pub fn cond_clauses(&self) -> Object {
        self.cdr()
    }

    pub fn is_cond_else_clause(&self) -> bool {
        self.cond_predicate() == Object::Symbol("else".to_string())
    }

    pub fn cond_predicate(&self) -> Object {
        self.car()
    }

    pub fn cond_actions(&self) -> Object {
        self.cdr()
    }

    pub fn cond_to_if(&self) -> Object {
        self.cond_clauses().expand_clauses()
    }

    pub fn expand_clauses(&self) -> Object {
        if self.is_null() {
            Object::Bool(false)
        } else {
            let first = self.car();
            let rest = self.cdr();
            if first.is_cond_else_clause() {
                if rest.is_null() {
                    first.cond_actions().sequence_to_exp()
                } else {
                    panic!("Else clause isn't last: {:?}", self);
                }
            } else {
                Object::make_if(first.cond_predicate(),
                              first.cond_actions().sequence_to_exp(),
                              rest.expand_clauses())
            }
        }
    }

    pub fn lookup_variable_value(self, env: &Environment) -> Option<Object> {
        env.lookup_variable_value(&self.symbol_value())
    }

    pub fn is_primitive_procedure(&self) -> bool {
        self.is_tagged_list("primitive".to_string())
    }

    pub fn primitive_implementation(&self) -> Object {
        self.cadr()
    }

    pub fn unwrap_number(self) -> BigInt {
        match self {
            Object::Number(n) => n,
            _ => panic!("compiler error in unwrap_number"),
        }
    }

    pub fn apply_primitive_procedure(self, args: Object) -> Option<Object> {
        let procedure = self.primitive_implementation();
        let primitive = match procedure {
            Object::Primitive(p) => p,
            _ => panic!("compiler error in apply_primitive_procedure"),
        };
        primitive.run(args)
    }

    pub fn push(&self, next: Object) -> Object {
        if self.is_null() {
            Object::cons(next, Object::Nil)
        } else {
            Object::cons(self.car(), self.cdr().push(next))
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Object::Nil => write!(f, "()"),
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
            Object::Primitive(l) => {
                write!(f, "#<procedure {}", l.name)?;
                if let Some(n) = l.args {
                    write!(f, " {} args", n)?;
                }
                write!(f, ">")
            }
            Object::Error(e) => write!(f, "ERROR: {}", e),
        }
    }
}

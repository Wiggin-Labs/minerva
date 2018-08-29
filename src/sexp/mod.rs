mod number;
mod pair;
mod primitive;

pub use self::number::{ComplexExact, ComplexFloating, Number};
pub use self::pair::Pair;
pub use self::primitive::{Arity, Primitive};

use {Environment, Error, eval};

use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct Lambda {
    pub parameters: Vec<String>,
    pub arity: Arity,
    pub body: Sexp,
    pub env: Environment,
}

impl Lambda {
    pub fn new(mut parameters: Sexp, body: Sexp, env: Environment) -> Self {
        let mut params = Vec::new();
        let mut variadic = false;
        while !parameters.is_null() {
            if !parameters.is_pair() {
                params.push(parameters.symbol_value());
                variadic = true;
                break;
            }
            params.push(parameters.car().symbol_value());
            parameters = parameters.cdr();
        }

        let arity = if variadic {
            Arity::AtLeast(params.len())
        } else {
            Arity::Exactly(params.len())
        };

        Lambda {
            parameters: params,
            arity: arity,
            body: body,
            env: env,
        }
    }
}

#[derive(Clone, Debug, PartialEq, is_enum_variant)]
pub enum Sexp {
    Void,
    #[is_enum_variant(name="is_null")]
    Nil,
    Bool(bool),
    Number(Number),
    String(String),
    Symbol(String),
    Pair(Pair<Sexp>),
    #[is_enum_variant(skip)]
    Lambda(Rc<Lambda>),
    Primitive(Primitive),
    Error(Error),
}

impl Sexp {
    // -----------------------------------
    // Interpreter procedures
    // -----------------------------------

    pub(crate) fn is_self_evaluating(&self) -> bool {
        match self {
            Sexp::Nil | Sexp::Bool(_) | Sexp::Number(_) | Sexp::String(_) => true,
            _ => false,
        }
    }

    pub(crate) fn is_variable(&self) -> bool {
        match self {
            Sexp::Symbol(_) => true,
            _ => false,
        }
    }

    pub(crate) fn lookup_variable_value(self, env: &Environment) -> Sexp {
        let var = self.symbol_value();
        if let Some(value) = env.lookup_variable_value(&var) {
            value
        } else {
            Sexp::Error(Error::UnboundVariable(var))
        }
    }

    pub(crate) fn is_quoted(&self) -> bool {
        self.is_tagged_list("quote".to_string())
    }

    fn is_tagged_list(&self, tag: String) -> bool {
        match self {
            Sexp::Pair(pair) => pair.car() == Sexp::Symbol(tag),
            _ => false,
        }
    }

    pub(crate) fn text_of_quotation(self) -> Sexp {
        self.cadr()
    }

    pub(crate) fn is_assignment(&self) -> bool {
        self.is_tagged_list("set!".to_string())
    }

    pub(crate) fn eval_assignment(self, env: &Environment) -> Sexp {
        let var = self.assignment_variable().symbol_value();
        let val = eval(self.assignment_value(), env);
        env.set_variable_value(var, val)
    }

    fn assignment_variable(&self) -> Sexp {
        self.cadr()
    }

    fn assignment_value(&self) -> Sexp {
        self.caddr()
    }

    pub(crate) fn is_definition(&self) -> bool {
        self.is_tagged_list("define".to_string())
    }

    pub(crate) fn eval_definition(self, env: &Environment) -> Sexp {
        let var = self.definition_variable().symbol_value();
        let val = eval(self.definition_value(), env);
        env.define_variable(var, val);
        Sexp::Void
    }

    fn definition_variable(&self) -> Sexp {
        let cadr = self.cadr();
        if cadr.is_symbol() {
            cadr
        } else {
            self.caadr()
        }
    }

    fn definition_value(&self) -> Sexp {
        if self.cadr().is_symbol() {
            self.caddr()
        } else {
            Sexp::make_lambda(self.cdadr(), self.cddr())
        }
    }

    fn make_lambda(parameters: Sexp, body: Sexp) -> Sexp {
        Sexp::cons(Sexp::Symbol("lambda".to_string()),
                   Sexp::cons(parameters, body))
    }

    pub(crate) fn is_if(&self) -> bool {
        self.is_tagged_list("if".to_string())
    }

    pub(crate) fn eval_if(self, env: &Environment) -> Sexp {
        if eval(self.if_predicate(), env).is_true() {
            eval(self.if_consequent(), env)
        } else {
            eval(self.if_alternative(), env)
        }
    }

    fn if_predicate(&self) -> Sexp {
        self.cadr()
    }

    fn if_consequent(&self) -> Sexp {
        self.caddr()
    }

    fn if_alternative(&self) -> Sexp {
        if !self.cdddr().is_null() {
            self.cadddr()
        } else {
            Sexp::Bool(false)
        }
    }

    pub(crate) fn is_lambda(&self) -> bool {
        self.is_tagged_list("lambda".to_string())
    }

    pub(crate) fn make_procedure(self, env: &Environment) -> Self {
        let parameters = self.lambda_parameters();
        let body = self.lambda_body();
        let env = env.extend();
        let procedure = Lambda::new(parameters, body, env);
        Sexp::cons(Sexp::Symbol("procedure".to_string()),
                     Sexp::Lambda(Rc::new(procedure)))
    }

    fn lambda_parameters(&self) -> Self {
        self.cadr()
    }

    fn lambda_body(&self) -> Self {
        self.cddr()
    }

    pub(crate) fn is_begin(&self) -> bool {
        self.is_tagged_list("begin".to_string())
    }

    pub(crate) fn eval_sequence(self, env: &Environment) -> Self {
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

    fn first_exp(&self) -> Self {
        self.car()
    }

    fn rest_exps(&self) -> Self {
        self.cdr()
    }

    pub(crate) fn is_cond(&self) -> bool {
        self.is_tagged_list("cond".to_string())
    }

    pub(crate) fn cond_to_if(&self) -> Self {
        self.cond_clauses().expand_clauses()
    }

    fn cond_clauses(&self) -> Self {
        self.cdr()
    }

    fn expand_clauses(&self) -> Self {
        if self.is_null() {
            Sexp::Bool(false)
        } else {
            let first = self.car();
            let rest = self.cdr();
            if first.is_cond_else_clause() {
                if rest.is_null() {
                    first.cond_actions().sequence_to_exp()
                } else {
                    Sexp::Error(Error::ElseNotLast)
                }
            } else {
                let predicate = first.cond_predicate();
                let actions = first.cond_actions().sequence_to_exp();
                let rest = rest.expand_clauses();
                if rest.is_error() {
                    rest
                } else {
                    Sexp::make_if(predicate, actions, rest)
                }
            }
        }
    }

    fn make_if(predicate: Self, consequent: Self, alternative: Self) -> Self {
        Sexp::cons(Sexp::Symbol("if".to_string()),
                   Sexp::cons(predicate,
                              Sexp::cons(consequent,
                                         Sexp::cons(alternative, Sexp::Nil))))
    }

    fn is_cond_else_clause(&self) -> bool {
        self.cond_predicate() == Sexp::Symbol("else".to_string())
    }

    fn cond_actions(&self) -> Self {
        self.cdr()
    }

    fn sequence_to_exp(self) -> Self {
        if self.is_null() {
            self
        } else if self.is_last_exp() {
            self.first_exp()
        } else {
            self.make_begin()
        }
    }

    fn make_begin(self) -> Self {
        Sexp::cons(Sexp::Symbol("begin".to_string()), self)
    }

    fn cond_predicate(&self) -> Self {
        self.car()
    }

    pub(crate) fn is_application(&self) -> bool {
        self.is_pair()
    }

    pub(crate) fn operator(&self) -> Self {
        self.car()
    }

    pub(crate) fn operands(&self) -> Self {
        self.cdr()
    }

    pub(crate) fn list_of_values(self, env: &Environment) -> Self {
        if self.has_no_operands() {
            Sexp::Nil
        } else {
            let car = eval(self.first_operand(), env);
            if car.is_error() {
                return car;
            }
            let cdr = self.rest_operands().list_of_values(env);
            if cdr.is_error() {
                return cdr;
            }
            Sexp::cons(car, cdr)
        }
    }

    fn has_no_operands(&self) -> bool {
        self.is_null()
    }

    fn first_operand(&self) -> Self {
        self.car()
    }

    fn rest_operands(&self) -> Self {
        self.cdr()
    }

    pub(crate) fn is_primitive_procedure(&self) -> bool {
        self.is_tagged_list("primitive".to_string())
    }

    pub(crate) fn apply_primitive_procedure(self, args: Self) -> Self {
        let procedure = self.primitive_implementation();
        let primitive = match procedure {
            Sexp::Primitive(p) => p,
            _ => panic!("compiler error in apply_primitive_procedure"),
        };
        primitive.run(args)
    }

    fn primitive_implementation(&self) -> Self {
        self.cadr()
    }

    pub(crate) fn is_compound_procedure(&self) -> bool {
        self.is_tagged_list("procedure".to_string())
    }

    pub(crate) fn procedure_env(&self) -> Environment {
        match self.cdr() {
            Sexp::Lambda(procedure) => procedure.env.procedure_local(),
            _ => panic!("compiler error in procedure_env"),
        }
    }

    pub(crate) fn procedure_parameters(&self) -> Vec<String> {
        match self.cdr() {
            Sexp::Lambda(procedure) => procedure.parameters.clone(),
            _ => panic!("compiler error in procedure_parameters"),
        }
    }

    pub(crate) fn procedure_arity(&self) -> Arity {
        match self.cdr() {
            Sexp::Lambda(procedure) => procedure.arity,
            _ => panic!("compiler error in procedure_parameters"),
        }
    }

    pub(crate) fn procedure_body(&self) -> Self {
        match self.cdr() {
            Sexp::Lambda(procedure) => procedure.body.clone(),
            _ => panic!("compiler error in procedure_body"),
        }
    }

    pub(crate) fn symbol_value(self) -> String {
        match self {
            Sexp::Symbol(s) => s,
            _ => panic!("compiler error in symbol_value"),
        }
    }

    // -----------------------------------
    // Primitive procedures
    // -----------------------------------

    #[cfg_attr(feature="profile", flame)]
    pub fn cons(car: Self, cdr: Self) -> Self {
        Sexp::Pair(Pair::new(car, cdr))
    }

    #[cfg_attr(feature="profile", flame)]
    pub fn car(&self) -> Self {
        match self {
            Sexp::Pair(pair) => pair.car().clone(),
            _ => Sexp::Error(Error::PairExpected),
        }
    }

    pub fn caar(&self) -> Self {
        self.car().car()
    }

    pub fn caaar(&self) -> Self {
        self.caar().car()
    }

    pub fn caaaar(&self) -> Self {
        self.caaar().car()
    }

    #[cfg_attr(feature="profile", flame)]
    pub fn cdr(&self) -> Self {
        match self {
            Sexp::Pair(pair) => pair.cdr(),
            _ => Sexp::Error(Error::PairExpected),
        }
    }

    pub fn cddr(&self) -> Self {
        self.cdr().cdr()
    }

    pub fn cdddr(&self) -> Self {
        self.cddr().cdr()
    }

    pub fn cddddr(&self) -> Self {
        self.cdddr().cdr()
    }

    pub fn cadr(&self) -> Self {
        self.cdr().car()
    }

    pub fn caadr(&self) -> Self {
        self.cadr().car()
    }

    pub fn caaadr(&self) -> Self {
        self.caadr().car()
    }

    pub fn caddr(&self) -> Self {
        self.cddr().car()
    }

    pub fn cadddr(&self) -> Self {
        self.cdddr().car()
    }

    pub fn cdar(&self) -> Self {
        self.car().cdr()
    }

    pub fn cdadr(&self) -> Self {
        self.cadr().cdr()
    }

    pub fn push(&self, next: Self) -> Self {
        if self.is_null() {
            Sexp::cons(next, Sexp::Nil)
        } else {
            Sexp::cons(self.car(), self.cdr().push(next))
        }
    }

    pub fn length(&self) -> Self {
        if !self.is_pair() && !self.is_null() {
            return Sexp::Error(Error::UserDefined("Expected list?".into()));
        }
        let mut list = self.clone();
        let mut len = 0;
        while !list.is_null() {
            len += 1;
            list = list.cdr();
        }
        Sexp::from(len)
    }

    pub(crate) fn as_usize(&self) -> usize {
        match self {
            Sexp::Number(Number::Exact(i)) => i.as_usize(),
            _ => panic!("compiler error"),
        }
    }

    pub fn is_procedure(&self) -> bool {
        self.is_tagged_list("primitive".to_string()) ||
        self.is_tagged_list("procedure".to_string())
    }

    pub fn is_true(&self) -> bool {
        match *self {
            Sexp::Bool(b) => b,
            _ => true,
        }
    }

    pub fn is_false(&self) -> bool {
        match *self {
            Sexp::Bool(b) => !b,
            _ => false,
        }
    }

    pub fn set_car(&self, car: Self) -> Self {
        if let Sexp::Pair(p) = self {
            p.set_car(car);
            Sexp::Void
        } else {
            Sexp::Error(Error::PairExpected)
        }
    }

    pub fn set_cdr(&self, cdr: Self) -> Self {
        if let Sexp::Pair(p) = self {
            p.set_cdr(cdr);
            Sexp::Void
        } else {
            Sexp::Error(Error::PairExpected)
        }
    }
}

impl Display for Sexp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Sexp::Nil => write!(f, "()"),
            Sexp::Void => Ok(()),
            Sexp::Bool(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Sexp::Number(n) => write!(f, "{}", n),
            Sexp::String(s) => write!(f, "\"{}\"", s),
            Sexp::Symbol(s) => write!(f, "{}", s),
            Sexp::Pair(p) => {
                write!(f, "({}", p.car())?;
                let mut pair = p.cdr();
                while !pair.is_null() {
                    match pair {
                        Sexp::Pair(p) => {
                            write!(f, " {}", p.car())?;
                            pair = p.cdr();
                        },
                        _ => return write!(f, " . {})", pair),
                    }
                }
                write!(f, ")")
            }
            // TODO: print proc name and number of args
            Sexp::Lambda(_l) => write!(f, "#<procedure TODO>"),
            Sexp::Primitive(l) => write!(f, "#<procedure {} {} args>", l.name, l.args),
            Sexp::Error(e) => write!(f, "ERROR: {}", e),
        }
    }
}

impl From<i64> for Sexp {
    fn from(n: i64) -> Self {
        Sexp::Number(Number::from(n))
    }
}

impl From<bool> for Sexp {
    fn from(n: bool) -> Self {
        Sexp::Bool(n)
    }
}

#![feature(nll)]
#![cfg_attr(feature="profile", feature(plugin, custom_attribute))]
#![cfg_attr(feature="profile", plugin(flamer))]
#[cfg(feature="profile")]
extern crate flame;

#[macro_use]
extern crate derive_is_enum_variant;
#[macro_use]
extern crate lazy_static;
//#[macro_use]
//extern crate maplit;
extern crate ramp;
extern crate regex;
extern crate vm;

mod compiler;
//mod environment;
mod error;
mod parser;
//mod sexp;

pub use compiler::{Ast, compile, CompilePrimitive};
//pub use environment::{Environment, init_env};
pub use error::Error;
pub use parser::{Parser, Token};
//pub use sexp::{Lambda, Number, ComplexExact, ComplexFloating, Sexp, Pair, Primitive};

/*
#[cfg_attr(feature="profile", flame)]
pub fn eval(exp: Sexp, env: &Environment) -> Sexp {
    if exp.is_self_evaluating() {
        exp
    } else if exp.is_variable() {
        exp.lookup_variable_value(env)
    } else if exp.is_quoted() {
        exp.text_of_quotation()
    } else if exp.is_assignment() {
        exp.eval_assignment(env)
    } else if exp.is_definition() {
        exp.eval_definition(env)
    } else if exp.is_if() {
        exp.eval_if(env)
    } else if exp.is_lambda() {
        exp.make_procedure(env)
    } else if exp.is_begin() {
        exp.eval_sequence(env)
    } else if exp.is_cond() {
        eval(exp.cond_to_if(), env)
    } else if exp.is_application() {
        let operator = eval(exp.operator(), env);
        if operator.is_error() {
            return operator;
        }
        let operands = exp.operands().list_of_values(env);
        if operands.is_error() {
            return operands;
        }
        apply(operator, operands)
    } else {
        Sexp::Error(Error::UserDefined(format!("Unknown expression type {}", exp)))
    }
}

#[cfg_attr(feature="profile", flame)]
pub fn apply(procedure: Sexp, mut arguments: Sexp) -> Sexp {
    if procedure.is_primitive_procedure() {
        procedure.apply_primitive_procedure(arguments)
    } else if procedure.is_compound_procedure() {
        let mut env = procedure.procedure_env();
        let parameters = procedure.procedure_parameters();

        let arity = procedure.procedure_arity();
        let number_of_args = arguments.length().as_usize();
        if !arity.correct_number_of_args(number_of_args) {
            return Sexp::Error(Error::WrongArgs);
        }

        let variadic = arity.is_variadic();
        let arity = arity.as_usize();
        for i in 0..arity - 1 {
            env.define_variable(parameters[i].clone(), arguments.car());
            arguments = arguments.cdr();
        }

        // Handle the last argument
        if variadic {
            env.define_variable(parameters[arity-1].clone(), arguments);
        } else {
            env.define_variable(parameters[arity-1].clone(), arguments.car());
        }

        procedure.procedure_body().eval_sequence(&mut env)
    } else {
        Sexp::Error(Error::UserDefined("Unknown procedure type".to_string()))
    }
}
*/

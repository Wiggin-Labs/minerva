#![feature(match_default_bindings)]
#![cfg_attr(feature="flame_it", feature(plugin, custom_attribute))]
#![cfg_attr(feature="flame_it", plugin(flamer))]
#![cfg_attr(feature="flame_it", flame)]
#[cfg(feature="flame_it")]
extern crate flame;

#[macro_use]
extern crate derive_is_enum_variant;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
extern crate num;
extern crate regex;

//mod bytecode;
mod environment;
mod error;
mod object;
mod parser;
//pub mod vm;

pub use environment::{Environment, init_env};
pub use error::Error;
pub use object::{Lambda, Number, ComplexExact, ComplexFloating, Object, Pair, Primitive};
pub use parser::{Parser, Token};

pub fn eval(exp: Object, env: &Environment) -> Object {
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
    } else if exp.is_macro_def() {
        exp.eval_macro(env)
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
        Object::Error(Error::UserDefined(format!("Unknown expression type {}", exp)))
    }
}

pub fn apply(procedure: Object, mut arguments: Object) -> Object {
    if procedure.is_primitive_procedure() {
        procedure.apply_primitive_procedure(arguments)
    } else if procedure.is_compound_procedure() {
        let mut env = procedure.procedure_env();
        let parameters = procedure.procedure_parameters();

        let arity = procedure.procedure_arity();
        let number_of_args = arguments.length().as_usize();
        if !arity.correct_number_of_args(number_of_args) {
            return Object::Error(Error::WrongArgs);
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
        Object::Error(Error::UserDefined("Unknown procedure type".to_string()))
    }
}

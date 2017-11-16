#![feature(match_default_bindings)]

#[macro_use]
extern crate maplit;
extern crate num;

//mod bytecode;
mod environment;
mod object;
mod parser;
//pub mod vm;

pub use environment::{Environment, init_env};
pub use object::{Lambda, Object, Pair, Primitive};
pub use parser::{Parser, Token};

pub fn eval(exp: Object, env: &Environment) -> Option<Object> {
    if exp.is_self_evaluating() {
        Some(exp)
    } else if exp.is_variable() {
        exp.lookup_variable_value(env)
    } else if exp.is_quoted() {
        exp.text_of_quotation()
    //} else if exp.is_assignment() {
        //exp.eval_assignment(env);
        //None
    } else if exp.is_definition() {
        exp.eval_definition(env);
        None
    } else if exp.is_if() {
        exp.eval_if(env)
    } else if exp.is_lambda() {
        exp.make_procedure(env)
    } else if exp.is_begin() {
        exp.eval_sequence(env)
    } else if exp.is_cond() {
        eval(exp.cond_to_if(), env)
    } else if exp.is_application() {
        apply(eval(exp.operator(), env).unwrap(), exp.operands().list_of_values(env))
    } else {
        Some(Object::Error(format!("Unknown expression type {}", exp)))
    }
}

pub fn apply(procedure: Object, arguments: Object) -> Option<Object> {
    if procedure.is_primitive_procedure() {
        procedure.apply_primitive_procedure(arguments)
    } else if procedure.is_compound_procedure() {
        let mut env = procedure.procedure_env();
        let mut parameters = procedure.procedure_parameters();
        while !parameters.is_null() {
            env.define_variable(parameters.car().symbol_value(), arguments.car());
            parameters = parameters.cdr();
        }
        procedure.procedure_body().eval_sequence(&mut env)
    } else {
        Some(Object::Error(format!("Unbound variable: {}", procedure)))
    }
}

#[cfg(test)]
mod test {
    use super::{eval, init_env, Environment, Object, Parser, Token};

    use num::BigInt;

    fn run(input: &str, env: &Environment) -> Option<Object> {
        let tokens = Parser::parse(input);
        let objects = Token::build_ast(tokens);
        let object = objects[0].clone();
        eval(object, env)
    }

    #[test]
    fn parse_test() {
        let env = init_env();
        let input = "(cons 1 2)";
        let ans = run(input, &env);
        let expected = Object::cons(Object::Number(BigInt::from(1)), Object::Number(BigInt::from(2)));
        assert_eq!(Some(expected), ans);

        let input = "(define a (cons 1 2))";
        assert!(run(input, &env).is_none());

        let input = "(car a)";
        assert_eq!(Some(Object::Number(BigInt::from(1))), run(input, &env));
        let input = "(cons 1 '())";
        assert_eq!(Some(Object::cons(Object::Number(BigInt::from(1)), Object::Nil)), run(input, &env));
        let input = r"
(define (factorial n)
  (if (= n 1)
      1
      (* (factorial (- n 1)) n)))
";
        assert!(run(input, &env).is_none());
        let input = "(factorial 1)";
        assert_eq!(Some(Object::Number(BigInt::from(1))), run(input, &env));
        let input = "(factorial 3)";
        assert_eq!(Some(Object::Number(BigInt::from(6))), run(input, &env));
    }
}

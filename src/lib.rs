#![feature(match_default_bindings)]

#[macro_use]
extern crate derive_is_enum_variant;
#[macro_use]
extern crate maplit;
extern crate num;

//mod bytecode;
mod environment;
mod error;
mod object;
mod parser;
//pub mod vm;

pub use environment::{Environment, init_env};
pub use error::Error;
pub use object::{Lambda, Number, Object, Pair, Primitive};
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
        let mut parameters = procedure.procedure_parameters();

        if arguments.length() != parameters.length() {
            return Object::Error(Error::WrongArgs);
        }

        while !parameters.is_null() {
            env.define_variable(parameters.car().symbol_value(), arguments.car());
            parameters = parameters.cdr();
            arguments = arguments.cdr();
        }
        procedure.procedure_body().eval_sequence(&mut env)
    } else {
        Object::Error(Error::UserDefined("Unknown procedure type".to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::{eval, init_env, Environment, Object, Parser, Token};

    fn run(input: &str, env: &Environment) -> Object {
        let tokens = Parser::parse(input).unwrap();
        let objects = Token::build_ast(tokens).unwrap();
        let object = objects[0].clone();
        eval(object, env)
    }

    #[test]
    fn parse_test() {
        let env = init_env();
        let input = "(cons 1 2)";
        let ans = run(input, &env);
        let expected = Object::cons(Object::from(1),
                                    Object::from(2));
        assert_eq!(expected, ans);

        let input = "(define a (cons 1 2))";
        assert!(run(input, &env).is_void());

        let input = "(car a)";
        assert_eq!(Object::from(1), run(input, &env));
        let input = "(set-car! a 3)";
        assert!(run(input, &env).is_void());
        let input = "(car a)";
        assert_eq!(Object::from(3), run(input, &env));

        let input = "(cons 1 '())";
        assert_eq!(Object::cons(Object::from(1), Object::Nil), run(input, &env));
        let input = r"
(define (factorial n)
  (if (= n 1)
      1
      (* (factorial (- n 1)) n)))
";
        assert!(run(input, &env).is_void());
        let input = "(factorial 1)";
        assert_eq!(Object::from(1), run(input, &env));
        let input = "(factorial 3)";
        assert_eq!(Object::from(6), run(input, &env));

        let input = r"
(define (sum b)
  (define (loop a count)
    (if (= a b)
        count
        (loop (+ a 1) (+ count a))))
  (loop 1 0))
";
        assert!(run(input, &env).is_void());
        let input = "(sum 5)";
        assert_eq!(Object::from(10), run(input, &env));

        let input = "(define a 5)";
        assert!(run(input, &env).is_void());
        assert_eq!(Object::from(5), run("a", &env));
        let input = "(set! a 6)";
        assert!(run(input, &env).is_void());
        assert_eq!(Object::from(6), run("a", &env));

        let input = "(Car '(a b c))";
        assert!(!run(input, &env).is_void());

        let input = "(car b)";
        assert!(!run(input, &env).is_void());

        let input = "(- = 2 3)";
        assert!(!run(input, &env).is_void());

        let input = "((/ 1 2 3) /41 2 3)";
        assert!(!run(input, &env).is_void());

        let input = "'(5 . 6)";
        assert_eq!(Object::cons(Object::from(5), Object::from(6)), run(input, &env));

        let input = "'(5 . ())";
        assert_eq!(Object::cons(Object::from(5), Object::Nil), run(input, &env));

        let input = "'(5 . (a b))";
        let expected = Object::cons(Object::from(5),
                                    Object::cons(Object::Symbol("a".into()),
                                                 Object::cons(Object::Symbol("b".into()),
                                                              Object::Nil)));
        assert_eq!(expected, run(input, &env));
    }
}

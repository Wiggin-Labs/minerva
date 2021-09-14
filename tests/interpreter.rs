extern crate minerva;

/*
use minerva::{eval, init_env, Environment, Sexp, Parser, Token};

fn run(input: &str, env: &Environment) -> Sexp {
    let tokens = Parser::parse(input).unwrap();
    let objects = Token::build_ast(tokens).unwrap();
    let object = objects[0].clone();
    eval(object, env)
}

#[test]
fn cons() {
    let env = init_env();
    let input = "(cons 1 2)";
    let ans = run(input, &env);
    let expected = Sexp::cons(Sexp::from(1),
                              Sexp::from(2));
    assert_eq!(expected, ans);

    let input = "(cons 1 '())";
    assert_eq!(Sexp::cons(Sexp::from(1), Sexp::Nil), run(input, &env));
}

#[test]
fn basics() {
    let env = init_env();

    let input = "(define a (cons 1 2))";
    assert!(run(input, &env).is_void());

    // test car and cdr
    let input = "(car a)";
    assert_eq!(Sexp::from(1), run(input, &env));
    println!("b");
    let input = "(cdr a)";
    assert_eq!(Sexp::from(2), run(input, &env));

    println!("c");
    // set-car! and set-cdr!
    let input = "(set-car! a 3)";
    assert!(run(input, &env).is_void());
    let input = "(car a)";
    assert_eq!(Sexp::from(3), run(input, &env));
    let input = "(cdr a)";
    assert_eq!(Sexp::from(2), run(input, &env));

    let input = "(set-cdr! a 4)";
    assert!(run(input, &env).is_void());
    let input = "(car a)";
    assert_eq!(Sexp::from(3), run(input, &env));
    let input = "(cdr a)";
    assert_eq!(Sexp::from(4), run(input, &env));
}

#[test]
fn env() {
    let env = init_env();
    let input = "(define add +)";
    run(input, &env);
    let input = "(define (+ a b) (add a b))";
    run(input, &env);
    let input = "(+ 1 2)";
    assert_eq!(Sexp::from(3), run(input, &env));
}

#[test]
fn recurse() {
    let env = init_env();

    let input = r"
(define (factorial n)
(if (= n 1)
  1
  (* (factorial (- n 1)) n)))
";
    assert!(run(input, &env).is_void());
    let input = "(factorial 1)";
    assert_eq!(Sexp::from(1), run(input, &env));
    let input = "(factorial 3)";
    assert_eq!(Sexp::from(6), run(input, &env));
}

#[test]
fn tail_recurse() {
    let env = init_env();

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
    assert_eq!(Sexp::from(10), run(input, &env));
}

#[test]
fn set() {
    let env = init_env();

    let input = "(define a 5)";
    assert!(run(input, &env).is_void());
    assert_eq!(Sexp::from(5), run("a", &env));
    let input = "(set! a 6)";
    assert!(run(input, &env).is_void());
    assert_eq!(Sexp::from(6), run("a", &env));
}

#[test]
fn dotted_list() {
    let env = init_env();

    let input = "'(5 . 6)";
    assert_eq!(Sexp::cons(Sexp::from(5), Sexp::from(6)), run(input, &env));

    let input = "'(5 . ())";
    assert_eq!(Sexp::cons(Sexp::from(5), Sexp::Nil), run(input, &env));

    let input = "'(5 . (a b))";
    let expected = Sexp::cons(Sexp::from(5),
                              Sexp::cons(Sexp::Symbol("a".into()),
                                         Sexp::cons(Sexp::Symbol("b".into()),
                                                    Sexp::Nil)));
    assert_eq!(expected, run(input, &env));
}

#[test]
fn lambda() {
    let env = init_env();

    let input = "(lambda (x) x)";
    assert!(run(input, &env).is_procedure());

    let input = "((lambda (x) x) 5)";
    assert_eq!(Sexp::from(5), run(input, &env));
}

#[test]
fn variadic() {
    let env = init_env();

    let input = "(define (list . x) x)";
    assert!(run(input, &env).is_void());

    let input = "(list 1 2 3)";
    let expected = Sexp::cons(Sexp::from(1),
                              Sexp::cons(Sexp::from(2),
                                         Sexp::cons(Sexp::from(3),
                                                    Sexp::Nil)));
    assert_eq!(expected, run(input, &env));

    let input = "((lambda x x) 1 2 3)";
    assert_eq!(expected, run(input, &env));

    let input = "(define (add . x) (if (null? x) 0 (+ (car x) (apply add (cdr x)))))";
    run(input, &env);
    assert_eq!(Sexp::from(6), run("(add 1 2 3)", &env));
}

#[test]
fn fuzz_tests() {
    let env = init_env();

    let input = "(Car '(a b c))";
    assert!(!run(input, &env).is_void());

    let input = "(car b)";
    assert!(!run(input, &env).is_void());

    let input = "(- = 2 3)";
    assert!(!run(input, &env).is_void());

    let input = "((/ 1 2 3) /41 2 3)";
    assert!(!run(input, &env).is_void());
}
*/

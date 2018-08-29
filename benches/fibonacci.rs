extern crate akuma;
#[macro_use]
extern crate criterion;

use akuma::{eval, Environment, Parser, Sexp, Token};
use criterion::Criterion;

fn run(input: &str, env: &Environment) -> Sexp {
    let tokens = Parser::parse(input).unwrap();
    let objects = Token::build_ast(tokens).unwrap();
    let object = objects[0].clone();
    eval(object, env)
}


fn recursive_fibonacci(c: &mut Criterion) {
    let env = akuma::init_env();
    let input = "(define (fib n) (cond ((= n 0) 1) ((= n 1) 1) (else (+ (fib (- n 1)) (fib (- n 2))))))";
    run(input, &env);
    let input = "(fib 10)";
    c.bench_function("fib 10", move |b| b.iter(|| run(input, &env)));
}

criterion_group!(benches, recursive_fibonacci);
criterion_main!(benches);

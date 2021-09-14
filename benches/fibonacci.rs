extern crate minerva;
#[macro_use]
extern crate criterion;
extern crate vm;

use minerva::{Parser, Token};
use vm::{assemble, init_env, VM};
use criterion::Criterion;

fn recursive_fibonacci(c: &mut Criterion) {
    let mut vm = VM::new();
    let env = init_env();
    vm.assign_environment(env);

    let input = "(define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))";
    let tokens = Parser::parse(input).unwrap();
    let mut ast = Token::build_ast(tokens).unwrap();
    let asm = minerva::compile(ast.remove(0));
    vm.load_code(assemble(asm));
    vm.run();

    let input = "(fib 10)";
    let tokens = Parser::parse(input).unwrap();
    let mut ast = Token::build_ast(tokens).unwrap();
    let asm = minerva::compile(ast.remove(0));
    let code = assemble(asm);
    c.bench_function("fib 10", move |b| b.iter(|| {
        vm.run();
        vm.load_code(code.clone());
    }));
}

criterion_group!(benches, recursive_fibonacci);
criterion_main!(benches);

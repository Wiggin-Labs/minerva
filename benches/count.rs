extern crate akuma;
#[macro_use]
extern crate criterion;
extern crate vm;

use akuma::{Parser, Token};
use vm::{assemble, init_env, VM};
use criterion::Criterion;

fn count_10_000(c: &mut Criterion) {
    let mut vm = VM::new();
    let env = init_env();
    vm.assign_environment(env);

    let input = "(define x (lambda (i) (if (= 10000 i) i (x (+ 1 i)))))";
    let tokens = Parser::parse(input).unwrap();
    let mut ast = Token::build_ast(tokens).unwrap();
    let asm = akuma::compile(ast.remove(0));
    vm.load_code(assemble(asm));
    vm.run();

    let input = "(x 0)";
    let tokens = Parser::parse(input).unwrap();
    let mut ast = Token::build_ast(tokens).unwrap();
    let asm = akuma::compile(ast.remove(0));
    let code = assemble(asm);
    c.bench_function("count 10000", move |b| b.iter(|| {
        vm.run();
        vm.load_code(code.clone());
    }));
}

criterion_group!(benches, count_10_000);
criterion_main!(benches);

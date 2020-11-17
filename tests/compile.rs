extern crate akuma;
extern crate vm;

use akuma::{Ast, compile, CompilePrimitive};
use vm::{assemble, init_env, Register, Value, VM};

#[test]
fn factorial() {
    let code = Ast::Define {
        name: "fact".to_string(),
        value: Box::new(
            Ast::Lambda {
                args: vec!["x".to_string()],
                body: vec![Ast::If {
                    predicate: Box::new(Ast::Apply(vec![Ast::Ident("=".to_string()),
                                                        Ast::Ident("x".to_string()),
                                                        Ast::Primitive(CompilePrimitive::Integer(1))])),
                    consequent: Box::new(Ast::Primitive(CompilePrimitive::Integer(1))),
                    alternative: Box::new(Ast::Apply(vec![Ast::Ident("*".to_string()),
                                                          Ast::Ident("x".to_string()),
                                                          Ast::Apply(vec![Ast::Ident("fact".to_string()),
                                                                          Ast::Apply(vec![Ast::Ident("-".to_string()),
                                                                                          Ast::Ident("x".to_string()),
                                                                                          Ast::Primitive(CompilePrimitive::Integer(1))])])]))}]}
    )};
    let asm = compile(code);
    let mut vm = VM::new();
    let env = init_env();
    vm.assign_environment(env);
    vm.load_code(assemble(asm));
    vm.run();

    let code = Ast::Apply(vec![Ast::Ident("fact".to_string()), Ast::Primitive(CompilePrimitive::Integer(5))]);
    let asm = compile(code);
    vm.load_code(assemble(asm));
    vm.run();
    assert_eq!(Value::Integer(120), vm.load_register(Register(0)));
}

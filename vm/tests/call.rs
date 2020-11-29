extern crate vm;

use vm::*;

/*
#[test]
fn call_factorial() {
    let factorial = vec![
        ASM::LoadConst(Register(1), Value::Integer(1)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::LoadConst(Register(3), Value::Integer(0)),
        // iter
        ASM::Label("iter".to_string()),
        ASM::LT(Register(4), Register(0), Register(3)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::Eq(Register(4), Register(0), Register(3)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::Mul(Register(1), Register(1), Register(0)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label("iter".to_string())),
        // done
        ASM::Label("done".to_string()),
        ASM::Move(Register(0), Register(1)),
    ];

    let mut vm = VM::new();
    let code = vec![
        // Define factorial
        ASM::MakeClosure(Register(1), Box::new(factorial)),
        ASM::LoadConst(Register(2), Value::String(String::from("factorial"))),
        ASM::StringToSymbol(Register(2), Register(2)),
        ASM::Define(Register(2), Register(1)),
        ASM::Lookup(Register(1), Register(2)),
        ASM::Call(Register(1)),
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register(0), Value::Integer(5));
    vm.run();

    assert_eq!(120, vm.load_register(Register(0)).to_integer());
    assert_eq!(Value::Integer(120), vm.load_register(Register(0)));
    assert_eq!(vm.stack_size(), 0);
}
*/

extern crate vm;

use vm::*;

#[test]
fn call_factorial() {
    let factorial = vec![
        ASM::LoadConst(Register::B, Value::Integer(1)),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::LoadConst(Register::D, Value::Integer(0)),
        // iter
        ASM::Label("iter".to_string()),
        ASM::LT(Register::Flag, Register::A, Register::D),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register::Flag),
        ASM::Eq(Register::Flag, Register::A, Register::D),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register::Flag),
        ASM::Mul(Register::B, Register::B, Register::A),
        ASM::Sub(Register::A, Register::A, Register::C),
        ASM::Goto(GotoValue::Label("iter".to_string())),
        // done
        ASM::Label("done".to_string()),
        ASM::Move(Register::A, Register::B),
    ];

    let mut vm = VM::new();
    let code = vec![
        // Define factorial
        ASM::MakeClosure(Register::B, Box::new(factorial)),
        ASM::LoadConst(Register::C, Value::String(String::from("factorial"))),
        ASM::StringToSymbol(Register::C, Register::C),
        ASM::Define(Register::C, Register::B),
        ASM::Lookup(Register::B, Register::C),
        ASM::Call(Register::B),
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register::A, Value::Integer(5));
    vm.run();

    assert_eq!(120, vm.load_register(Register::A).to_integer());
    assert_eq!(Value::Integer(120), vm.load_register(Register::A));
    assert_eq!(vm.stack_size(), 0);
}

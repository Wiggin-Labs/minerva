extern crate vm;

use vm::*;

#[test]
fn assemble_iterative_factorial() {
    let code = vec![
        ASM::LoadConst(Register::B, Value::Integer(1)),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::LoadConst(Register::D, Value::Integer(0)),
        // iter
        ASM::Label("Iter".to_string()),
        ASM::LT(Register::Flag, Register::A, Register::D),
        ASM::GotoIf(GotoValue::Label("Done".to_string()), Register::Flag),
        ASM::Eq(Register::Flag, Register::A, Register::D),
        ASM::GotoIf(GotoValue::Label("Done".to_string()), Register::Flag),
        ASM::Mul(Register::B, Register::B, Register::A),
        ASM::Sub(Register::A, Register::A, Register::C),
        ASM::Goto(GotoValue::Label("Iter".to_string())),
        // done
        ASM::Label("Done".to_string()),
        ASM::Move(Register::A, Register::B),
    ];
    let code = assemble(code);

    let expected_code = vec![
        Operation::LoadConst(Register::B),
        Operation(Value::Integer(1).0 as u32),
        Operation((Value::Integer(1).0 >> 32) as u32),
        Operation::LoadConst(Register::C),
        Operation(Value::Integer(1).0 as u32),
        Operation((Value::Integer(1).0 >> 32) as u32),
        Operation::LoadConst(Register::D),
        Operation(Value::Integer(0).0 as u32),
        Operation((Value::Integer(0).0 >> 32) as u32),
        Operation::LT(Register::Flag, Register::A, Register::D),
        Operation::GotoIf(Register::Flag, Some(16)),
        Operation::Eq(Register::Flag, Register::A, Register::D),
        Operation::GotoIf(Register::Flag, Some(16)),
        Operation::Mul(Register::B, Register::B, Register::A),
        Operation::Sub(Register::A, Register::A, Register::C),
        Operation::Goto(Some(9)),
        Operation::Move(Register::A, Register::B),
    ];
    assert_eq!(expected_code, code);
}

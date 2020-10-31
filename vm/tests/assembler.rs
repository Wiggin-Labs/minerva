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
    let (code, consts) = assemble(code);

    let expected_consts = vec![Value::Integer(1), Value::Integer(0)];
    assert_eq!(expected_consts, consts);

    let expected_code = vec![
        Operation::LoadConst(Register::B, 0),
        Operation::LoadConst(Register::C, 0),
        Operation::LoadConst(Register::D, 1),
        Operation::LT(Register::Flag, Register::A, Register::D),
        Operation::GotoIf(Register::Flag, Some(10)),
        Operation::Eq(Register::Flag, Register::A, Register::D),
        Operation::GotoIf(Register::Flag, Some(10)),
        Operation::Mul(Register::B, Register::B, Register::A),
        Operation::Sub(Register::A, Register::A, Register::C),
        Operation::Goto(Some(3)),
        Operation::Move(Register::A, Register::B),
    ];
    assert_eq!(expected_code, code);
}
extern crate vm;

use vm::*;

/*
#[test]
fn assemble_iterative_factorial() {
    let code = vec![
        ASM::LoadConst(Register(1), Value::Integer(1)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::LoadConst(Register(3), Value::Integer(0)),
        // iter
        ASM::Label("Iter".to_string()),
        ASM::LT(Register(4), Register(0), Register(3)),
        ASM::GotoIf(GotoValue::Label("Done".to_string()), Register(4)),
        ASM::Eq(Register(4), Register(0), Register(3)),
        ASM::GotoIf(GotoValue::Label("Done".to_string()), Register(4)),
        ASM::Mul(Register(1), Register(1), Register(0)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label("Iter".to_string())),
        // done
        ASM::Label("Done".to_string()),
        ASM::Move(Register(0), Register(1)),
    ];
    let code = assemble(code);

    let expected_code = vec![
        Operation::LoadConst(Register(1)),
        Operation(Value::Integer(1).0 as u32),
        Operation((Value::Integer(1).0 >> 32) as u32),
        Operation::LoadConst(Register(2)),
        Operation(Value::Integer(1).0 as u32),
        Operation((Value::Integer(1).0 >> 32) as u32),
        Operation::LoadConst(Register(3)),
        Operation(Value::Integer(0).0 as u32),
        Operation((Value::Integer(0).0 >> 32) as u32),
        Operation::LT(Register(4), Register(0), Register(3)),
        Operation::GotoIf(Register(4), Some(16)),
        Operation::Eq(Register(4), Register(0), Register(3)),
        Operation::GotoIf(Register(4), Some(16)),
        Operation::Mul(Register(1), Register(1), Register(0)),
        Operation::Sub(Register(0), Register(0), Register(2)),
        Operation::Goto(Some(9)),
        Operation::Move(Register(0), Register(1)),
    ];
    assert_eq!(expected_code, code);
}
*/

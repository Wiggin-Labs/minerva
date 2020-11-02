extern crate vm;

use vm::*;

#[test]
fn object_size() {
    assert_eq!(8, std::mem::size_of::<Value>());
    assert_eq!(32, std::mem::size_of::<ASM>());
    assert_eq!(4, std::mem::size_of::<Operation>());
    assert_eq!(1, std::mem::size_of::<Instruction>());
}

#[test]
fn recursive_factorial() {
    let mut vm = VM::new();
    let code = vec![
        ASM::LoadContinue("done".to_string()),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        // loop
        ASM::Label("loop".to_string()),
        ASM::Eq(Register::Flag, Register::A, Register::C),
        ASM::GotoIf(GotoValue::Label("base-case".to_string()), Register::Flag),
        ASM::Save(Register::A),
        ASM::SaveContinue,
        ASM::LoadContinue("after-fact".to_string()),
        ASM::Sub(Register::A, Register::A, Register::C),
        ASM::Goto(GotoValue::Label("loop".to_string())),
        // base case
        ASM::Label("base-case".to_string()),
        ASM::LoadConst(Register::B, Value::Integer(1)),
        ASM::Goto(GotoValue::Register),
        // after-fact
        ASM::Label("after-fact".to_string()),
        ASM::RestoreContinue,
        ASM::Restore(Register::A),
        ASM::Mul(Register::B, Register::B, Register::A),
        ASM::Goto(GotoValue::Register),
        // Done
        ASM::Label("done".to_string()),
        ASM::Move(Register::A, Register::B),
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register::A, Value::Integer(5));
    vm.run();

    assert_eq!(Value::Integer(120), vm.load_register(Register::A));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn iterative_factorial() {
    let mut vm = VM::new();
    let code = vec![
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
    vm.load_code(assemble(code));
    vm.assign_register(Register::A, Value::Integer(5));
    vm.run();

    assert_eq!(Value::Integer(120), vm.load_register(Register::A));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn recursive_fibonacci() {
    let mut vm = VM::new();
    let code = vec![
        ASM::LoadContinue("done".to_string()),
        // Fib loop
        ASM::Label("fib-loop".to_string()),
        ASM::LoadConst(Register::C, Value::Integer(2)),
        ASM::LT(Register::Flag, Register::A, Register::C),
        ASM::GotoIf(GotoValue::Label("immediate-answer".to_string()), Register::Flag),
        ASM::SaveContinue,
        ASM::LoadContinue("after-fib-1".to_string()),
        ASM::Save(Register::A),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::Sub(Register::A, Register::A, Register::C),
        ASM::Goto(GotoValue::Label("fib-loop".to_string())),
        //afterfib n-1
        ASM::Label("after-fib-1".to_string()),
        ASM::Restore(Register::A),
        ASM::LoadConst(Register::C, Value::Integer(2)),
        ASM::Sub(Register::A, Register::A, Register::C),
        ASM::LoadContinue("after-fib-2".to_string()),
        ASM::Save(Register::B),
        ASM::Goto(GotoValue::Label("fib-loop".to_string())),
        //afterfib n-2
        ASM::Label("after-fib-2".to_string()),
        ASM::Move(Register::A, Register::B),
        ASM::Restore(Register::B),
        ASM::RestoreContinue,
        ASM::Add(Register::B, Register::B, Register::A),
        ASM::Goto(GotoValue::Register),
        // immediate answer
        ASM::Label("immediate-answer".to_string()),
        ASM::Move(Register::B, Register::A),
        ASM::Goto(GotoValue::Register),
        // Fib done
        ASM::Label("done".to_string()),
        ASM::Move(Register::A, Register::B),
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register::A, Value::Integer(5));
    vm.run();

    assert_eq!(Value::Integer(5), vm.load_register(Register::A));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn sum_ints() {
    let mut vm = VM::new();
    let code = vec![
        // Setup
        ASM::LoadContinue("done".to_string()),
        ASM::LoadConst(Register::C, Value::Integer(0)),
        ASM::Eq(Register::Flag, Register::A, Register::C),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register::Flag),
        ASM::LoadConst(Register::C, Value::Nil),
        ASM::Cons(Register::B, Register::A, Register::C),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::Sub(Register::A, Register::A, Register::C),
        // list builder loop
        ASM::Label("build-list".to_string()),
        ASM::LoadConst(Register::C, Value::Integer(0)),
        ASM::Eq(Register::Flag, Register::A, Register::C),
        ASM::GotoIf(GotoValue::Label("sum-list".to_string()), Register::Flag),
        ASM::Cons(Register::B, Register::A, Register::B),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::Sub(Register::A, Register::A, Register::C),
        ASM::Goto(GotoValue::Label("build-list".to_string())),
        // sum list
        ASM::Label("sum-list".to_string()),
        ASM::LoadConst(Register::C, Value::Nil),
        ASM::Eq(Register::Flag, Register::B, Register::C),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register::Flag),
        ASM::Car(Register::C, Register::B),
        ASM::Add(Register::A, Register::A, Register::C),
        ASM::Cdr(Register::B, Register::B),
        ASM::Goto(GotoValue::Label("sum-list".to_string())),
        // done
        ASM::Label("done".to_string()),
        ASM::Return,
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register::A, Value::Integer(5));
    vm.run();

    assert_eq!(Value::Integer(15), vm.load_register(Register::A));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
#[ignore]
fn count_to_1billion() {
    let mut vm = VM::new();
    let code = vec![
        ASM::LoadConst(Register::B, Value::Integer(1)),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        // iter
        ASM::Label("iter".to_string()),
        ASM::Eq(Register::Flag, Register::A, Register::B),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register::Flag),
        ASM::Add(Register::B, Register::B, Register::C),
        ASM::Goto(GotoValue::Label("iter".to_string())),
        // done
        ASM::Label("done".to_string()),
        ASM::Move(Register::A, Register::B),
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register::A, Value::Integer(1_000_000_000));
    vm.run();
}

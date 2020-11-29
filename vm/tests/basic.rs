extern crate string_interner;
extern crate vm;

use string_interner::get_symbol;
use vm::*;

#[test]
fn object_size() {
    assert_eq!(8, std::mem::size_of::<Value>());
    assert_eq!(24, std::mem::size_of::<ASM>());
    assert_eq!(4, std::mem::size_of::<Operation>());
    assert_eq!(1, std::mem::size_of::<Instruction>());
}

#[test]
fn recursive_factorial() {
    let mut vm = VM::new();
    let code = vec![
        ASM::LoadContinue(get_symbol("done".to_string())),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        // loop
        ASM::Label(get_symbol("loop".to_string())),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label(get_symbol("base-case".to_string())), Register(4)),
        ASM::Save(Register(0)),
        ASM::SaveContinue,
        ASM::LoadContinue(get_symbol("after-fact".to_string())),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label(get_symbol("loop".to_string()))),
        // base case
        ASM::Label(get_symbol("base-case".to_string())),
        ASM::LoadConst(Register(1), Value::Integer(1)),
        ASM::Goto(GotoValue::Register),
        // after-fact
        ASM::Label(get_symbol("after-fact".to_string())),
        ASM::RestoreContinue,
        ASM::Restore(Register(0)),
        ASM::Mul(Register(1), Register(1), Register(0)),
        ASM::Goto(GotoValue::Register),
        // Done
        ASM::Label(get_symbol("done".to_string())),
        ASM::Move(Register(0), Register(1)),
    ];

    let (code, consts) = assemble(code);
    vm.load_code(code, consts);
    vm.assign_register(Register(0), Value::Integer(5));
    vm.run();

    assert_eq!(Value::Integer(120), vm.load_register(Register(0)));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn iterative_factorial() {
    let mut vm = VM::new();
    let code = vec![
        ASM::LoadConst(Register(1), Value::Integer(1)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::LoadConst(Register(3), Value::Integer(0)),
        // iter
        ASM::Label(get_symbol("iter".to_string())),
        ASM::LT(Register(4), Register(0), Register(3)),
        ASM::GotoIf(GotoValue::Label(get_symbol("done".to_string())), Register(4)),
        ASM::Eq(Register(4), Register(0), Register(3)),
        ASM::GotoIf(GotoValue::Label(get_symbol("done".to_string())), Register(4)),
        ASM::Mul(Register(1), Register(1), Register(0)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label(get_symbol("iter".to_string()))),
        // done
        ASM::Label(get_symbol("done".to_string())),
        ASM::Move(Register(0), Register(1)),
    ];
    let (code, consts) = assemble(code);
    vm.load_code(code, consts);
    vm.assign_register(Register(0), Value::Integer(5));
    vm.run();

    assert_eq!(Value::Integer(120), vm.load_register(Register(0)));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn recursive_fibonacci() {
    let mut vm = VM::new();
    let code = vec![
        ASM::LoadContinue(get_symbol("done".to_string())),
        // Fib loop
        ASM::Label(get_symbol("fib-loop".to_string())),
        ASM::LoadConst(Register(2), Value::Integer(2)),
        ASM::LT(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label(get_symbol("immediate-answer".to_string())), Register(4)),
        ASM::SaveContinue,
        ASM::LoadContinue(get_symbol("after-fib-1".to_string())),
        ASM::Save(Register(0)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label(get_symbol("fib-loop".to_string()))),
        //afterfib n-1
        ASM::Label(get_symbol("after-fib-1".to_string())),
        ASM::Restore(Register(0)),
        ASM::LoadConst(Register(2), Value::Integer(2)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::LoadContinue(get_symbol("after-fib-2".to_string())),
        ASM::Save(Register(1)),
        ASM::Goto(GotoValue::Label(get_symbol("fib-loop".to_string()))),
        //afterfib n-2
        ASM::Label(get_symbol("after-fib-2".to_string())),
        ASM::Move(Register(0), Register(1)),
        ASM::Restore(Register(1)),
        ASM::RestoreContinue,
        ASM::Add(Register(1), Register(1), Register(0)),
        ASM::Goto(GotoValue::Register),
        // immediate answer
        ASM::Label(get_symbol("immediate-answer".to_string())),
        ASM::Move(Register(1), Register(0)),
        ASM::Goto(GotoValue::Register),
        // Fib done
        ASM::Label(get_symbol("done".to_string())),
        ASM::Move(Register(0), Register(1)),
    ];

    let (code, consts) = assemble(code);
    vm.load_code(code, consts);
    vm.assign_register(Register(0), Value::Integer(5));
    vm.run();

    assert_eq!(Value::Integer(5), vm.load_register(Register(0)));
    assert_eq!(vm.stack_size(), 0);
}

/*
#[test]
fn sum_ints() {
    let mut vm = VM::new();
    let code = vec![
        // Setup
        ASM::LoadContinue(get_symbol("done".to_string())),
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label(get_symbol("done".to_string())), Register(4)),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Cons(Register(1), Register(0), Register(2)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        // list builder loop
        ASM::Label(get_symbol("build-list".to_string())),
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label(get_symbol("sum-list".to_string())), Register(4)),
        ASM::Cons(Register(1), Register(0), Register(1)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label(get_symbol("build-list".to_string()))),
        // sum list
        ASM::Label(get_symbol("sum-list".to_string())),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Eq(Register(4), Register(1), Register(2)),
        ASM::GotoIf(GotoValue::Label(get_symbol("done".to_string())), Register(4)),
        ASM::Car(Register(2), Register(1)),
        ASM::Add(Register(0), Register(0), Register(2)),
        ASM::Cdr(Register(1), Register(1)),
        ASM::Goto(GotoValue::Label(get_symbol("sum-list".to_string()))),
        // done
        ASM::Label(get_symbol("done".to_string())),
        ASM::Return,
    ];

    let (code, consts) = assemble(code);
    vm.load_code(code, consts);
    vm.assign_register(Register(0), Value::Integer(5));
    vm.run();

    assert_eq!(Value::Integer(15), vm.load_register(Register(0)));
    assert_eq!(vm.stack_size(), 0);
}
*/

#[test]
#[ignore]
fn count_to_1billion() {
    let mut vm = VM::new();
    let code = vec![
        ASM::LoadConst(Register(1), Value::Integer(1)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        // iter
        ASM::Label(get_symbol("iter".to_string())),
        ASM::Eq(Register(4), Register(0), Register(1)),
        ASM::GotoIf(GotoValue::Label(get_symbol("done".to_string())), Register(4)),
        ASM::Add(Register(1), Register(1), Register(2)),
        ASM::Goto(GotoValue::Label(get_symbol("iter".to_string()))),
        // done
        ASM::Label(get_symbol("done".to_string())),
        ASM::Move(Register(0), Register(1)),
    ];

    let (code, consts) = assemble(code);
    vm.load_code(code, consts);
    vm.assign_register(Register(0), Value::Integer(1_000_000_000));
    vm.run();
}

/*
#[test]
fn cons() {
    let mut vm = VM::new();
    let code = vec![
        ASM::LoadConst(Register(1), Value::Integer(1)),
        ASM::LoadConst(Register(2), Value::Integer(2)),
        // iter
        ASM::Label(get_symbol("iter".to_string())),
        ASM::Cons(Register(0), Register(1), Register(2)),
        ASM::Goto(GotoValue::Label(get_symbol("iter".to_string()))),
    ];

    let (code, consts) = assemble(code);
    vm.load_code(code, consts);
    vm.run();
}
*/

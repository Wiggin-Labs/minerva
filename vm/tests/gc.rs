extern crate vm;

use vm::*;

/*
#[test]
fn gc_test() {
    let mut vm = VM::new();
    let code = vec![
        // Create an initial list to later be garbage collected
        ASM::LoadConst(Register(1), Value::Nil),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Cons(Register(1), Register(2), Register(1)),
        ASM::LoadConst(Register(2), Value::Integer(2)),
        ASM::Cons(Register(1), Register(2), Register(1)),
        // Setup
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Cons(Register(1), Register(0), Register(2)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        // list builder loop
        ASM::Label("build-list".to_string()),
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label("sum-list".to_string()), Register(4)),
        ASM::Cons(Register(1), Register(0), Register(1)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label("build-list".to_string())),
        // sum list
        ASM::Label("sum-list".to_string()),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Eq(Register(4), Register(1), Register(2)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::Car(Register(2), Register(1)),
        ASM::Add(Register(0), Register(0), Register(2)),
        ASM::Cdr(Register(1), Register(1)),
        ASM::Goto(GotoValue::Label("sum-list".to_string())),
        // done
        ASM::Label("done".to_string()),
        ASM::Return,
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register(0), Value::Integer(16));
    vm.run();

    assert_eq!(Value::Integer(136), vm.load_register(Register(0)));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn big_garbage() {
    let mut vm = VM::new();
    let code = vec![
        // Setup
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Cons(Register(1), Register(0), Register(2)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        // list builder loop
        ASM::Label("build-list".to_string()),
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label("sum-list".to_string()), Register(4)),
        ASM::Cons(Register(1), Register(0), Register(1)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label("build-list".to_string())),
        // sum list
        ASM::Label("sum-list".to_string()),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Eq(Register(4), Register(1), Register(2)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::Car(Register(2), Register(1)),
        ASM::Add(Register(0), Register(0), Register(2)),
        ASM::Cdr(Register(1), Register(1)),
        ASM::Goto(GotoValue::Label("sum-list".to_string())),
        // done
        ASM::Label("done".to_string()),
        ASM::Move(Register(0), Register(0)),
        // Trigger garbage collection of the sum list
        ASM::LoadConst(Register(1), Value::Nil),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Cons(Register(1), Register(2), Register(1)),
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register(0), Value::Integer(16));
    vm.run();

    assert_eq!(Value::Integer(136), vm.load_register(Register(0)));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn cycle() {
    let mut vm = VM::new();
    let code = vec![
        // Create cycle
        ASM::LoadConst(Register(1), Value::Nil),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Cons(Register(1), Register(2), Register(1)),
        ASM::LoadConst(Register(2), Value::Integer(2)),
        ASM::Cons(Register(2), Register(2), Register(1)),
        ASM::SetCdr(Register(1), Register(2)),
        ASM::LoadConst(Register(2), Value::Nil),
        // Setup
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Cons(Register(1), Register(0), Register(2)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        // list builder loop
        ASM::Label("build-list".to_string()),
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label("sum-list".to_string()), Register(4)),
        ASM::Cons(Register(1), Register(0), Register(1)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label("build-list".to_string())),
        // sum list
        ASM::Label("sum-list".to_string()),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Eq(Register(4), Register(1), Register(2)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::Car(Register(2), Register(1)),
        ASM::Add(Register(0), Register(0), Register(2)),
        ASM::Cdr(Register(1), Register(1)),
        ASM::Goto(GotoValue::Label("sum-list".to_string())),
        // done
        ASM::Label("done".to_string()),
        ASM::Return,
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register(0), Value::Integer(16));
    vm.run();

    assert_eq!(Value::Integer(136), vm.load_register(Register(0)));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn self_cycle() {
    let mut vm = VM::new();
    let code = vec![
        // Create cycle
        ASM::LoadConst(Register(1), Value::Nil),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Cons(Register(1), Register(2), Register(1)),
        ASM::SetCdr(Register(1), Register(1)),
        // Setup
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Cons(Register(1), Register(0), Register(2)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        // list builder loop
        ASM::Label("build-list".to_string()),
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label("sum-list".to_string()), Register(4)),
        ASM::Cons(Register(1), Register(0), Register(1)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label("build-list".to_string())),
        // sum list
        ASM::Label("sum-list".to_string()),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Eq(Register(4), Register(1), Register(2)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::Car(Register(2), Register(1)),
        ASM::Add(Register(0), Register(0), Register(2)),
        ASM::Cdr(Register(1), Register(1)),
        ASM::Goto(GotoValue::Label("sum-list".to_string())),
        // done
        ASM::Label("done".to_string()),
        ASM::Return,
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register(0), Value::Integer(16));
    vm.run();

    assert_eq!(Value::Integer(136), vm.load_register(Register(0)));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn car_eq_cdr() {
    let mut vm = VM::new();
    let code = vec![
        // Create a pair where both car and cdr reference the same list
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::LoadConst(Register(1), Value::Nil),
        ASM::Cons(Register(1), Register(2), Register(1)),
        ASM::Cons(Register(1), Register(1), Register(1)),
        // Setup
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Cons(Register(1), Register(0), Register(2)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        // list builder loop
        ASM::Label("build-list".to_string()),
        ASM::LoadConst(Register(2), Value::Integer(0)),
        ASM::Eq(Register(4), Register(0), Register(2)),
        ASM::GotoIf(GotoValue::Label("sum-list".to_string()), Register(4)),
        ASM::Cons(Register(1), Register(0), Register(1)),
        ASM::LoadConst(Register(2), Value::Integer(1)),
        ASM::Sub(Register(0), Register(0), Register(2)),
        ASM::Goto(GotoValue::Label("build-list".to_string())),
        // sum list
        ASM::Label("sum-list".to_string()),
        ASM::LoadConst(Register(2), Value::Nil),
        ASM::Eq(Register(4), Register(1), Register(2)),
        ASM::GotoIf(GotoValue::Label("done".to_string()), Register(4)),
        ASM::Car(Register(2), Register(1)),
        ASM::Add(Register(0), Register(0), Register(2)),
        ASM::Cdr(Register(1), Register(1)),
        ASM::Goto(GotoValue::Label("sum-list".to_string())),
        // done
        ASM::Label("done".to_string()),
        ASM::Return,
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register(0), Value::Integer(16));
    vm.run();

    assert_eq!(Value::Integer(136), vm.load_register(Register(0)));
    assert_eq!(vm.stack_size(), 0);
}
*/

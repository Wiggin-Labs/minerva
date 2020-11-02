extern crate vm;

use vm::*;

#[test]
fn gc_test() {
    let mut vm = VM::new();
    let code = vec![
        // Create an initial list to later be garbage collected
        ASM::LoadConst(Register::B, Value::Nil),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::Cons(Register::B, Register::C, Register::B),
        ASM::LoadConst(Register::C, Value::Integer(2)),
        ASM::Cons(Register::B, Register::C, Register::B),
        // Setup
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
    vm.assign_register(Register::A, Value::Integer(16));
    vm.run();

    assert_eq!(Value::Integer(136), vm.load_register(Register::A));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn big_garbage() {
    let mut vm = VM::new();
    let code = vec![
        // Setup
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
        ASM::Move(Register::A, Register::A),
        // Trigger garbage collection of the sum list
        ASM::LoadConst(Register::B, Value::Nil),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::Cons(Register::B, Register::C, Register::B),
    ];

    vm.load_code(assemble(code));
    vm.assign_register(Register::A, Value::Integer(16));
    vm.run();

    assert_eq!(Value::Integer(136), vm.load_register(Register::A));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn cycle() {
    let mut vm = VM::new();
    let code = vec![
        // Create cycle
        ASM::LoadConst(Register::B, Value::Nil),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::Cons(Register::B, Register::C, Register::B),
        ASM::LoadConst(Register::C, Value::Integer(2)),
        ASM::Cons(Register::C, Register::C, Register::B),
        ASM::SetCdr(Register::B, Register::C),
        ASM::LoadConst(Register::C, Value::Nil),
        // Setup
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
    vm.assign_register(Register::A, Value::Integer(16));
    vm.run();

    assert_eq!(Value::Integer(136), vm.load_register(Register::A));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn self_cycle() {
    let mut vm = VM::new();
    let code = vec![
        // Create cycle
        ASM::LoadConst(Register::B, Value::Nil),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::Cons(Register::B, Register::C, Register::B),
        ASM::SetCdr(Register::B, Register::B),
        // Setup
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
    vm.assign_register(Register::A, Value::Integer(16));
    vm.run();

    assert_eq!(Value::Integer(136), vm.load_register(Register::A));
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn car_eq_cdr() {
    let mut vm = VM::new();
    let code = vec![
        // Create a pair where both car and cdr reference the same list
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::LoadConst(Register::B, Value::Nil),
        ASM::Cons(Register::B, Register::C, Register::B),
        ASM::Cons(Register::B, Register::B, Register::B),
        // Setup
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
    vm.assign_register(Register::A, Value::Integer(16));
    vm.run();

    assert_eq!(Value::Integer(136), vm.load_register(Register::A));
    assert_eq!(vm.stack_size(), 0);
}

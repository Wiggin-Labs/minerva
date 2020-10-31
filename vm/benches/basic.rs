#[macro_use]
extern crate criterion;
extern crate vm;

use criterion::Criterion;
use vm::*;

fn iterative_factorial(c: &mut Criterion) {
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

    c.bench_function("iterative factorial of 5", move |b| b.iter(|| {
        vm.run();
        vm.reset();
        vm.assign_register(Register::A, Value::Integer(5));
    }));
}

fn recursive_factorial(c: &mut Criterion) {
    let mut vm = VM::new();
    let code = vec![
        ASM::LoadContinue("done".to_string()),
        // loop
        ASM::Label("loop".to_string()),
        ASM::LoadConst(Register::C, Value::Integer(1)),
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
    let (code, consts) = assemble(code);
    vm.load_code(code);
    vm.load_constants(consts);
    vm.assign_register(Register::A, Value::Integer(5));

    c.bench_function("recursive factorial of 5", move |b| b.iter(|| {
        vm.run();
        vm.reset();
        vm.assign_register(Register::A, Value::Integer(5));
    }));
}

fn recursive_fibonacci(c: &mut Criterion) {
    let mut vm = VM::new();
    let code = vec![
        ASM::LoadContinue("done".to_string()),
        // Fib loop
        ASM::Label("loop".to_string()),
        ASM::LoadConst(Register::C, Value::Integer(2)),
        ASM::LT(Register::Flag, Register::A, Register::C),
        ASM::GotoIf(GotoValue::Label("immediate-answer".to_string()), Register::Flag),
        ASM::SaveContinue,
        ASM::LoadContinue("after-fib-1".to_string()),
        ASM::Save(Register::A),
        ASM::LoadConst(Register::C, Value::Integer(1)),
        ASM::Sub(Register::A, Register::A, Register::C),
        ASM::Goto(GotoValue::Label("loop".to_string())),
        //afterfib n-1
        ASM::Label("after-fib-1".to_string()),
        ASM::Restore(Register::A),
        ASM::LoadConst(Register::C, Value::Integer(2)),
        ASM::Sub(Register::A, Register::A, Register::C),
        ASM::LoadContinue("after-fib-2".to_string()),
        ASM::Save(Register::B),
        ASM::Goto(GotoValue::Label("loop".to_string())),
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
    let (code, consts) = assemble(code);
    vm.load_code(code);
    vm.load_constants(consts);
    vm.assign_register(Register::A, Value::Integer(5));

    c.bench_function("recursive fibonacci of 5", move |b| b.iter(|| {
        vm.run();
        vm.reset();
        vm.assign_register(Register::A, Value::Integer(5));
    }));
}

criterion_group!(benches,
                 iterative_factorial,
                 recursive_factorial,
                 recursive_fibonacci);
criterion_main!(benches);

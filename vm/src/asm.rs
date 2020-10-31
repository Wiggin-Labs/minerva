use {Instruction, Environment, Lambda, Operation, Value};

use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash)]
#[repr(u8)]
pub enum Register {
    Flag = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
}

impl From<u64> for Register {
    fn from(r: u64) -> Self {
        match r {
            0 => Register::Flag,
            1 => Register::A,
            2 => Register::B,
            3 => Register::C,
            4 => Register::D,
            _ => panic!("Invalid register value {}", r),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GotoValue {
    Label(String),
    Register,
}

// TODO: optimize size here. currently 64 bytes...
/// The instructions for the virtual machine.
#[derive(Clone, Debug, PartialEq)]
pub enum ASM {
    // Instructions for the continue register
    /// Load a Label to the `continue` register.
    LoadContinue(String),
    /// Save the `continue` register to the continue stack.
    SaveContinue,
    /// Restore the last item on the continue stack to the `continue` register.
    RestoreContinue,
    // Stack instructions
    /// Save a general purpose register to the stack.
    Save(Register),
    /// Restore the last item on the stack to a general purpose register.
    Restore(Register),
    // Register instructions
    /// LoadConst(reg, arg) Place a constant `arg` in `reg`.
    LoadConst(Register, Value),
    MakeClosure(Register, Box<Vec<ASM>>),
    /// Move(reg1, reg2) Copy the value in `reg2` to `reg1`.
    Move(Register, Register),
    // Branch instructions
    /// Goto either a label or the current value in the `continue` register.
    Goto(GotoValue),
    GotoIf(GotoValue, Register),
    GotoIfNot(GotoValue, Register),
    // Integer arithmetic instructions
    /// Add(reg, arg1, arg2) Compute `arg1 + arg2` and place the result in `reg`.
    Add(Register, Register, Register),
    /// Sub(reg, arg1, arg2) Compute `arg1 - arg2` and place the result in `reg`.
    Sub(Register, Register, Register),
    /// Mul(reg, arg1, arg2) Compute `arg1 * arg2` and place the result in `reg`.
    Mul(Register, Register, Register),
    Eq(Register, Register, Register),
    LT(Register, Register, Register),
    StringToSymbol(Register, Register),
    // Pair operations
    /// Cons(reg, arg1, arg2) Create a pair of `(cons arg1 arg2)` and place the result in `reg`.
    Cons(Register, Register, Register),
    /// Car(reg1, reg2) Retrive the car of `reg2` and place the result in `reg1`.
    Car(Register, Register),
    /// Cdr(reg1, reg2) Retrive the cdr of `reg2` and place the result in `reg1`.
    Cdr(Register, Register),
    /// SetCar(reg, arg) Set the car of the pair in `reg` to `arg`.
    SetCar(Register, Register),
    /// SetCdr(reg, arg) Set the cdr of the pair in `reg` to `arg`.
    SetCdr(Register, Register),
    Define(Register, Register),
    Lookup(Register, Register),
    Call(Register),
    Return,
    Label(String),
}

pub fn assemble(asm: Vec<ASM>) -> (Vec<Operation>, Vec<Value>) {
    let mut ops = Vec::new();
    let mut constants = Vec::new();
    let mut labels = HashMap::new();
    let mut jumps = Vec::new();

    for inst in asm {
        match inst {
            ASM::Label(l) => {
                if labels.contains_key(&l) {
                    panic!("Label `{}` defined more than once", l);
                }
                labels.insert(l, ops.len());
            }
            ASM::LoadContinue(l) => if let Some(p) = labels.get(&l) {
                ops.push(Operation::LoadContinue(*p));
            } else {
                jumps.push((l, ops.len()));
                ops.push(Operation::LoadContinue(0));
            },
            ASM::SaveContinue => ops.push(Operation::SaveContinue),
            ASM::RestoreContinue => ops.push(Operation::RestoreContinue),
            ASM::Save(r) => ops.push(Operation::Save(r)),
            ASM::Restore(r) => ops.push(Operation::Restore(r)),
            ASM::LoadConst(r, v) => {
                if let Some(i) = constants.iter().position(|x| *x == v) {
                    ops.push(Operation::LoadConst(r, i as u64));
                } else {
                    ops.push(Operation::LoadConst(r, constants.len() as u64));
                    constants.push(v);
                }
            }
            ASM::MakeClosure(r, code) => {
                // Compile lambda
                let (lambda_code, lambda_consts) = assemble(*code);
                let lambda = Lambda::new(lambda_code, lambda_consts, Environment::new());
                let lambda = Value::Lambda(Box::new(lambda));
                if let Some(i) = constants.iter().position(|x| *x == lambda) {
                    ops.push(Operation::MakeClosure(r, i as u64));
                } else {
                    ops.push(Operation::MakeClosure(r, constants.len() as u64));
                    constants.push(lambda);
                }
            }
            ASM::Move(r1, r2) => ops.push(Operation::Move(r1, r2)),
            ASM::Goto(l) => match l {
                GotoValue::Register => ops.push(Operation::Goto(None)),
                GotoValue::Label(l) => if let Some(p) = labels.get(&l) {
                    ops.push(Operation::Goto(Some(*p)));
                } else {
                    jumps.push((l, ops.len()));
                    ops.push(Operation::Goto(None));
                }
            },
            ASM::GotoIf(l, r) => match l {
                GotoValue::Register => ops.push(Operation::GotoIf(r, None)),
                GotoValue::Label(l) => if let Some(p) = labels.get(&l) {
                    ops.push(Operation::GotoIf(r, Some(*p)));
                } else {
                    jumps.push((l, ops.len()));
                    ops.push(Operation::GotoIf(r, Some(0)));
                }
            },
            ASM::GotoIfNot(l, r) => match l {
                GotoValue::Register => ops.push(Operation::GotoIfNot(r, None)),
                GotoValue::Label(l) => if let Some(p) = labels.get(&l) {
                    ops.push(Operation::GotoIfNot(r, Some(*p)));
                } else {
                    jumps.push((l, ops.len()));
                    ops.push(Operation::GotoIfNot(r, Some(0)));
                }
            },
            ASM::Add(r, a1, a2) => {
                ops.push(Operation::Add(r, a1, a2));
            }
            ASM::Sub(r, a1, a2) => {
                ops.push(Operation::Sub(r, a1, a2));
            }
            ASM::Mul(r, a1, a2) => {
                ops.push(Operation::Mul(r, a1, a2));
            }
            ASM::Eq(r, a1, a2) => {
                ops.push(Operation::Eq(r, a1, a2));
            }
            ASM::LT(r, a1, a2) => {
                ops.push(Operation::LT(r, a1, a2));
            }
            ASM::StringToSymbol(r, a) => {
                ops.push(Operation::StringToSymbol(r, a));
            }
            ASM::Cons(r, a1, a2) => {
                ops.push(Operation::Cons(r, a1, a2));
            }
            ASM::Car(r1, r2) => ops.push(Operation::Car(r1, r2)),
            ASM::Cdr(r1, r2) => ops.push(Operation::Cdr(r1, r2)),
            ASM::SetCar(r, a) => {
                ops.push(Operation::SetCar(r, a));
            }
            ASM::SetCdr(r, a) => {
                ops.push(Operation::SetCdr(r, a));
            }
            ASM::Define(a1, a2) => {
                ops.push(Operation::Define(a1, a2));
            }
            ASM::Lookup(r, a) => {
                ops.push(Operation::Lookup(r, a));
            }
            ASM::Call(r) => ops.push(Operation::Call(r)),
            ASM::Return => ops.push(Operation::Return),
        };
    }

    for (label, i) in jumps {
        let p = if let Some(p) = labels.get(&label) {
            *p
        } else {
            panic!("Unknown label `{}`", label);
        };

        assert!(i < ops.len());
        match ops[i].instruction() {
            Instruction::LoadContinue => ops[i] = Operation::LoadContinue(p),
            Instruction::Goto => ops[i] = Operation::Goto(Some(p)),
            Instruction::GotoIf => ops[i] = ops[i].gotoif_set_label(p),
            Instruction::GotoIfNot => ops[i] = ops[i].gotoifnot_set_label(p),
            _ => unreachable!(),
        }
    }

    (ops, constants)
}

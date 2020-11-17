use {Instruction, Environment, Operation, Value};

use std::collections::HashMap;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub struct Register(pub u8);

impl Register {
    pub fn from_str(r: &str) -> Option<Self> {
        Some(Register(match r {
            "X0" => 0,
            "X1" => 1,
            "X2" => 2,
            "X3" => 3,
            "X4" => 4,
            "X5" => 5,
            "X6" => 6,
            "X7" => 7,
            "X8" => 8,
            "X9" => 9,
            "X10" => 10,
            "X11" => 11,
            "X12" => 12,
            "X13" => 13,
            "X14" => 14,
            "X15" => 15,
            "X16" => 16,
            "X17" => 17,
            "X18" => 18,
            "X19" => 19,
            "X20" => 20,
            "X21" => 21,
            "X22" => 22,
            "X23" => 23,
            "X24" => 24,
            "X25" => 25,
            "X26" => 26,
            "X27" => 27,
            "X28" => 28,
            "X29" | "FP" => 29,
            "X30" | "SP" => 30,
            "X31" | "XZR" => 31,
            _ => return None,
        }))
    }
}

impl From<u32> for Register {
    fn from(r: u32) -> Self {
        if r < 32 {
            Register(r as u8)
        } else {
            panic!("Invalid register value {}", r);
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            29 => write!(f, "FP"),
            30 => write!(f, "SP"),
            31 => write!(f, "XZR"),
            i => write!(f, "X{}", i),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GotoValue {
    Label(String),
    Register,
}

impl fmt::Display for GotoValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GotoValue::Label(s) => write!(f, "`{}`", s),
            GotoValue::Register => write!(f, "LR"),
        }
    }
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
    ReadStack(Register, usize),
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

impl fmt::Display for ASM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ASM::*;
        match self {
            LoadContinue(s) => write!(f, "LOADCONTINUE {}", s),
            SaveContinue => write!(f, "SAVECONTINUE"),
            RestoreContinue => write!(f, "RESTORECONTINUE"),
            Save(r) => write!(f, "SAVE {}", r),
            Restore(r) => write!(f, "RESTORE {}", r),
            ReadStack(r, p) => write!(f, "READSTACK {}, -{}", r, p),
            LoadConst(r, v) => write!(f, "LOADCONST {}, {}", r, v),
            MakeClosure(r, v) => {
                writeln!(f, "MAKECLOSURE {}", r)?;
                for i in &**v {
                    writeln!(f, "\t{}", i)?;
                }
                Ok(())
            },
            Move(r1, r2) => write!(f, "MOVE {}, {}", r1, r2),
            Goto(j) => write!(f, "GOTO {}", j),
            GotoIf(r, j) => write!(f, "GOTOIF {}, {}", r, j),
            GotoIfNot(r, j) => write!(f, "GOTOIFNOT {}, {}", r, j),
            Add(r1, r2, r3) => write!(f, "ADD {}, {}, {}", r1, r2, r3),
            Sub(r1, r2, r3) => write!(f, "SUB {}, {}, {}", r1, r2, r3),
            Mul(r1, r2, r3) => write!(f, "MUL {}, {}, {}", r1, r2, r3),
            Eq(r1, r2, r3) => write!(f, "EQ {}, {}, {}", r1, r2, r3),
            LT(r1, r2, r3) => write!(f, "LT {}, {}, {}", r1, r2, r3),
            StringToSymbol(r1, r2) => write!(f, "STRINGTOSYMBOL {}, {}", r1, r2),
            Cons(r1, r2, r3) => write!(f, "CONS {}, {}, {}", r1, r2, r3),
            Car(r1, r2) => write!(f, "CAR {}, {}", r1, r2),
            Cdr(r1, r2) => write!(f, "CDR {}, {}", r1, r2),
            SetCar(r1, r2) => write!(f, "SETCAR {}, {}", r1, r2),
            SetCdr(r1, r2) => write!(f, "SETCDR {}, {}", r1, r2),
            Define(r1, r2) => write!(f, "DEFINE {}, {}", r1, r2),
            Lookup(r1, r2) => write!(f, "LOOKUP {}, {}", r1, r2),
            Call(r) => write!(f, "CALL {}", r),
            Return => write!(f, "RETURN"),
            Label(s) => write!(f, "{}:", s),
        }
    }
}

pub fn assemble(asm: Vec<ASM>) -> Vec<Operation> {
    let mut ops = Vec::new();
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
            ASM::ReadStack(r, p) => ops.push(Operation::ReadStack(r, p)),
            ASM::LoadConst(r, v) => {
                ops.push(Operation::LoadConst(r));
                ops.push(Operation(v.0 as u32));
                ops.push(Operation((v.0 >> 32) as u32));
            }
            ASM::MakeClosure(r, code) => {
                // Compile lambda
                let lambda_code = assemble(*code);
                // TODO: gc, arity
                let lambda = Value::Lambda(Environment::new(), 0, lambda_code);
                ops.push(Operation::MakeClosure(r));
                ops.push(Operation(lambda.0 as u32));
                ops.push(Operation((lambda.0 >> 32) as u32));
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

    ops
}

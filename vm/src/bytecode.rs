#![allow(non_upper_case_globals, non_snake_case, unused_doc_comments)]

use Register;

use Instruction::*;

use std::fmt;

/// Represents an instruction for our machine. Instructions are 1 byte, allowing for up to 255
/// instructions. The instruction is the lowest byte. The remaining 7 bytes are used for arguments
/// to the instruction and their use differs by instruction.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Hash)]
pub struct Operation(pub u32);

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.instruction() {
            LoadContinue | SaveContinue | RestoreContinue => self.print_continue(f),
            Save | Restore | ReadStack | LoadConst | MakeClosure | Call | TailCall => self.print_register(f),
            Move | Car | Cdr | StringToSymbol | Set | SetCar | SetCdr | Define | Lookup => self.print_register2(f),
            Add | Sub | Mul | Eq | LT | Cons => self.print_register_opvalue2(f),
            Goto | GotoIf | GotoIfNot => self.print_goto(f),
            Return => write!(f, "RETURN"),
        }
    }
}

macro_rules! register {
    ($instruction:ident, $register:ident) => {
        pub fn $instruction(register: Register) -> Self {
            let register = register.0 as u32;
            Operation((register << 8) | ($instruction as u32))
        }

        pub fn $register(self) -> Register {
            Register::from(self.0 >> 8)
        }
    };
}

macro_rules! register_constant {
    ($instruction:ident, $register:ident, $constant:ident) => {
        pub fn $instruction(register: Register, constant: usize) -> Self {
            let register = register.0 as u32;
            let constant = constant as u32;
            Operation((constant << 16) | (register << 8) | ($instruction as u32))
        }

        pub fn $register(self) -> Register {
            Register::from((self.0 >> 8) & 0xFF)
        }

        pub fn $constant(self) -> usize {
            (self.0 >> 16) as usize
        }
    };
}

macro_rules! register2 {
    ($instruction:ident, $to:ident, $from:ident) => {
        pub fn $instruction(to: Register, from: Register) -> Self {
            let to = to.0 as u32;
            let from = from.0 as u32;
            Operation((from << 16) | (to << 8) | ($instruction as u32))
        }

        pub fn $to(self) -> Register {
            Register::from((self.0 >> 8) & 255)
        }

        pub fn $from(self) -> Register {
            Register::from(self.0 >> 16)
        }
    };
}

macro_rules! register_opvalue2 {
    ($instruction:ident, $register:ident, $left:ident, $right:ident) => {
        pub fn $instruction(register: Register, left: Register, right: Register) -> Self {
            let register = register.0 as u32;
            let left = left.0 as u32;
            let right = right.0 as u32;
            Operation((right << 24) | (left << 16) | (register << 8) | ($instruction as u32))
        }

        pub fn $register(self) -> Register {
            Register::from((self.0 >> 8) & 255)
        }

        pub fn $left(self) -> Register {
            Register::from((self.0 >> 16) & 255)
        }

        pub fn $right(self) -> Register {
            Register::from(self.0 >> 24)
        }
    };
}

macro_rules! register_gotovalue {
    ($instruction:ident, $register:ident, $goto:ident, $set:ident) => {
        pub fn $instruction(register: Register, value: Option<usize>) -> Self {
            let register = register.0 as u32;
            let value = if let Some(p) = value { (p as u32) << 16 } else { 1 << 12 };
            Operation(value | (register << 8) | ($instruction as u32))
        }

        pub fn $register(self) -> Register {
            //Register::from((self.0 >> 8) & 15)
            Register::from((self.0 >> 8) & 31)
        }

        pub fn $goto(self) -> Option<usize> {
            let cond = (self.0 >> 12) & 255;
            if cond == 1 {
                None
            } else {
                Some((self.0 >> 16) as usize)
            }
        }

        pub fn $set(self, label: usize) -> Self {
            Operation(((label as u32) << 16) | (65535 & self.0))
        }
    };
}

impl Operation {
    // Retrieve the instruction of an Operation.
    pub fn instruction(self) -> Instruction {
        Instruction::from(self.0 & 255)
    }

    fn print_continue(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.instruction() {
            LoadContinue => write!(f, "LOADCONTINUE {}", self.loadcontinue_label()),
            SaveContinue => write!(f, "SAVECONTINUE"),
            RestoreContinue => write!(f, "RESTORECONTINUE"),
            _ => unreachable!(),
        }
    }

    fn print_register(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.instruction() {
            Save => write!(f, "SAVE {}", self.save_register()),
            Restore => write!(f, "RESTORE {}", self.restore_register()),
            ReadStack => write!(f, "READSTACK {}, -{}", self.readstack_register(), self.readstack_offset()),
            LoadConst => write!(f, "LOADCONST {}", self.loadconst_register()),
            MakeClosure => write!(f, "MAKECLOSURE {}", self.makeclosure_register()),
            Call => write!(f, "CALL {}", self.call_register()),
            TailCall => write!(f, "TAILCALL {}", self.call_register()),
            _ => unreachable!(),
        }
    }

    fn print_register2(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.instruction() {
            Move => write!(f, "MOVE {}, {}", self.move_to(), self.move_from()),
            Car => write!(f, "CAR {}, {}", self.car_to(), self.car_from()),
            Cdr => write!(f, "CDR {}, {}", self.cdr_to(), self.cdr_from()),
            Set => write!(f, "SET {}, {}", self.set_name(), self.set_value()),
            SetCar => write!(f, "SETCAR {}, {}", self.setcar_register(), self.setcar_value()),
            SetCdr => write!(f, "SETCDR {}, {}", self.setcdr_register(), self.setcdr_value()),
            StringToSymbol => write!(f, "STRINGTOSYMBOL {}, {}", self.stringtosymbol_register(), self.stringtosymbol_value()),
            Define => write!(f, "DEFINE {}, {}", self.define_name(), self.define_value()),
            Lookup => write!(f, "LOOKUP {}, {}", self.lookup_register(), self.lookup_name()),
            _ => unreachable!(),
        }
    }

    fn print_register_opvalue2(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.instruction() {
            Add => write!(f, "ADD {}, {}, {}", self.add_register(), self.add_left(), self.add_right()),
            Sub => write!(f, "SUB {}, {}, {}", self.sub_register(), self.sub_left(), self.sub_right()),
            Mul => write!(f, "MUL {}, {}, {}", self.mul_register(), self.mul_left(), self.mul_right()),
            Eq => write!(f, "EQ {}, {}, {}", self.eq_register(), self.eq_left(), self.eq_right()),
            LT => write!(f, "LT {}, {}, {}", self.lt_register(), self.lt_left(), self.lt_right()),
            Cons => write!(f, "CONS {}, {}, {}", self.cons_register(), self.cons_car(), self.cons_cdr()),
            _ => unreachable!(),
        }
    }

    fn print_goto(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.instruction() {
            Goto => if let Some(v) = self.goto_value() {
                write!(f, "GOTO {}", v)
            } else {
                write!(f, "GOTO CONTINUE")
            },
            GotoIf => if let Some(v) = self.gotoif_value() {
                write!(f, "GOTOIF {}, {}", v, self.gotoif_register())
            } else {
                write!(f, "GOTOIF CONTINUE, {}", self.gotoif_register())
            },
            GotoIfNot => if let Some(v) = self.gotoifnot_value() {
                write!(f, "GOTOIFNOT {}, {}", v, self.gotoifnot_register())
            } else {
                write!(f, "GOTOIFNOT CONTINUE, {}", self.gotoifnot_register())
            },
            _ => unreachable!(),
        }
    }

    // Create a LoadContinue instruction. The label takes up the remaining 7 bytes.
    pub fn LoadContinue(label: usize) -> Self {
        let label = label as u32;
        Operation((label << 8) | LoadContinue as u32)
    }

    pub fn loadcontinue_label(self) -> usize {
        (self.0 >> 8) as usize
    }

    // Create a SaveContinue instruction.
    pub const SaveContinue: Self = Operation(SaveContinue as u32);

    // Create a RestoreContinue instruction.
    pub const RestoreContinue: Self = Operation(RestoreContinue as u32);

    // Create a Save instruction. The register uses 1 byte.
    // Retrieve the register used in a Save instruction.
    register!(Save, save_register);

    // Create a Restore instruction. The register uses 1 byte.
    // Retrieve the register used in a Restore instruction.
    register!(Restore, restore_register);

    pub fn ReadStack(register: Register, p: usize) -> Self {
        let register = register.0 as u32;
        let p = p as u32;
        Operation((p << 16) | (register << 8) | (ReadStack as u32))
    }

    pub fn readstack_register(self) -> Register {
        Register::from((self.0 >> 8) & 255)
    }

    pub fn readstack_offset(self) -> usize {
        (self.0 >> 16) as usize
    }


    // Create a LoadConst instruction. The register to load into uses 1 byte.
    register_constant!(LoadConst, loadconst_register, loadconst_constant);

    // Create a MakeClosure instruction. The register to load into uses 1 byte.
    register_constant!(MakeClosure, makeclosure_register, makeclosure_constant);

    // Creates a Move instruction. Takes the form `from-to-Move`.
    // Retrieve the `to` register used in a Move instruction.
    // Retrieve the `from` register used in a Move instruction.
    register2!(Move, move_to, move_from);

    pub fn Goto(value: Option<usize>) -> Self {
        match value {
            Some(p) => Operation(((p as u32) << 16) | Goto as u32),
            None => Operation((1 << 8) | Goto as u32),
        }
    }

    pub fn goto_value(self) -> Option<usize> {
        let cond = (self.0 >> 8) & 255;
        if cond == 1 {
            None
        } else {
            Some((self.0 >> 16) as usize)
        }
    }

    register_gotovalue!(GotoIf, gotoif_register, gotoif_value, gotoif_set_label);
    register_gotovalue!(GotoIfNot, gotoifnot_register, gotoifnot_value, gotoifnot_set_label);

    register_opvalue2!(Add, add_register, add_left, add_right);
    register_opvalue2!(Sub, sub_register, sub_left, sub_right);
    register_opvalue2!(Mul, mul_register, mul_left, mul_right);
    register_opvalue2!(Eq, eq_register, eq_left, eq_right);
    register_opvalue2!(LT, lt_register, lt_left, lt_right);
    register2!(StringToSymbol, stringtosymbol_register, stringtosymbol_value);

    // Creates a Cons instruction. The register to call from uses 1 byte. `car` and `cdr` each
    // take up one byte. If they are values then their byte is 0, otherwise their byte represents
    // their register. Takes the form `cdr-car-register-Cons`.
    // Retrieve the register from a Cons instruction.
    // Retrieve the `car` from a Cons instruction.
    // Retrieve the `cdr` from a Cons instruction.
    register_opvalue2!(Cons, cons_register, cons_car, cons_cdr);

    // Creates a Car instruction. Takes the form from-to-Car.
    // Retrieve the `to` register from a Car instruction.
    // Retrieve the `from` register from a Car instruction.
    register2!(Car, car_to, car_from);

    // Creates a Cdr instruction. Takes the form from-to-Car.
    // Retrieve the `to` register from a Cdr instruction.
    // Retrieve the `from` register from a Cdr instruction.
    register2!(Cdr, cdr_to, cdr_from);


    register2!(Set, set_name, set_value);

    // Creates a SetCar instruction. Takes the form `value-register-SetCar`.
    // Retrieve the register from a SetCar instruction.
    // Retrieve the `value` from a SetCar instruction.
    register2!(SetCar, setcar_register, setcar_value);

    // Creates a SetCdr instruction. Takes the form `value-register-SetCdr`.
    // Retrieve the register from a SetCdr instruction.
    // Retrieve the `value` from a SetCdr instruction.
    register2!(SetCdr, setcdr_register, setcdr_value);

    // Creates a Define instruction. Takes the form `value-name-Define`.
    // Retrive the `name` from a Define instruction.
    // Retrive the `value` from a Define instruction.
    register2!(Define, define_name, define_value);

    // Creates a Lookup instruction. Takes the form `name-register-Define`.
    // Retrive the register from a Lookup instruction.
    // Retrive the `name` from a Lookup instruction.
    register2!(Lookup, lookup_register, lookup_name);

    // Creates a Call instruction. The register to call from uses 1 byte.
    // Retrieve the register from a Call instruction.
    register!(Call, call_register);
    register!(TailCall, tail_call_register);

    // Creates a Return instruction.
    pub const Return: Self = Operation(Return as u32);
}

impl ::std::ops::Deref for Operation {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq)]
pub enum Instruction {
    // Instructions for the continue register
    /// Load a Label to the `continue` register.
    LoadContinue = 0,
    /// Save the `continue` register to the continue stack.
    SaveContinue = 1,
    /// Restore the last item on the continue stack to the `continue` register.
    RestoreContinue = 2,
    // Stack instructions
    /// Save a general purpose register to the stack.
    Save = 3,
    /// Restore the last item on the stack to a general purpose register.
    Restore = 4,
    // Register instructions
    /// LoadConst(reg, arg) Place a constant `arg` in `reg`.
    LoadConst = 5,
    MakeClosure = 6,
    /// Move(reg1, reg2) Copy the value in `reg2` to `reg1`.
    Move = 7,
    // Jump instructions
    /// Goto either a label or the current value in the `continue` register.
    Goto = 8,
    /// GotoIf(label/register, reg) Goto the label/continue register if the value in `reg` is true.
    GotoIf = 9,
    /// GotoIfNot(label/register, reg) Goto the label/continue register if the value in `reg` is false.
    GotoIfNot = 10,
    // Integer arithmetic instructions
    /// Add(reg, arg1, arg2) Compute `arg1 + arg2` and place the result in `reg`.
    Add = 11,
    /// Sub(reg, arg1, arg2) Compute `arg1 - arg2` and place the result in `reg`.
    Sub = 12,
    /// Mul(reg, arg1, arg2) Compute `arg1 * arg2` and place the result in `reg`.
    Mul = 13,
    /// Eq(reg, arg1, arg2) Compute `arg1 == arg2` and place the result in `reg`.
    Eq = 14,
    /// LT(reg, arg1, arg2) Compute `arg1 < arg2` and place the result in `reg`.
    LT = 15,
    StringToSymbol = 16,
    // Pair operations
    /// Cons(reg, arg1, arg2) Create a pair of `(cons arg1 arg2)` and place the result in `reg`.
    Cons = 17,
    /// Car(reg1, reg2) Retrieve the car of `reg2` and place the result in `reg1`.
    Car = 18,
    /// Cdr(reg1, reg2) Retrieve the cdr of `reg2` and place the result in `reg1`.
    Cdr = 19,
    /// SetCar(reg, arg) Set the car of the pair in `reg` to `arg`.
    SetCar = 20,
    /// SetCdr(reg, arg) Set the cdr of the pair in `reg` to `arg`.
    SetCdr = 21,
    Define = 22,
    Lookup = 23,
    Call = 24,
    Return = 25,
    ReadStack = 26,
    Set = 27,
    TailCall = 28,
}

impl From<u32> for Instruction {
    fn from(r: u32) -> Self {
        use Instruction::*;
        match r {
            0 => LoadContinue,
            1 => SaveContinue,
            2 => RestoreContinue,
            3 => Save,
            4 => Restore,
            5 => LoadConst,
            6 => MakeClosure,
            7 => Move,
            8 => Goto,
            9 => GotoIf,
            10 => GotoIfNot,
            11 => Add,
            12 => Sub,
            13 => Mul,
            14 => Eq,
            15 => LT,
            16 => StringToSymbol,
            17 => Cons,
            18 => Car,
            19 => Cdr,
            20 => SetCar,
            21 => SetCdr,
            22 => Define,
            23 => Lookup,
            24 => Call,
            25 => Return,
            26 => ReadStack,
            27 => Set,
            28 => TailCall,
            _ => panic!("Invalid Instruction value {}", r),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_continue() {
        let op = Operation::LoadContinue(1);
        assert_eq!(LoadContinue, op.instruction());
        assert_eq!(1, op.loadcontinue_label());
    }

    #[test]
    fn save_continue() {
        let op = Operation::SaveContinue;
        assert_eq!(SaveContinue, op.instruction());
    }

    #[test]
    fn restore_continue() {
        let op = Operation::RestoreContinue;
        assert_eq!(RestoreContinue, op.instruction());
    }

    #[test]
    fn save() {
        let op = Operation::Save(Register(0));
        assert_eq!(Save, op.instruction());
        assert_eq!(Register(0), op.save_register());
    }

    #[test]
    fn restore() {
        let op = Operation::Restore(Register(0));
        assert_eq!(Restore, op.instruction());
        assert_eq!(Register(0), op.restore_register());
    }

    #[test]
    fn load_const() {
        let op = Operation::LoadConst(Register(0), 0);
        assert_eq!(LoadConst, op.instruction());
        assert_eq!(Register(0), op.loadconst_register());
    }

    #[test]
    fn make_closure() {
        let op = Operation::MakeClosure(Register(0), 0);
        assert_eq!(MakeClosure, op.instruction());
        assert_eq!(Register(0), op.makeclosure_register());
    }

    #[test]
    fn mov() {
        let op = Operation::Move(Register(0), Register(0));
        assert_eq!(Move, op.instruction());
        assert_eq!(Register(0), op.move_to());
        assert_eq!(Register(0), op.move_from());
    }

    #[test]
    fn goto() {
        let op = Operation::Goto(None);
        assert_eq!(Goto, op.instruction());
        assert_eq!(None, op.goto_value());

        let op = Operation::Goto(Some(1));
        assert_eq!(Goto, op.instruction());
        assert_eq!(Some(1), op.goto_value());
    }

    #[test]
    fn goto_if() {
        let op = Operation::GotoIf(Register(4), None);
        assert_eq!(GotoIf, op.instruction());
        assert_eq!(Register(4), op.gotoif_register());
        assert_eq!(None, op.gotoif_value());

        let op = Operation::GotoIf(Register(4), Some(1));
        assert_eq!(GotoIf, op.instruction());
        assert_eq!(Register(4), op.gotoif_register());
        assert_eq!(Some(1), op.gotoif_value());
        let op = op.gotoif_set_label(2);
        assert_eq!(Some(2), op.gotoif_value());
    }

    #[test]
    fn goto_if_not() {
        let op = Operation::GotoIfNot(Register(4), None);
        assert_eq!(GotoIfNot, op.instruction());
        assert_eq!(Register(4), op.gotoifnot_register());
        assert_eq!(None, op.gotoifnot_value());

        let op = Operation::GotoIfNot(Register(4), Some(1));
        assert_eq!(GotoIfNot, op.instruction());
        assert_eq!(Register(4), op.gotoifnot_register());
        assert_eq!(Some(1), op.gotoifnot_value());
        let op = op.gotoifnot_set_label(2);
        assert_eq!(Some(2), op.gotoifnot_value());
    }

    #[test]
    fn add() {
        let op = Operation::Add(Register(0), Register(0), Register(0));
        assert_eq!(Add, op.instruction());
        assert_eq!(Register(0), op.add_register());
        assert_eq!(Register(0), op.add_left());
        assert_eq!(Register(0), op.add_right());
    }

    #[test]
    fn sub() {
        let op = Operation::Sub(Register(0), Register(0), Register(0));
        assert_eq!(Sub, op.instruction());
        assert_eq!(Register(0), op.sub_register());
        assert_eq!(Register(0), op.sub_left());
        assert_eq!(Register(0), op.sub_right());
    }

    #[test]
    fn mul() {
        let op = Operation::Mul(Register(0), Register(0), Register(0));
        assert_eq!(Mul, op.instruction());
        assert_eq!(Register(0), op.mul_register());
        assert_eq!(Register(0), op.mul_left());
        assert_eq!(Register(0), op.mul_right());
    }

    #[test]
    fn symbol_to_string() {
        let op = Operation::StringToSymbol(Register(0), Register(0));
        assert_eq!(StringToSymbol, op.instruction());
        assert_eq!(Register(0), op.stringtosymbol_register());
        assert_eq!(Register(0), op.stringtosymbol_value());
    }

    #[test]
    fn cons() {
        let op = Operation::Cons(Register(0), Register(0), Register(0));
        assert_eq!(Cons, op.instruction());
        assert_eq!(Register(0), op.cons_register());
        assert_eq!(Register(0), op.cons_car());
        assert_eq!(Register(0), op.cons_cdr());
    }

    #[test]
    fn car() {
        let op = Operation::Car(Register(0), Register(0));
        assert_eq!(Car, op.instruction());
        assert_eq!(Register(0), op.car_to());
        assert_eq!(Register(0), op.car_from());
    }

    #[test]
    fn cdr() {
        let op = Operation::Cdr(Register(0), Register(0));
        assert_eq!(Cdr, op.instruction());
        assert_eq!(Register(0), op.cdr_to());
        assert_eq!(Register(0), op.cdr_from());
    }

    #[test]
    fn setcar() {
        let op = Operation::SetCar(Register(0), Register(0));
        assert_eq!(SetCar, op.instruction());
        assert_eq!(Register(0), op.setcar_register());
        assert_eq!(Register(0), op.setcar_value());
    }

    #[test]
    fn setcdr() {
        let op = Operation::SetCdr(Register(0), Register(0));
        assert_eq!(SetCdr, op.instruction());
        assert_eq!(Register(0), op.setcdr_register());
        assert_eq!(Register(0), op.setcdr_value());
    }

    #[test]
    fn define() {
        let op = Operation::Define(Register(0), Register(0));
        assert_eq!(Define, op.instruction());
        assert_eq!(Register(0), op.define_name());
        assert_eq!(Register(0), op.define_value());
    }

    #[test]
    fn lookup() {
        let op = Operation::Lookup(Register(0), Register(0));
        assert_eq!(Lookup, op.instruction());
        assert_eq!(Register(0), op.lookup_register());
        assert_eq!(Register(0), op.lookup_name());
    }

    #[test]
    fn call() {
        let op = Operation::Call(Register(0));
        assert_eq!(Call, op.instruction());
        assert_eq!(Register(0), op.call_register());
    }

    #[test]
    fn ret() {
        let op = Operation::Return;
        assert_eq!(Return, op.instruction());
    }
}

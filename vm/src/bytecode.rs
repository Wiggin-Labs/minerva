#![allow(non_upper_case_globals, non_snake_case, unused_doc_comments)]

use Register;

use Instruction::*;

/// Represents an instruction for our machine. Instructions are 1 byte, allowing for up to 255
/// instructions. The instruction is the lowest byte. The remaining 7 bytes are used for arguments
/// to the instruction and their use differs by instruction.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Hash)]
pub struct Operation(pub u64);

macro_rules! register {
    ($instruction:ident, $register:ident) => {
        pub fn $instruction(register: Register) -> Self {
            let register = register as u64;
            Operation((register << 8) | ($instruction as u64))
        }

        pub fn $register(self) -> Register {
            Register::from(self.0 >> 8)
        }
    };
}

macro_rules! register2 {
    ($instruction:ident, $to:ident, $from:ident) => {
        pub fn $instruction(to: Register, from: Register) -> Self {
            let to = to as u64;
            let from = from as u64;
            Operation((from << 16) | (to << 8) | ($instruction as u64))
        }

        pub fn $to(self) -> Register {
            Register::from((self.0 >> 8) & 255)
        }

        pub fn $from(self) -> Register {
            Register::from(self.0 >> 16)
        }
    };
}

macro_rules! register_opvalue {
    ($instruction:ident, $register:ident, $opvalue:ident) => {
        pub fn $instruction(register: Register, value: Register) -> Self {
            let register = register as u64;
            let value = value as u64;
            Operation((value << 16) | (register << 8) | ($instruction as u64))
        }

        pub fn $register(self) -> Register {
            Register::from((self.0 >> 8) & 255)
        }

        pub fn $opvalue(self) -> Register {
            Register::from(self.0 >> 16)
        }
    };
}

macro_rules! register_opvalue2 {
    ($instruction:ident, $register:ident, $left:ident, $right:ident) => {
        pub fn $instruction(register: Register, left: Register, right: Register) -> Self {
            let register = register as u64;
            let left = left as u64;
            let right = right as u64;
            Operation((right << 24) | (left << 16) | (register << 8) | ($instruction as u64))
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
            let register = register as u64;
            let value = if let Some(p) = value { (p as u64) << 16 } else { 1 << 12 };
            Operation(value | (register << 8) | ($instruction as u64))
        }

        pub fn $register(self) -> Register {
            Register::from((self.0 >> 8) & 15)
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
            Operation(((label as u64) << 16) | (65535 & self.0))
        }
    };
}

impl Operation {
    // Retrieve the instruction of an Operation.
    pub fn instruction(self) -> Instruction {
        Instruction::from(self.0 & 255)
    }

    // Create a LoadContinue instruction. The label takes up the remaining 7 bytes.
    pub fn LoadContinue(label: usize) -> Self {
        let label = label as u64;
        Operation((label << 8) | LoadContinue as u64)
    }

    pub fn loadcontinue_label(self) -> usize {
        (self.0 >> 8) as usize
    }

    // Create a SaveContinue instruction.
    pub const SaveContinue: Self = Operation(SaveContinue as u64);

    // Create a RestoreContinue instruction.
    pub const RestoreContinue: Self = Operation(RestoreContinue as u64);

    // Create a Save instruction. The register uses 1 byte.
    // Retrieve the register used in a Save instruction.
    register!(Save, save_register);

    // Create a Restore instruction. The register uses 1 byte.
    // Retrieve the register used in a Restore instruction.
    register!(Restore, restore_register);

    // Create a LoadConst instruction. The register to load into uses 1 byte.
    pub fn LoadConst(register: Register) -> Self {
        let register = register as u64;
        Operation((register << 8) | LoadConst as u64)
    }

    pub fn loadconst_register(self) -> Register {
        Register::from((self.0 >> 8) & 255)
    }

    // Create a MakeClosure instruction. The register to load into uses 1 byte.
    pub fn MakeClosure(register: Register) -> Self {
        let register = register as u64;
        Operation((register << 8) | MakeClosure as u64)
    }
    pub fn makeclosure_register(self) -> Register {
        Register::from((self.0 >> 8) & 255)
    }

    // Creates a Move instruction. Takes the form `from-to-Move`.
    // Retrieve the `to` register used in a Move instruction.
    // Retrieve the `from` register used in a Move instruction.
    register2!(Move, move_to, move_from);

    pub fn Goto(value: Option<usize>) -> Self {
        match value {
            Some(p) => Operation(((p as u64) << 16) | Goto as u64),
            None => Operation((1 << 8) | Goto as u64),
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
    register_opvalue!(StringToSymbol, stringtosymbol_register, stringtosymbol_value);

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


    // Creates a SetCar instruction. Takes the form `value-register-SetCar`.
    // Retrieve the register from a SetCar instruction.
    // Retrieve the `value` from a SetCar instruction.
    register_opvalue!(SetCar, setcar_register, setcar_value);

    // Creates a SetCdr instruction. Takes the form `value-register-SetCdr`.
    // Retrieve the register from a SetCdr instruction.
    // Retrieve the `value` from a SetCdr instruction.
    register_opvalue!(SetCdr, setcdr_register, setcdr_value);

    // Creates a Define instruction. Takes the form `value-name-Define`.
    pub fn Define(name: Register, value: Register) -> Self {
        let name = name as u64;
        let value = value as u64;
        Operation((value << 16) | (name << 8) | Define as u64)
    }

    // Retrive the `name` from a Define instruction.
    pub fn define_name(self) -> Register {
        Register::from((self.0 >> 8) & 255)
    }

    // Retrive the `value` from a Define instruction.
    pub fn define_value(self) -> Register {
        Register::from(self.0 >> 16)
    }

    // Creates a Lookup instruction. Takes the form `name-register-Define`.
    // Retrive the register from a Lookup instruction.
    // Retrive the `name` from a Lookup instruction.
    register_opvalue!(Lookup, lookup_register, lookup_name);

    // Creates a Call instruction. The register to call from uses 1 byte.
    // Retrieve the register from a Call instruction.
    register!(Call, call_register);

    // Creates a Return instruction.
    pub const Return: Self = Operation(Return as u64);
}

impl ::std::ops::Deref for Operation {
    type Target = u64;

    fn deref(&self) -> &u64 {
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
}

impl From<u64> for Instruction {
    fn from(r: u64) -> Self {
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
        let op = Operation::Save(Register::A);
        assert_eq!(Save, op.instruction());
        assert_eq!(Register::A, op.save_register());
    }

    #[test]
    fn restore() {
        let op = Operation::Restore(Register::A);
        assert_eq!(Restore, op.instruction());
        assert_eq!(Register::A, op.restore_register());
    }

    #[test]
    fn load_const() {
        let op = Operation::LoadConst(Register::A);
        assert_eq!(LoadConst, op.instruction());
        assert_eq!(Register::A, op.loadconst_register());
    }

    #[test]
    fn make_closure() {
        let op = Operation::MakeClosure(Register::A);
        assert_eq!(MakeClosure, op.instruction());
        assert_eq!(Register::A, op.makeclosure_register());
    }

    #[test]
    fn mov() {
        let op = Operation::Move(Register::A, Register::A);
        assert_eq!(Move, op.instruction());
        assert_eq!(Register::A, op.move_to());
        assert_eq!(Register::A, op.move_from());
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
        let op = Operation::GotoIf(Register::Flag, None);
        assert_eq!(GotoIf, op.instruction());
        assert_eq!(Register::Flag, op.gotoif_register());
        assert_eq!(None, op.gotoif_value());

        let op = Operation::GotoIf(Register::Flag, Some(1));
        assert_eq!(GotoIf, op.instruction());
        assert_eq!(Register::Flag, op.gotoif_register());
        assert_eq!(Some(1), op.gotoif_value());
        let op = op.gotoif_set_label(2);
        assert_eq!(Some(2), op.gotoif_value());
    }

    #[test]
    fn goto_if_not() {
        let op = Operation::GotoIfNot(Register::Flag, None);
        assert_eq!(GotoIfNot, op.instruction());
        assert_eq!(Register::Flag, op.gotoifnot_register());
        assert_eq!(None, op.gotoifnot_value());

        let op = Operation::GotoIfNot(Register::Flag, Some(1));
        assert_eq!(GotoIfNot, op.instruction());
        assert_eq!(Register::Flag, op.gotoifnot_register());
        assert_eq!(Some(1), op.gotoifnot_value());
        let op = op.gotoifnot_set_label(2);
        assert_eq!(Some(2), op.gotoifnot_value());
    }

    #[test]
    fn add() {
        let op = Operation::Add(Register::A, Register::A, Register::A);
        assert_eq!(Add, op.instruction());
        assert_eq!(Register::A, op.add_register());
        assert_eq!(Register::A, op.add_left());
        assert_eq!(Register::A, op.add_right());
    }

    #[test]
    fn sub() {
        let op = Operation::Sub(Register::A, Register::A, Register::A);
        assert_eq!(Sub, op.instruction());
        assert_eq!(Register::A, op.sub_register());
        assert_eq!(Register::A, op.sub_left());
        assert_eq!(Register::A, op.sub_right());
    }

    #[test]
    fn mul() {
        let op = Operation::Mul(Register::A, Register::A, Register::A);
        assert_eq!(Mul, op.instruction());
        assert_eq!(Register::A, op.mul_register());
        assert_eq!(Register::A, op.mul_left());
        assert_eq!(Register::A, op.mul_right());
    }

    #[test]
    fn symbol_to_string() {
        let op = Operation::StringToSymbol(Register::A, Register::A);
        assert_eq!(StringToSymbol, op.instruction());
        assert_eq!(Register::A, op.stringtosymbol_register());
        assert_eq!(Register::A, op.stringtosymbol_value());
    }

    #[test]
    fn cons() {
        let op = Operation::Cons(Register::A, Register::A, Register::A);
        assert_eq!(Cons, op.instruction());
        assert_eq!(Register::A, op.cons_register());
        assert_eq!(Register::A, op.cons_car());
        assert_eq!(Register::A, op.cons_cdr());
    }

    #[test]
    fn car() {
        let op = Operation::Car(Register::A, Register::A);
        assert_eq!(Car, op.instruction());
        assert_eq!(Register::A, op.car_to());
        assert_eq!(Register::A, op.car_from());
    }

    #[test]
    fn cdr() {
        let op = Operation::Cdr(Register::A, Register::A);
        assert_eq!(Cdr, op.instruction());
        assert_eq!(Register::A, op.cdr_to());
        assert_eq!(Register::A, op.cdr_from());
    }

    #[test]
    fn setcar() {
        let op = Operation::SetCar(Register::A, Register::A);
        assert_eq!(SetCar, op.instruction());
        assert_eq!(Register::A, op.setcar_register());
        assert_eq!(Register::A, op.setcar_value());
    }

    #[test]
    fn setcdr() {
        let op = Operation::SetCdr(Register::A, Register::A);
        assert_eq!(SetCdr, op.instruction());
        assert_eq!(Register::A, op.setcdr_register());
        assert_eq!(Register::A, op.setcdr_value());
    }

    #[test]
    fn define() {
        let op = Operation::Define(Register::A, Register::A);
        assert_eq!(Define, op.instruction());
        assert_eq!(Register::A, op.define_name());
        assert_eq!(Register::A, op.define_value());
    }

    #[test]
    fn lookup() {
        let op = Operation::Lookup(Register::A, Register::A);
        assert_eq!(Lookup, op.instruction());
        assert_eq!(Register::A, op.lookup_register());
        assert_eq!(Register::A, op.lookup_name());
    }

    #[test]
    fn call() {
        let op = Operation::Call(Register::A);
        assert_eq!(Call, op.instruction());
        assert_eq!(Register::A, op.call_register());
    }

    #[test]
    fn ret() {
        let op = Operation::Return;
        assert_eq!(Return, op.instruction());
    }
}

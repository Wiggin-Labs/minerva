extern crate string_interner;
mod asm;
mod bytecode;
mod environment;
mod value;

pub use asm::{assemble, GotoValue, ASM, Register};
pub use environment::{init_env, Environment};
pub use bytecode::{Instruction, Operation};
pub use value::Value;
pub use value::heap_repr::{Lambda, Pair};

use string_interner::{INTERNER, Symbol};

use std::mem;

/// A Virtual Machine for Scheme.
#[derive(Debug)]
pub struct VM {
    debug: bool,
    step: usize,
    operations: Vec<Operation>,
    environment: Environment,
    stack: Vec<Value>,
    kontinue_stack: Vec<usize>,
    // Registers
    pc: usize,
    kontinue: usize,
    flag: Value,
    registers: [Value; 32],
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    /// Create a new `VM`.
    pub fn new() -> Self {
        VM {
            debug: false,
            step: 0,
            operations: vec![],
            environment: Environment::new(),
            stack: vec![],
            kontinue_stack: vec![],
            pc: 0,
            kontinue: 0,
            flag: Value::Nil,
            registers: [Value::Nil; 32],
        }
    }

    /// Run the currently loaded code.
    pub fn run(&mut self) {
        loop {
            if self.debug { self.print_debug(); }

            if self.pc >= self.operations.len() {
                break;
            }


            let op = self.operations[self.pc];
            self.step += 1;
            self.pc += 1;
            match op.instruction() {
                Instruction::LoadContinue => self.load_kontinue(op),
                Instruction::SaveContinue => self.save_kontinue(),
                Instruction::RestoreContinue => self.restore_kontinue(),
                Instruction::Save => self.save(op),
                Instruction::Restore => self.restore(op),
                Instruction::LoadConst => self.load_const(op),
                Instruction::MakeClosure => self.make_closure(op),
                Instruction::Move => self.mov(op),
                Instruction::Goto => self.goto(op),
                Instruction::GotoIf => self.goto_if(op),
                Instruction::GotoIfNot => self.goto_if_not(op),
                Instruction::Add => self.add(op),
                Instruction::Sub => self.sub(op),
                Instruction::Mul => self.mul(op),
                Instruction::Eq => self.eq(op),
                Instruction::LT => self.lt(op),
                Instruction::StringToSymbol => self.string_to_symbol(op),
                Instruction::Cons => self.cons(op),
                Instruction::Car => self.car(op),
                Instruction::Cdr => self.cdr(op),
                Instruction::SetCar => self.set_car(op),
                Instruction::SetCdr => self.set_cdr(op),
                Instruction::Define => self.define(op),
                Instruction::Lookup => self.lookup(op),
                Instruction::Call => self.call(op),
                Instruction::Return => break,
            }
        }
    }

    /// Reset the machine. Keeps the current code and constants.
    pub fn reset(&mut self) {
        let mut new = Self::new();
        new.debug = self.debug;
        mem::swap(&mut new.operations, &mut self.operations);
        mem::swap(&mut new, self);
    }

    /// Sets the vm to print debug information.
    pub fn set_debug(&mut self) {
        self.debug = true;
    }

    fn print_debug(&mut self) {
        println!("step {}:", self.step);
        for (i, reg) in self.registers.iter().enumerate() {
            println!("X{}: {:?}", i, reg);
        }
        println!("flag: {:?}", self.flag);
        println!("continue: {:?}", self.kontinue);
        println!("stack size: {}", self.stack.len());
        println!();
    }

    /// Returns the current stack size.
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Load code into the machine.
    pub fn load_code(&mut self, code: Vec<Operation>) {
        self.operations = code;
        self.pc = 0;
    }

    /// Set `register` to `value`.
    pub fn assign_register(&mut self, register: Register, value: Value) {
        match register {
            Register::Flag => self.flag = value,
            Register::A => self.registers[0] = value,
            Register::B => self.registers[1] = value,
            Register::C => self.registers[2] = value,
            Register::D => self.registers[3] = value,
        }
    }

    /// Get the value of `register`.
    pub fn load_register(&self, register: Register) -> Value {
        match register {
            Register::Flag => self.flag,
            Register::A => self.registers[0],
            Register::B => self.registers[1],
            Register::C => self.registers[2],
            Register::D => self.registers[3],
        }
    }

    pub fn assign_environment(&mut self, env: Environment) {
        self.environment = env;
    }

    pub fn get_definitions(&self) -> Vec<Symbol> {
        self.environment.get_definitions()
    }

    /// Convert `symbol` to a Symbol.
    pub fn intern_symbol(&mut self, symbol: String) -> Symbol {
        INTERNER.lock().unwrap().get_symbol(symbol)
    }

    /// Get the string value of `symbol`.
    pub fn get_symbol_value(&self, symbol: Symbol) -> String {
        INTERNER.lock().unwrap().get_value(symbol).unwrap()
    }

    /// Assign a label to the `continue` register.
    pub fn assign_continue(&mut self, label: usize) {
        self.kontinue = label;
    }

    fn load_kontinue(&mut self, op: Operation) {
        self.kontinue = op.loadcontinue_label();
    }

    fn save_kontinue(&mut self) {
        self.kontinue_stack.push(self.kontinue);
    }

    fn restore_kontinue(&mut self) {
        assert!(!self.kontinue_stack.is_empty());
        self.kontinue = self.kontinue_stack.pop().unwrap();
    }

    fn save(&mut self, op: Operation) {
        self.stack.push(self.load_register(op.save_register()));
    }

    fn restore(&mut self, op: Operation) {
        assert!(!self.stack.is_empty());
        let value = self.stack.pop().unwrap();
        self.assign_register(op.restore_register(), value);
    }

    fn load_const(&mut self, op: Operation) {
        let c = (self.operations[self.pc+1].0 as u64) << 32;
        let c = c | (self.operations[self.pc].0 as u64);
        let constant = Value(c);
        self.pc += 2;
        self.assign_register(op.loadconst_register(), constant);
    }

    fn make_closure(&mut self, op: Operation) {
        let c = (self.operations[self.pc+1].0 as u64) << 32;
        let c = c | (self.operations[self.pc].0 as u64);
        let pointer = Value(c);
        self.pc += 2;
        let mut lambda = unsafe { Box::from_raw(pointer.to_pointer() as *mut Lambda) };
        // TODO extend env?
        (*lambda).env = self.environment.extend();
        self.assign_register(op.makeclosure_register(), pointer);
        // Make sure this value isn't freed.
        Box::into_raw(lambda);
    }

    fn mov(&mut self, op: Operation) {
        let to = op.move_to();
        let from = op.move_from();
        self.assign_register(to, self.load_register(from));
    }

    fn goto(&mut self, op: Operation) {
        self._goto(op.goto_value());
    }

    fn goto_if(&mut self, op: Operation) {
        if Value::Bool(true) == self.load_register(op.gotoif_register()) {
            self._goto(op.gotoif_value());
        }
    }

    fn goto_if_not(&mut self, op: Operation) {
        if Value::Bool(false) == self.load_register(op.gotoifnot_register()) {
            self._goto(op.gotoifnot_value());
        }
    }

    #[inline]
    fn _goto(&mut self, label: Option<usize>) {
        if let Some(label) = label {
            self.pc = label;
        } else {
            self.pc = self.kontinue;
        }
    }

    fn add(&mut self, op: Operation) {
        let left = self.load_register(op.add_left()).to_integer();
        let right = self.load_register(op.add_right()).to_integer();
        self.assign_register(op.add_register(), Value::Integer(left + right));
    }

    fn sub(&mut self, op: Operation) {
        let left = self.load_register(op.sub_left()).to_integer();
        let right = self.load_register(op.sub_right()).to_integer();
        self.assign_register(op.sub_register(), Value::Integer(left - right));
    }

    fn mul(&mut self, op: Operation) {
        let left = self.load_register(op.mul_left()).to_integer();
        let right = self.load_register(op.mul_right()).to_integer();
        self.assign_register(op.mul_register(), Value::Integer(left * right));
    }

    fn eq(&mut self, op: Operation) {
        let left = self.load_register(op.eq_left());
        let right = self.load_register(op.eq_right());
        self.assign_register(op.eq_register(), Value::Bool(left == right));
    }

    fn lt(&mut self, op: Operation) {
        let left = self.load_register(op.lt_left());
        let right = self.load_register(op.lt_right());
        self.assign_register(op.lt_register(), Value::Bool(left < right));
    }

    fn string_to_symbol(&mut self, op: Operation) {
        // TODO: handle case where `string` isn't a string
        let pointer = self.load_register(op.stringtosymbol_value()).to_string();
        let sym = self.intern_symbol(pointer.p.clone());
        self.assign_register(op.stringtosymbol_register(), Value::Symbol(sym));
        // Make sure this value isn't freed.
        Box::into_raw(pointer);
    }

    fn cons(&mut self, op: Operation) {
        let car = self.load_register(op.cons_car()).clone();
        let cdr = self.load_register(op.cons_cdr()).clone();
        // TODO: gc bits
        let pointer = Value::Pair(car, cdr);

        self.assign_register(op.cons_register(), pointer);
    }

    fn car(&mut self, op: Operation) {
        let pair = self.load_register(op.car_from()).to_pair();
        self.assign_register(op.car_to(), pair.car);
        // Make sure this value isn't freed.
        Box::into_raw(pair);
    }

    fn cdr(&mut self, op: Operation) {
        let pair = self.load_register(op.cdr_from()).to_pair();
        self.assign_register(op.cdr_to(), pair.cdr);
        // Make sure this value isn't freed.
        Box::into_raw(pair);
    }

    fn set_car(&mut self, op: Operation) {
        let mut pair = self.load_register(op.setcar_register()).to_pair();
        let value = self.load_register(op.setcar_value());
        pair.car = value;
        // Make sure this value isn't freed.
        Box::into_raw(pair);
    }

    fn set_cdr(&mut self, op: Operation) {
        let mut pair = self.load_register(op.setcdr_register()).to_pair();
        let value = self.load_register(op.setcdr_value());
        pair.cdr = value;
        // Make sure this value isn't freed.
        Box::into_raw(pair);
    }

    fn define(&mut self, op: Operation) {
        // TODO: handle error when name is not a symbol
        let name = self.load_register(op.define_name()).to_symbol();
        let value = self.load_register(op.define_value());
        self.environment.define_variable(name, value);
    }

    fn lookup(&mut self, op: Operation) {
        // TODO: handle error when name is not a symbol
        let name = self.load_register(op.lookup_name()).to_symbol();
        // TODO: we want an error if `name` is undefined
        let value = self.environment.lookup_variable_value(name).unwrap_or(Value::Void);
        self.assign_register(op.lookup_register(), value);
    }

    fn call(&mut self, op: Operation) {
        if self.debug {
            println!("beginning call");
        }

        // TODO
        let v = self.load_register(op.call_register());
        if v.is_lambda() {
            let lambda = unsafe { Box::from_raw(v.to_pointer() as *mut Lambda) };
            // Save the current code and env
            let mut code = lambda.code.clone();
            let mut env = lambda.env.procedure_local();
            mem::swap(&mut code, &mut self.operations);
            mem::swap(&mut env, &mut self.environment);

            // Save the program counter
            let pc = self.pc;
            self.pc = 0;
            self.run();

            // Restore the saved program counter, code, and environment
            self.pc = pc;
            self.operations = code;
            self.environment = env;
        } else {
            // TODO: return error
        }

        if self.debug {
            println!("ending call");
        }
    }
}

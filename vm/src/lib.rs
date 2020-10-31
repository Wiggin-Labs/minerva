#![feature(nll)]
extern crate string_interner;

mod asm;
mod bytecode;
mod environment;
mod value;

pub use asm::{assemble, GotoValue, ASM, Register};
pub use environment::{init_env, Environment};
pub use bytecode::{Instruction, Operation};
pub use value::{Lambda, Value};

use string_interner::{DefaultStringInterner, StringInterner, Sym};

use std::mem;

/// A Virtual Machine for Scheme.
#[derive(Debug)]
pub struct VM {
    debug: bool,
    step: usize,
    operations: Vec<Operation>,
    constants: Vec<Value>,
    symbols: DefaultStringInterner,
    environment: Environment,
    stack: Vec<Value>,
    kontinue_stack: Vec<usize>,
    // Registers
    pc: usize,
    kontinue: usize,
    flag: Value,
    a: Value,
    b: Value,
    c: Value,
    d: Value,
    max_heap: usize,
    cars: Vec<Value>,
    cdrs: Vec<Value>,
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
            constants: vec![],
            symbols: StringInterner::default(),
            stack: vec![],
            kontinue_stack: vec![],
            environment: Environment::new(),
            pc: 0,
            kontinue: 0,
            flag: Value::Nil,
            a: Value::Nil,
            b: Value::Nil,
            c: Value::Nil,
            d: Value::Nil,
            max_heap: 16,
            cars: vec![],
            cdrs: vec![],
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
        mem::swap(&mut new.constants, &mut self.constants);
        mem::swap(&mut new, self);
    }

    /// Sets the vm to print debug information.
    pub fn set_debug(&mut self) {
        self.debug = true;
    }

    fn print_debug(&mut self) {
        println!("step {}:", self.step);
        println!("a: {:?}", self.a);
        println!("b: {:?}", self.b);
        println!("c: {:?}", self.c);
        println!("d: {:?}", self.d);
        println!("flag: {:?}", self.flag);
        println!("continue: {:?}", self.kontinue);
        println!("stack size: {}", self.stack.len());
        println!("heap size: {}", self.cars.len());
        println!("max heap size: {}", self.max_heap);
        println!();
        self.step += 1;
    }

    /// Returns the current heap size. This correlates to the number of pairs currently allocated.
    pub fn heap_size(&self) -> usize {
        self.cars.len()
    }

    /// Returns the current stack size.
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Load code into the machine.
    pub fn load_code(&mut self, (code, constants): (Vec<Operation>, Vec<Value>)) {
        self.operations = code;
        self.constants = constants;
        self.pc = 0;
    }

    /// Set `register` to `value`.
    pub fn assign_register(&mut self, register: Register, value: Value) {
        match register {
            Register::Flag => self.flag = value,
            Register::A => self.a = value,
            Register::B => self.b = value,
            Register::C => self.c = value,
            Register::D => self.d = value,
        }
    }

    /// Get the value of `register`.
    pub fn load_register(&self, register: Register) -> &Value {
        match register {
            Register::Flag => &self.flag,
            Register::A => &self.a,
            Register::B => &self.b,
            Register::C => &self.c,
            Register::D => &self.d,
        }
    }

    pub fn assign_environment(&mut self, env: Environment) {
        self.environment = env;
    }

    /// Convert `symbol` to a Symbol.
    pub fn intern_symbol(&mut self, symbol: String) -> Sym {
        self.symbols.get_or_intern(symbol)
    }

    /// Get the string value of `symbol`.
    pub fn get_symbol_value(&self, symbol: Sym) -> &str {
        self.symbols.resolve(symbol).unwrap()
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
        match op.save_register() {
            Register::Flag => self.stack.push(self.flag.clone()),
            Register::A => self.stack.push(self.a.clone()),
            Register::B => self.stack.push(self.b.clone()),
            Register::C => self.stack.push(self.c.clone()),
            Register::D => self.stack.push(self.d.clone()),
        }
    }

    fn restore(&mut self, op: Operation) {
        assert!(!self.stack.is_empty());
        let value = self.stack.pop().unwrap();
        self.assign_register(op.restore_register(), value);
    }

    fn load_const(&mut self, op: Operation) {
        let index = op.loadconst_position();
        let constant = self.constants[index].clone();
        self.assign_register(op.loadconst_register(), constant);
    }

    fn make_closure(&mut self, op: Operation) {
        let constant_index = op.makeclosure_position() as usize;
        let mut lambda = self.constants[constant_index].clone().unwrap_lambda();
        // TODO extend env?
        lambda.set_env(self.environment.extend());
        self.assign_register(op.makeclosure_register(), Value::Lambda(lambda));
    }

    fn mov(&mut self, op: Operation) {
        let to = op.move_to();
        let from = op.move_from();
        match from {
            Register::Flag => self.assign_register(to, self.flag.clone()),
            Register::A => self.assign_register(to, self.a.clone()),
            Register::B => self.assign_register(to, self.b.clone()),
            Register::C => self.assign_register(to, self.c.clone()),
            Register::D => self.assign_register(to, self.d.clone()),
        }
    }

    fn goto(&mut self, op: Operation) {
        self._goto(op.goto_value());
    }

    fn goto_if(&mut self, op: Operation) {
        if Value::Bool(true) == *self.load_register(op.gotoif_register()) {
            self._goto(op.gotoif_value());
        }
    }

    fn goto_if_not(&mut self, op: Operation) {
        if Value::Bool(false) == *self.load_register(op.gotoifnot_register()) {
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
        let left = self.load_register(op.add_left());
        let right = self.load_register(op.add_right());
        self.assign_register(op.add_register(), left + right);
    }

    fn sub(&mut self, op: Operation) {
        let left = self.load_register(op.sub_left());
        let right = self.load_register(op.sub_right());
        self.assign_register(op.sub_register(), left - right);
    }

    fn mul(&mut self, op: Operation) {
        let left = self.load_register(op.mul_left());
        let right = self.load_register(op.mul_right());
        self.assign_register(op.mul_register(), left * right);
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
        let string = self.load_register(op.stringtosymbol_value()).clone().unwrap_string();
        let sym = self.intern_symbol(*string);
        self.assign_register(op.stringtosymbol_register(), Value::Symbol(sym));
    }

    fn cons(&mut self, op: Operation) {
        if self.cars.len() >= self.max_heap {
            self.collect_garbage();
        }

        let car = self.load_register(op.cons_car()).clone();
        let cdr = self.load_register(op.cons_cdr()).clone();
        self.cars.push(car);
        self.cdrs.push(cdr);

        let pointer = Value::Pair(self.cars.len() - 1);
        self.assign_register(op.cons_register(), pointer);
    }

    fn car(&mut self, op: Operation) {
        let pointer = self.load_register(op.car_from()).pair_pointer();
        assert!(pointer < self.cars.len());
        self.assign_register(op.car_to(), self.cars[pointer].clone());
    }

    fn cdr(&mut self, op: Operation) {
        let pointer = self.load_register(op.cdr_from()).pair_pointer();
        assert!(pointer < self.cdrs.len());
        self.assign_register(op.cdr_to(), self.cdrs[pointer].clone());
    }

    fn set_car(&mut self, op: Operation) {
        let pointer = self.load_register(op.setcar_register()).pair_pointer();
        assert!(pointer < self.cars.len());
        let value = self.load_register(op.setcar_value());
        self.cars[pointer] = value.clone();
    }

    fn set_cdr(&mut self, op: Operation) {
        let pointer = self.load_register(op.setcdr_register()).pair_pointer();
        assert!(pointer < self.cdrs.len());
        let value = self.load_register(op.setcdr_value());
        self.cdrs[pointer] = value.clone();
    }

    fn collect_garbage(&mut self) {
        if self.debug {
            println!("starting garbage collection with {} pairs", self.cars.len());
        }

        // We are using a stop-and-copy garbage collector. The idea is that when you
        // need to run the garbage collector, you stop execution of the program. You
        // go through all of the registers and the stack, looking for pointers into
        // the heap. You then copy all of the pairs pointed to by these objects into
        // backup memory, following any pointers as you go. We replace copied objects
        // with a (Broken Heart, new pointer) so that any objects which still point
        // here know the new location. In the process we compact memory. When we are
        // done, we set the backup memory as the active memory and continue execution.
        let mut cars = vec![];
        let mut cdrs = vec![];

        // Handle registers
        self.copy_pair_register(Register::Flag, &mut cars, &mut cdrs);
        self.copy_pair_register(Register::A, &mut cars, &mut cdrs);
        self.copy_pair_register(Register::B, &mut cars, &mut cdrs);
        self.copy_pair_register(Register::C, &mut cars, &mut cdrs);
        self.copy_pair_register(Register::D, &mut cars, &mut cdrs);

        // Handle stack
        for value in self.stack.clone() {
            if let Value::Pair(p) = value {
                self.copy_pair(p, &mut cars, &mut cdrs);
            }
        }

        // TODO: Handle environment

        mem::swap(&mut self.cars, &mut cars);
        mem::swap(&mut self.cdrs, &mut cdrs);
        self.max_heap *= 2;

        if self.debug {
            println!("ended garbage collection with {} pairs", self.cars.len());
        }
    }

    fn copy_pair_register(&mut self,
                          register: Register,
                          cars: &mut Vec<Value>,
                          cdrs: &mut Vec<Value>)
    {
        if let Value::Pair(p) = self.load_register(register) {
            let new_pointer = self.copy_pair(*p, cars, cdrs);
            self.assign_register(register, new_pointer);
        }
    }

    fn copy_pair(&mut self, pair: usize, cars: &mut Vec<Value>, cdrs: &mut Vec<Value>) -> Value {
        // This pointer has already been moved
        if self.cars[pair] == Value::BrokenHeart {
            self.cdrs[pair].clone()
        } else {
            let car = self.cars[pair].clone();
            let cdr = self.cdrs[pair].clone();
            let new_pointer = cars.len();
            cars.push(car.clone());
            cdrs.push(cdr.clone());
            self.cars[pair] = Value::BrokenHeart;
            self.cdrs[pair] = Value::Pair(new_pointer);

            if let Value::Pair(p) = car {
                let moved = self.copy_pair(p, cars, cdrs);
                cars[new_pointer] = moved;
            }

            if let Value::Pair(p) = cdr {
                let moved = self.copy_pair(p, cars, cdrs);
                cdrs[new_pointer] = moved;
            }
            Value::Pair(new_pointer)
        }
    }

    fn define(&mut self, op: Operation) {
        // TODO: handle error when name is not a symbol
        let name = self.load_register(op.define_name()).unwrap_symbol();
        let value = self.load_register(op.define_value()).clone();
        self.environment.define_variable(name, value);
    }

    fn lookup(&mut self, op: Operation) {
        // TODO: handle error when name is not a symbol
        let name = self.load_register(op.lookup_name()).unwrap_symbol();
        // TODO: we want an error if `name` is undefined
        let value = self.environment.lookup_variable_value(name).unwrap_or(Value::Void);
        self.assign_register(op.lookup_register(), value);
    }

    fn call(&mut self, op: Operation) {
        if self.debug {
            println!("beginning call");
        }

        if let Value::Lambda(lambda) = self.load_register(op.call_register()) {
            // Save the current code and env
            let mut code = lambda.code.clone();
            let mut env = lambda.environment.procedure_local();
            let mut consts = lambda.consts.clone();
            mem::swap(&mut code, &mut self.operations);
            mem::swap(&mut env, &mut self.environment);
            mem::swap(&mut consts, &mut self.constants);

            // Save the program counter
            let pc = self.pc;
            self.pc = 0;
            self.run();

            // Restore the saved program counter, code, and environment
            self.pc = pc;
            self.operations = code;
            self.environment = env;
            self.constants = consts;
        } else {
            // TODO: return error
        }

        if self.debug {
            println!("ending call");
        }
    }
}

use {Environment, Object};
use bytecode::*;
use stack::Stack;

pub struct ExecutionFrame {
    code_object: CodeObject,
    pc: usize,
    env: Environment,
}

pub struct Vm {
    value_stack: Stack<Object>,
    frame_stack: Stack<ExecutionFrame>,
    frame: ExecutionFrame,
    debug: bool,
}

impl Vm {
    fn get_variable(&self, index: usize) -> String {
        self.frame.code_object.varnames[index].clone()
    }

    // TODO
    fn get_const(&self, index: usize) -> Object {
        self.frame.code_object.constants[index]
    }

    pub fn run(&mut self, code: CodeObject) {
        self.frame.code_object = code;
        self.frame.pc = 0;

        loop {
            let instruction = if let Some(inst) = self.get_next_instruction() {
                inst
            } else {
                break;
            };

            if self.debug {
                // TODO: handle returning debug info
            }

            match instruction.opcode {
                OP_CONST => {
                    let constant = self.get_const(instruction.arg);
                    self.value_stack.push(constant);
                },
                OP_LOADVAR => {
                    let var = self.get_variable(instruction.arg);
                    let value = self.frame.env.lookup_var(&var);
                    self.value_stack.push(value);
                },
                OP_STOREVAR => {
                    let value = self.value_stack.pop();
                    let var = self.get_variable(instruction.arg);
                    self.frame.env.set_var_value(var, value);
                },
                OP_DEFVAR => {
                    let value = self.value_stack.pop();
                    let var = self.get_variable(instruction.arg);
                    self.frame.env.define_var(var, value);
                },
                OP_POP => {
                    if self.value_stack.len() > 0 {
                        self.value_stack.pop();
                    }
                },
                OP_JUMP => {
                    self.frame.pc = instruction.arg;
                },
                OP_FJUMP => {
                    let predicate = self.value_stack.pop();
                    if predicate.is_bool() && predicate.is_false() {
                        self.frame.pc = instruction.arg;
                    }
                },
                OP_FUNCTION => {
                    let func = self.get_const(instruction.arg);
                    func.env = self.frame.env;
                    self.value_stack.push(func);
                },
                OP_CALL => {
                    let procedure = self.value_stack.pop().unwrap().unwrap_closure();
                    let num_of_args = instruction.arg;
                    let mut arg_values = Vec::with_capacity(num_of_args);
                    for _ in 0..num_of_args {
                        arg_values.push(self.value_stack.pop());
                    }
                    arg_values.reverse();

                    if procedure.is_builtin() {
                        let result = procedure.apply(arg_values);
                        self.value_stack.push(result);
                    } else if procedure.is_closure() {
                        if procedure.code.args.len() != num_of_args {
                            // TODO
                            panic!("procedure expected more arguments");
                        }

                        self.frame_stack.push(self.frame);
                        let mut arg_bindings = Environment::extend(procedure.env);
                        for (i, arg) in procedure.code.args.iter().enumerate() {
                            arg_bindings.insert(arg, arg_values[i]);
                        }

                        self.frame = ExecutionFrame::new(procedure.code, 0, arg_bindings);
                    } else {
                        // TODO
                        panic!("invalid procedure object");
                    }
                },
                OP_RETURN => {
                    self.frame = self.frame_stack.pop().unwrap();
                },
                // TODO: return error
                _ => panic!("unknown instruction opcode: {}", instruction.opcode),
            }
        }
    }

    fn get_next_instruction(&self) -> Option<Instruction> {
        if self.frame.pc >= self.frame.code_object.code.len() {
            None
        } else {
            let instruction = self.frame.code_object.code[self.frame.pc];
            self.frame.pc += 1;
            Some(instruction)
        }
    }

    fn is_in_toplevel_code(&self) -> bool {
        self.frame_stack.len() == 0
    }
}

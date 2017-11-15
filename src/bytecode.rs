use Object;

type Op = u8;
const OP_CONST: Op = 0x00;
const OP_LOADVAR: Op = 0x10;
const OP_STOREVAR: Op = 0x11;
const OP_DEFVAR: Op = 0x12;
const OP_FUNCTION: Op = 0x20;
const OP_POP: Op = 0x30;
const OP_JUMP: Op = 0x40;
const OP_FJUMP: Op = 0x41;
const OP_RETURN: Op = 0x50;
const OP_CALL: Op = 0x51;

const STACK_SIZE: usize = 1<<20;

#[derive(Copy, Clone, Debug, Hash, PartialEq, PartialOrd, Eq)]
pub struct Instruction {
    crate opcode: Op,
    crate arg: usize,
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq)]
pub struct CodeObject {
    crate name: String,
    crate args: Vec<String>,
    crate code: Vec<Instruction>,
    crate constants: Vec<Object>,
    crate varnames: Vec<String>,
}

/*
pub struct Stack {
    stack: Vec<u64>,
    sp: usize,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            stack: vec![0; STACK_SIZE],
            sp: 0,
        }
    }
}

pub struct Heap {
    heap: Vec<u8>,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            heap: Vec::new(),
        }
    }
}

pub struct VM {
    pc: usize,
    stack: Stack,
    heap: Heap,
    instructions: Vec<Instruction>,
}

impl VM {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        VM {
            pc: 0,
            stack: Stack::new(),
            heap: Heap::new(),
            instructions: instructions,
        }
    }
}

use std::collections::HashMap;
pub struct Env {
    env: HashMap<u8, u8>,
    head: Option<Box<Env>>,
}
*/

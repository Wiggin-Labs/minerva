mod ir;

pub use self::ir::IR;

use vm::{ASM, GotoValue, Register, Value};

use string_interner::Symbol;

use std::collections::{HashMap, HashSet};

pub fn optimize(mut ir: Vec<IR>) -> Vec<IR> {
    optimize_lambda_formals(&mut ir);
    optimize_lookups(&mut ir);
    optimize_copies(&mut ir);
    optimize_dead_code(&mut ir);
    //optimize_tail_call(&mut ir);
    //optimize_recursion(&mut ir);
    ir
}

// TODO: make recursion jumps rather than calls
fn optimize_recursion(ir: &mut Vec<IR>) {
    fn inner(f: &mut IR) {
    }

    for i in ir.iter_mut() {
        if let IR::Fn(_, _, _) = *i {
            inner(i);
        }
    }
}

// TODO
fn optimize_tail_call(ir: &mut Vec<IR>) {
    fn inner(f: &mut IR) {
    }

    for i in ir.iter_mut() {
        if let IR::Fn(_, _, _) = *i {
            inner(i);
        }
    }
}

fn optimize_lambda_formals(ir: &mut Vec<IR>) {
    fn inner(f: &mut IR) {
        let (formals, ir) = if let IR::Fn(_, b, c) = f { (b, c) } else { unreachable!() };
        for i in ir.iter_mut() {
            match *i {
                IR::Fn(_, _, _) => inner(i),
                IR::Lookup(t, ident) => if formals.contains(&ident) {
                    *i = IR::Copy(t, ident);
                },
                _ => (),
            }
        }
    }

    for i in ir.iter_mut() {
        if let IR::Fn(_, _, _) = *i {
            inner(i);
        }
    }
}

fn optimize_lookups(ir: &mut Vec<IR>) {
    let mut lookups = HashMap::new();

    for i in ir.iter_mut() {
        match i {
            IR::Lookup(target, ident) => {
                // Shitty bc
                let ident = *ident;
                if let Some(t) = lookups.get(&ident) {
                    *i = IR::Copy(*target, *t);
                } else {
                    lookups.insert(ident, *target);
                }
            }
            IR::Fn(_, _, ir) => optimize_lookups(ir),
            _ => (),
        }
    }
}

fn optimize_dead_code(ir: &mut Vec<IR>) {
    let mut used = HashSet::new();
    for i in ir.iter().rev() {
        match i {
            IR::GotoIf(_, s) => { used.insert(*s); }
            IR::GotoIfNot(_, s) => { used.insert(*s); }
            //IR::Param(s) => { used.insert(*s); }
            IR::Return(s) => { used.insert(*s); }
            IR::Define(_, s) => { used.insert(*s); }
            IR::Call(_, s, args) => {
                used.insert(*s);
                for arg in args {
                    used.insert(*arg);
                }
            }
            // TODO
            IR::Phi(s1, s2, s3) => {
                used.insert(*s1);
                used.insert(*s2);
                used.insert(*s3);
            }
            _ => (),
        }
    }

    let mut idx = 0;
    while idx < ir.len() {
        match &mut ir[idx] {
            IR::Fn(s, _, ir) => if !used.contains(s) {
                ir.remove(idx);
                continue;
            } else {
                optimize_dead_code(ir);
            },
            IR::Primitive(s, _) => if !used.contains(s) {
                ir.remove(idx);
                continue;
            },
            IR::Lookup(s, _) => if !used.contains(s) {
                ir.remove(idx);
                continue;
            },
            _ => (),
        }
        idx += 1;
    }
}

fn optimize_copies(ir: &mut Vec<IR>) {
    let mut copies = HashMap::new();

    let mut idx = 0;
    while idx < ir.len() {
        match &mut ir[idx] {
            IR::Copy(target, s) => if let Some(&t) = copies.get(s) {
                copies.insert(*target, t);
                ir.remove(idx);
                continue;
            } else {
                copies.insert(*target, *s);
                ir.remove(idx);
                continue;
            },
            IR::Return(s) => if let Some(t) = copies.get(s) {
                ir[idx] = IR::Return(*t);
            },
            IR::GotoIf(a, s) => if let Some(t) = copies.get(s) {
                ir[idx] = IR::GotoIf(*a, *t);
            },
            IR::GotoIfNot(a, s) => if let Some(t) = copies.get(s) {
                ir[idx] = IR::GotoIfNot(*a, *t);
            },
            IR::Phi(a, s1, s2) => {
                let s1 = if let Some(t) = copies.get(s1) {
                    *t
                } else {
                    *s1
                };
                let s2 = if let Some(t) = copies.get(s2) {
                    *t
                } else {
                    *s2
                };
                ir[idx] = IR::Phi(*a, s1, s2);
            }
            IR::Define(b, s) => if let Some(t) = copies.get(s) {
                ir[idx] = IR::Define(*b, *t);
            },
            //IR::Param(s) => if let Some(t) = copies.get(s) {
            //    ir[idx] = IR::Param(*t);
            //},
            IR::Call(_, s, args) => {
                if let Some(t) = copies.get(s) {
                    *s = *t;
                }

                for arg in args {
                    if let Some(t) = copies.get(arg) {
                        *arg = *t;
                    }
                }
            }
            IR::Fn(_, _, ir) => optimize_copies(ir),
            _ => (),
        }
        idx += 1;
    }
}

pub fn output_asm(ir: Vec<IR>) -> Vec<ASM> {
    let mut output = Output {
        //var_reg: HashMap::new(),
        //var_stack: HashMap::new(),
        var_mapping: HashMap::new(),
        var_location: HashMap::new(),
        conflicts: HashMap::new(),
        used: HashMap::new(),
        live: HashMap::new(),
        registers: [None; 16],
        //stack: Vec::new(),
        stack: 0,
    };
    output._output_asm(ir, Register(0))
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum M {
    R(Register),
    S(usize),
}

struct Output {
    //var_reg: HashMap<Symbol, Register>,
    //var_stack: HashMap<Symbol, usize>,
    var_mapping: HashMap<Symbol, Register>,
    var_location: HashMap<Symbol, M>,
    conflicts: HashMap<Symbol, Symbol>,
    used: HashMap<Register, Symbol>,
    live: HashMap<Symbol, usize>,
    registers: [Option<Symbol>; 16],
    //stack: Vec<Symbol>,
    stack: usize,
}

impl Output {
    fn _output_asm(&mut self, ir: Vec<IR>, target: Register) -> Vec<ASM> {
        self.register_allocation(&ir, target);

        let mut asm = Vec::new();
        for i in ir {
            match i {
                IR::Primitive(s, v) => {
                    let r = self.get_register(s, &mut asm);
                    asm.push(ASM::LoadConst(r, v));
                },
                IR::Define(n, s2) => {
                    // TODO
                    let r = Register(17);
                    asm.push(ASM::LoadConst(r, Value::Symbol(n)));
                    let r2 = self.find_symbol(s2, &mut asm);
                    asm.push(ASM::Define(r, r2));
                }
                IR::Lookup(s, ident) => {
                    let r = self.get_register(s, &mut asm);
                    asm.push(ASM::LoadConst(r, Value::Symbol(ident)));
                    asm.push(ASM::Lookup(r, r));
                }
                IR::Call(s, proc, args) => {
                    // TODO: save used registers
                    for (i, arg) in args.iter().enumerate() {
                        self.load_symbol(*arg, Register(i as u8 + 1), &mut asm);
                    }

                    let r = self.find_symbol(proc, &mut asm);
                    asm.push(ASM::Call(r));
                    if Register(0) != self.lookup_register(s) {
                        asm.push(ASM::Move(self.lookup_register(s), Register(0)));
                    }
                    self.var_location.insert(s, M::R(self.lookup_register(s)));
                }
                IR::Fn(s, args, ir) => {
                    let mut output = Output {
                        //var_reg: HashMap::new(),
                        //var_stack: HashMap::new(),
                        var_mapping: HashMap::new(),
                        var_location: HashMap::new(),
                        conflicts: HashMap::new(),
                        used: HashMap::new(),
                        live: HashMap::new(),
                        registers: [None; 16],
                        //stack: Vec::new(),
                        stack: 0,
                    };
                    for (i, arg) in args.iter().enumerate() {
                        output.var_mapping.insert(*arg, Register(i as u8 + 1));
                        output.var_location.insert(*arg, M::R(Register(i as u8 + 1)));
                    }
                    let instructions = output._output_asm(ir, Register(0));
                    let r = self.get_register(s, &mut asm);
                    asm.push(ASM::MakeClosure(r, Box::new(instructions)));
                }
                IR::Label(s) => asm.push(ASM::Label(s)),
                IR::Goto(l) => asm.push(ASM::Goto(GotoValue::Label(l))),
                IR::GotoIf(l, s) => asm.push(ASM::GotoIf(GotoValue::Label(l), self.lookup_register(s))),
                IR::GotoIfNot(l, s) => asm.push(ASM::GotoIfNot(GotoValue::Label(l), self.lookup_register(s))),
                IR::Return(s) => {
                    if target != self.lookup_register(s) {
                        asm.push(ASM::Move(target, self.lookup_register(s)));
                    }
                    asm.push(ASM::Return);
                }
                // Not needed after register allocation
                IR::Phi(_, _, _) => (),
                //IR::Param(_) => (),
                // Only used for optimization
                IR::Copy(_, _) => unreachable!(),
            }
        }
        asm
    }

    fn liveness(&mut self, ir: &[IR]) {
        /*
        let mut liveness = HashMap::new();
        for (idx, i) in ir.iter().enumerate().rev() {
            match *i {
                IR::Phi(s, s1, s2) => {
                    if let Some(idx) = liveness.get(s) {
                        liveness.insert(s1, idx);
                        liveness.insert(s2, idx);
                    } else {
                        liveness.insert(s1, idx);
                        liveness.insert(s2, idx);
                    }
                }
                IR::Return(s) => if !liveness.contains(s) {
                    liveness.insert(s, idx);
                }
                _ => todo!(),
            }
        }
        */
    }

    fn register_allocation(&mut self, ir: &[IR], target: Register) {
        //let mut params = Vec::new();
        let mut phis = HashMap::new();
        // Iterate in reverse
        for (idx, i) in ir.iter().enumerate().rev() {
            match *i {
                IR::Call(s, proc, ref args) => {
                    self.live.entry(s).or_insert(idx);
                    //self.var_mapping.insert(s, target);
                    self.var_mapping.insert(proc, Register(0));
                    for (i, arg) in args.iter().enumerate() {
                        self.var_mapping.insert(*arg, Register(i as u8 + 1));
                        if !self.live.contains_key(arg) {
                            self.live.insert(s, idx);
                        }
                    }
                    //if args > 0 {
                    //    params.push((args as u8, idx));
                    //}
                }
                /*
                IR::Param(s) => {
                    assert!(params.len() > 0);
                    let (i, idx) = *params.last().unwrap();
                    self.var_mapping.insert(s, Register(i));
                    if !self.live.contains_key(&s) {
                        self.live.insert(s, idx);
                    }

                    if i == 1 {
                        params.pop();
                    } else {
                        params.last_mut().unwrap().0 -= 1;
                    }
                }
                */
                IR::Define(_, s) => if !self.live.contains_key(&s) {
                    self.live.insert(s, idx);
                    //self.conflicts.insert(s1, s2);
                    //self.conflicts.insert(s2, s1);
                    //if !self.var_mapping.contains_key(&s) {
                    //    self.var_mapping.insert(s, target);
                    //}
                }
                IR::Phi(s1, s2, s3) => {
                    if let Some(&r) = self.var_mapping.get(&s1) {
                        self.var_mapping.insert(s2, r);
                        self.var_mapping.insert(s3, r);
                    } else {
                        phis.insert(s2, s3);
                        phis.insert(s3, s2);
                    //    self.var_mapping.insert(s1, target);
                    //    self.var_mapping.insert(s2, target);
                    //    self.var_mapping.insert(s3, target);
                    }
                    if let Some(&i) = self.live.get(&s1) {
                        self.live.insert(s2, i);
                        self.live.insert(s3, i);
                    } else {
                        self.live.insert(s1, idx);
                        self.live.insert(s2, idx);
                        self.live.insert(s3, idx);
                    }
                }
                IR::Return(s) => {
                    self.var_mapping.insert(s, target);
                    self.live.entry(s).or_insert(idx);
                }
                IR::GotoIf(_, s) => self.live.entry(s).or_insert(idx),
                IR::GotoIfNot(_, s) => self.live.entry(s).or_insert(idx),
                IR::Goto(_) | IR::Label(_) => (),
                IR::Fn(s, _, _) => if !self.var_mapping.contains_key(&s) {
                    //self.var_mapping.insert(s, target);
                },
                IR::Primitive(s, _) => if !self.var_mapping.contains_key(&s) {
                    //self.var_mapping.insert(s, target);
                },
                IR::Lookup(s, _) => if !self.live.contains_key(&s) {
                    //self.var_mapping.insert(s, target);
                },
                // Only used for optimization
                IR::Copy(_, _) => unreachable!(),
            }
        }

        //assert!(params.is_empty());

        //let used = HashSet::new();
        /*
        for i in ir {
            match *i {
                IR::Call(s, _, _) => if !self.var_mapping.contains_key(&s) {
                    self.var_mapping.insert(s, target);
                    if let Some(&s2) = phis.get(&s) {
                        self.var_mapping.insert(s2, target);
                    }
                },
                IR::Fn(s, _, _) => if !self.var_mapping.contains_key(&s) {
                    self.var_mapping.insert(s, target);
                    if let Some(&s2) = phis.get(&s) {
                        self.var_mapping.insert(s2, target);
                    }
                },
                IR::Primitive(s, _) => if !self.var_mapping.contains_key(&s) {
                    self.var_mapping.insert(s, target);
                    if let Some(&s2) = phis.get(&s) {
                        self.var_mapping.insert(s2, target);
                    }
                },
                IR::Lookup(s, _) => if !self.var_mapping.contains_key(&s) {
                    self.var_mapping.insert(s, target);
                    if let Some(&s2) = phis.get(&s) {
                        self.var_mapping.insert(s2, target);
                    }
                },
                IR::Phi(s, s1, _) => { self.var_mapping.insert(s, *self.var_mapping.get(&s1).unwrap()); }
                // These variables must already have been mapped.
                IR::Define(_, _) => (),
                IR::Return(_) => (),
                IR::Param(_) => (),
                IR::Goto(_) | IR::GotoIf(_, _) | IR::GotoIfNot(_, _) | IR::Label(_) => (),
                // Only used for optimization
                IR::Copy(_, _) => unreachable!(),
            }
        }
        */
    }

    fn get_register(&mut self, s: Symbol, asm: &mut Vec<ASM>) -> Register {
        let r = self.lookup_register(s);
        if let Some(s) = self.used.get(&r) {
            self.var_location.insert(*s, M::S(self.stack));
            self.stack += 1;
            asm.push(ASM::Save(r));
        }
        self.used.insert(r, s);
        self.var_location.insert(s, M::R(r));
        r
    }

    fn load_symbol(&mut self, s: Symbol, target: Register, asm: &mut Vec<ASM>) {
        if *self.var_location.get(&s).unwrap() != M::R(target) {
            match self.var_location.get(&s).unwrap() {
                M::R(r) => {
                    asm.push(ASM::Move(target, *r));
                    self.var_location.insert(s, M::R(target));
                }
                M::S(l) => {
                    asm.push(ASM::ReadStack(target, self.stack-l));
                    self.var_location.insert(s, M::R(target));
                }
            }
        }
    }

    fn find_symbol(&mut self, s: Symbol, asm: &mut Vec<ASM>) -> Register {
        match self.var_location.get(&s).unwrap() {
            M::R(r) => *r,
            M::S(l) => {
                let target = Register(18);
                asm.push(ASM::ReadStack(target, self.stack-l));
                self.var_location.insert(s, M::R(target));
                target
            }
        }
    }

    fn lookup_register(&self, s: Symbol) -> Register {
        if let Some(&r) = self.var_mapping.get(&s) {
            r
        } else {
            Register(17)
        }
    }

    // Ok => Register
    // Err => Stack
    /*
    fn lookup(&self, s: Symbol) -> Option<Result<Register, usize>> {
        for (i, r) in self.registers.iter().enumerate() {
            if Some(s) == r {
                return Some(Ok(Register(i)));
            }
        }

        for (i, r) in self.stack.iter().rev().enumerate() {
            if s == r {
                return Some(Err(i));
            }
        }

        None
    }
    */
}

fn get_register(used: &HashSet<Register>) -> Option<Register> {
    for i in 0..16 {
        if !used.contains(&Register(i)) {
            return Some(Register(i));
        }
    }
    None
}

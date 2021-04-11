mod ir;

pub use self::ir::IR;

use vm::{ASM, Register};

use string_interner::Symbol;

use std::collections::HashMap;

pub fn optimize<T>(ir: Vec<IR<T>>) -> Vec<IR<T>> {
    let ir = optimize_lambda_formals(ir);
    let ir = optimize_lookups(ir);
    let ir = optimize_copies(ir);
    ir
}

fn optimize_lambda_formals<T>(ir: Vec<IR<T>>) -> Vec<IR<T>> {
    fn inner<T>(f: IR<T>) -> IR<T> {
        let (target, formals, ir) = if let IR::Fn(a, b, c) = f { (a, b, c) } else { unreachable!() };
        let mut new = Vec::new();
        for i in ir {
            match i {
                IR::Fn(_, _, _) => new.push(inner(i)),
                IR::Lookup(t, ident) => if formals.contains(&ident) {
                    new.push(IR::Copy(t, ident));
                } else {
                    new.push(i);
                },
                _ => new.push(i),
            }
        }
        IR::Fn(target, formals, new)
    }

    let mut new = Vec::new();
    for i in ir {
        match i {
            IR::Fn(_, _, _) => new.push(inner(i)),
            _ => new.push(i),
        }
    }
    new
}

fn optimize_lookups<T>(ir: Vec<IR<T>>) -> Vec<IR<T>> {
    let mut lookups = HashMap::new();

    let mut new = Vec::new();
    for i in ir {
        match i {
            IR::Lookup(target, ident) => if let Some(t) = lookups.get(&ident) {
                new.push(IR::Copy(target, *t));
            } else {
                lookups.insert(ident, target);
                new.push(i);
            },
            IR::Fn(a, b, ir) => new.push(IR::Fn(a, b, optimize_lookups(ir))),
            _ => new.push(i),
        }
    }
    new
}

fn optimize_copies<T>(ir: Vec<IR<T>>) -> Vec<IR<T>> {
    let mut copies = HashMap::new();

    let mut new = Vec::new();
    for i in ir {
        match i {
            IR::Copy(target, s) => if let Some(&t) = copies.get(&s) {
                copies.insert(target, t);
            } else {
                copies.insert(target, s);
            },
            IR::Return(s) => if let Some(t) = copies.get(&s) {
                new.push(IR::Return(*t));
            } else {
                new.push(i);
            },
            IR::GotoIf(a, s) => if let Some(t) = copies.get(&s) {
                new.push(IR::GotoIf(a, *t));
            } else {
                new.push(i);
            },
            IR::GotoIfNot(a, s) => if let Some(t) = copies.get(&s) {
                new.push(IR::GotoIfNot(a, *t));
            } else {
                new.push(i);
            },
            IR::Phi(a, s1, s2) => {
                let s1 = if let Some(t) = copies.get(&s1) {
                    *t
                } else {
                    s1
                };
                let s2 = if let Some(t) = copies.get(&s2) {
                    *t
                } else {
                    s2
                };
                new.push(IR::Phi(a, s1, s2));
            }
            IR::Define(a, b, s) => if let Some(t) = copies.get(&s) {
                new.push(IR::Define(a, b, *t));
            } else {
                new.push(i);
            },
            IR::Param(s) => if let Some(t) = copies.get(&s) {
                new.push(IR::Param(*t));
            } else {
                new.push(i);
            },
            IR::Call(a, s, b) => if let Some(t) = copies.get(&s) {
                new.push(IR::Call(a, *t, b));
            } else {
                new.push(i);
            },
            IR::Fn(a, b, ir) => new.push(IR::Fn(a, b, optimize_copies(ir))),
            _ => new.push(i),
        }
    }
    new
}

pub fn output_asm<T>(ir: Vec<IR<T>>) -> Vec<ASM<T>> {
    let mut output = Output {
        //var_reg: HashMap::new(),
        //var_stack: HashMap::new(),
        var_mapping: HashMap::new(),
    };
    output._output_asm(ir, Register(0))
}

struct Output {
    //var_reg: HashMap<Symbol, Register>,
    //var_stack: HashMap<Symbol, usize>,
    var_mapping: HashMap<Symbol, Register>,
}

impl Output {
    fn _output_asm<T>(&mut self, ir: Vec<IR<T>>, target: Register) -> Vec<ASM<T>> {
        /*
        let mut rev_ir = ir.clone();
        rev_ir.reverse();
        for i in rev_ir {
            match i {
                IR::Call(s, proc, args) => {
                    self.var_mapping.insert(s, target);
                    self.var_mapping.insert(proc, Register(0));
                    for (i, arg) in args.iter().enumerate() {
                        self.var_mapping.insert(*arg, Register(i as u8 + 1));
                    }
                }
                IR::Fn(s, _, _) => if !self.var_mapping.contains_key(&s) {
                    self.var_mapping.insert(s, target);
                },
                IR::Primitive(s1, _) => if !self.var_mapping.contains_key(&s1) {
                    self.var_mapping.insert(s1, target);
                },
                IR::BinOp(s, _, _, _) => if !self.var_mapping.contains_key(&s) {
                    self.var_mapping.insert(s, target);
                },
                IR::UnaryOp(s, _, _) => if !self.var_mapping.contains_key(&s) {
                    self.var_mapping.insert(s, target);
                },
                IR::Phi(s1, s2, s3) => if self.var_mapping.contains_key(&s1) {
                    let r = *self.var_mapping.get(&s1).unwrap();
                    self.var_mapping.insert(s2, r);
                    self.var_mapping.insert(s3, r);
                } else {
                    self.var_mapping.insert(s1, target);
                    self.var_mapping.insert(s2, target);
                    self.var_mapping.insert(s3, target);
                },
                _ => todo!(),
            }
        }
        */

        let mut asm = Vec::new();
        for i in ir {
            /*
            match i {
                IR::Copy(s1, s2) => write!(f, "COPY {}, {}", get_value(*s1).unwrap(), get_value(*s2).unwrap()),
                IR::Primitive(s, v) => write!(f, "PRIMITIVE {}, {}", get_value(*s).unwrap(), v),
                IR::BinOp(op, s1, s2) => write!(f, "{:?} {}, {}", op, get_value(*s1).unwrap(), get_value(*s2).unwrap()),
                IR::UnaryOp(op, s) => write!(f, "{:?} {}", op, get_value(*s).unwrap()),
                IR::Call(s, proc, args) => {
                    write!(f, "CALL {}, {}, (", get_value(*s).unwrap(), get_value(*proc).unwrap())?;
                    for arg in args {
                        write!(f, "{}, ", get_value(*arg).unwrap())?;
                    }
                    write!(f, ")")
                }
                IR::Fn(s, args, ir) => {
                    let mut output = Output {
                        var_reg: HashMap::new(),
                        var_stack: HashMap::new(),
                    };
                    for (i, arg) in args.iter().enumerate() {
                        var_reg.insert(arg, Register(i+1));
                    }
                    let instructions = output._compile(ir);
                }
                IR::Phi(s1, s2, s3) => write!(f, "PHI {}, {}, {}", get_value(*s1).unwrap(), get_value(*s2).unwrap(), get_value(*s3).unwrap()),
                IR::Label(s) => write!(f, "{}:", get_value(*s).unwrap()),
                IR::Goto(s) => write!(f, "GOTO {}", get_value(*s).unwrap()),
                IR::GotoIf(s1, s2) => write!(f, "GOTOIF {}, {}", get_value(*s1).unwrap(), get_value(*s2).unwrap()),
                IR::GotoIfNot(s1, s2) => write!(f, "GOTOIFNOT {}, {}", get_value(*s1).unwrap(), get_value(*s2).unwrap()),
                IR::Return(s) => write!(f, "RETURN {}", get_value(*s).unwrap()),
            }
            */
        }
        asm
    }
}

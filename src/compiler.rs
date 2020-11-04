use vm::{ASM, GotoValue, Register, Value};

use string_interner::INTERNER;

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

fn make_label() -> String {
    static LABEL: AtomicUsize = AtomicUsize::new(0);
    LABEL.fetch_add(1, Ordering::SeqCst).to_string()
}

fn get_register(used: &HashSet<Register>) -> Option<Register> {
    for i in 0..32 {
        if !used.contains(&Register(i)) {
            return Some(Register(i));
        }
    }
    None
}


pub fn compile(exp: Ast) -> Vec<ASM> {
    let mut used = HashSet::new();
    _compile(exp, Register(0), &mut used)
}

pub fn _compile(exp: Ast, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
    match exp {
        Ast::Primitive(p) => compile_self_evaluating(p, target, used),
        Ast::Ident(i) => compile_variable(i, target, used),
        Ast::Define { .. } => compile_define(exp, target, used),
        Ast::If { .. } => compile_if(exp, target, used),
        Ast::Begin(v) => compile_sequence(v, target, used),
        Ast::Lambda { .. } => compile_lambda(exp, target, used),
        Ast::Apply(v) => compile_application(v, target, used),
    }
}

fn compile_self_evaluating(p: CompilePrimitive, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
    used.insert(target);
    vec![ASM::LoadConst(target, p.to_value())]
}

fn compile_variable(i: String, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
    used.insert(target);
    vec![ASM::LoadConst(target, Value::Symbol(INTERNER.lock().unwrap().get_symbol(i))),
         ASM::Lookup(target, target)]
}

fn compile_define(exp: Ast, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
    let (name, value) = exp.unwrap_define();
    used.insert(target);
    let (savep, reg) = if let Some(reg) = get_register(used) {
        (false, reg)
    } else {
        (true, Register(1))
    };

    let mut value = _compile(value, reg, used);
    if savep { value.insert(0, ASM::Save(reg)); }

    value.push(ASM::LoadConst(target, Value::Symbol(INTERNER.lock().unwrap().get_symbol(name))));
    value.push(ASM::Define(target, reg));
    value.push(ASM::LoadConst(target, Value::Void));

    if savep { value.push(ASM::Restore(reg)); }

    value
}

fn compile_if(exp: Ast, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
    let alt_label = make_label();
    let after_if = make_label();

    let (savep, reg) = if used.contains(&Register(4)) {
        if let Some(reg) = get_register(used) {
            (false, reg)
        } else {
            (true, Register(4))
        }
    } else {
        (false, Register(4))
    };

    let (pred, cons, alt) = exp.unwrap_if();
    let mut pred = _compile(pred, reg, &mut used.clone());
    if savep { pred.insert(0, ASM::Save(reg)) }

    let mut cons = _compile(cons, target, &mut used.clone());
    let mut alt = _compile(alt, target, &mut used.clone());
    pred.push(ASM::GotoIfNot(GotoValue::Label(alt_label.clone()), reg));
    pred.append(&mut cons);
    pred.push(ASM::Goto(GotoValue::Label(after_if.clone())));
    pred.push(ASM::Label(alt_label));
    pred.append(&mut alt);
    pred.push(ASM::Label(after_if));

    if savep { pred.push(ASM::Restore(reg)) }

    pred
}

fn compile_sequence(v: Vec<Ast>, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
    let mut asm = vec![];
    for v in v {
        asm.append(&mut _compile(v, target, used));
    }
    asm
}

fn compile_lambda(exp: Ast, target: Register, _used: &mut HashSet<Register>) -> Vec<ASM> {
    let (args, body) = exp.unwrap_lambda();
    let mut instructions = vec![];

    let mut i = 1;
    for x in args {
        instructions.push(ASM::LoadConst(Register(0), Value::Symbol(INTERNER.lock().unwrap().get_symbol(x))));
        instructions.push(ASM::Define(Register(0), Register(i)));
        i += 1;
    }
    instructions.append(&mut compile_sequence(body, Register(0), &mut HashSet::new()));
    vec![ASM::MakeClosure(target, Box::new(instructions))]
}

fn compile_application(mut v: Vec<Ast>, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
    let op = v.remove(0);
    let mut instructions = vec![];
    let mut used = used.iter().collect::<Vec<_>>();
    for &&r in used.iter() {
        instructions.push(ASM::Save(r));
    }

    let mut i = 1;
    let mut u = HashSet::new();
    for v in v {
        instructions.append(&mut _compile(v, Register(i), &mut u));
        u.insert(Register(i));
        i += 1;
    }
    instructions.append(&mut _compile(op, Register(0), &mut u));
    instructions.push(ASM::Call(Register(0)));

    // Our calling convention requires result to be in A, so we can skip the move if that's where
    // we need it.
    if target != Register(0) {
        instructions.push(ASM::Move(target, Register(0)));
    }

    // We have to restore in reverse order
    used.reverse();
    for &r in used {
        instructions.push(ASM::Restore(r));
    }

    instructions
}

#[derive(Debug)]
pub enum Ast {
    Define {
        name: String,
        value: Box<Ast>,
    },
    Lambda {
        args: Vec<String>,
        body: Vec<Ast>,
    },
    If {
        predicate: Box<Ast>,
        consequent: Box<Ast>,
        alternative: Box<Ast>,
    },
    Begin(Vec<Ast>),
    Apply(Vec<Ast>),
    Ident(String),
    Primitive(CompilePrimitive),
}

impl Ast {
    fn unwrap_define(self) -> (String, Ast) {
        match self {
            Ast::Define { name, value } => (name, *value),
            _ => unreachable!(),
        }
    }

    fn unwrap_if(self) -> (Ast, Ast, Ast) {
        match self {
            Ast::If { predicate, consequent, alternative } =>
                (*predicate, *consequent, *alternative),
            _ => unreachable!(),
        }
    }

    fn unwrap_lambda(self) -> (Vec<String>, Vec<Ast>) {
        match self {
            Ast::Lambda { args, body } => (args, body),
            _ => unreachable!(),
        }
    }

    pub fn unwrap_begin(self) -> Vec<Ast> {
        match self {
            Ast::Begin(b) => b,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum CompilePrimitive {
    Nil,
    Bool(bool),
    Integer(i32),
    String(String),
}

impl CompilePrimitive {
    pub fn to_value(self) -> Value {
        use self::CompilePrimitive::*;
        match self {
            Nil => Value::Nil,
            Bool(b) => Value::Bool(b),
            Integer(i) => Value::Integer(i),
            String(s) => Value::String(s),
        }
    }
}

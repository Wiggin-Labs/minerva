use vm::{ASM, GotoValue, Register, Value};

use string_interner::{INTERNER, Symbol};

use std::collections::{HashMap, HashSet};
use std::mem;
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
    let mut c = Compiler {
        def: None,
        def_label: String::new(),
        last_expr: false,
        var_mapping: HashMap::new(),
    };
    c._compile(exp, Register(0), &mut used)
}

struct Compiler {
    def: Option<Symbol>,
    def_label: String,
    last_expr: bool,
    var_mapping: HashMap<String, u8>,
}

impl Compiler {
    fn _compile(&mut self, exp: Ast, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
        match exp {
            Ast::Primitive(p) => self.compile_self_evaluating(p, target, used),
            Ast::Ident(i) => self.compile_variable(i, target, used),
            Ast::Define { .. } => self.compile_define(exp, target, used),
            Ast::If { .. } => self.compile_if(exp, target, used),
            Ast::Begin(v) => self.compile_sequence(v, target, used),
            Ast::Lambda { .. } => self.compile_lambda(exp, target, used),
            Ast::Apply(v) => self.compile_application(v, target, used),
        }
    }

    fn compile_self_evaluating(&mut self, p: CompilePrimitive, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
        used.insert(target);
        vec![ASM::LoadConst(target, p.to_value())]
    }

    fn compile_variable(&mut self, i: String, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
        used.insert(target);
        if let Some(&r) = self.var_mapping.get(&i) {
            if target.0 != r {
                self.var_mapping.insert(i, target.0);
                vec![ASM::Move(target, Register(r))]
            } else {
                vec![]
            }
        } else {
            vec![ASM::LoadConst(target, Value::Symbol(INTERNER.lock().unwrap().get_symbol(i))),
                 ASM::Lookup(target, target)]
        }
    }

    fn compile_define(&mut self, exp: Ast, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
        let (name, value) = exp.unwrap_define();
        used.insert(target);
        let (savep, reg) = if let Some(reg) = get_register(used) {
            (false, reg)
        } else {
            (true, Register(1))
        };
        let name = INTERNER.lock().unwrap().get_symbol(name);
        let mut def_label = make_label();
        mem::swap(&mut self.def_label, &mut def_label);

        let mut s = Some(name);
        mem::swap(&mut self.def, &mut s);
        let l = self.last_expr;
        self.last_expr = true;

        let mut value = self._compile(value, reg, used);
        mem::swap(&mut self.def_label, &mut def_label);

        mem::swap(&mut self.def, &mut s);
        self.last_expr = l;
        if savep { value.insert(0, ASM::Save(reg)); }

        value.push(ASM::LoadConst(target, Value::Symbol(name)));
        value.push(ASM::Define(target, reg));
        value.push(ASM::LoadConst(target, Value::Void));

        if savep { value.push(ASM::Restore(reg)); }

        value
    }

    fn compile_if(&mut self, exp: Ast, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
        let alt_label = make_label();
        let after_if = make_label();

        let (pred, cons, alt) = exp.unwrap_if();
        let l = self.last_expr;
        self.last_expr = false;
        let mut pred = self._compile(pred, target, &mut used.clone());
        self.last_expr = l;

        let s = self.var_mapping.clone();
        let mut cons = self._compile(cons, target, &mut used.clone());
        self.var_mapping = s;
        let mut alt = self._compile(alt, target, &mut used.clone());
        pred.push(ASM::GotoIfNot(GotoValue::Label(alt_label.clone()), target));
        pred.append(&mut cons);
        pred.push(ASM::Goto(GotoValue::Label(after_if.clone())));
        pred.push(ASM::Label(alt_label));
        pred.append(&mut alt);
        pred.push(ASM::Label(after_if));

        pred
    }

    fn compile_sequence(&mut self, v: Vec<Ast>, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
        let mut asm = vec![];
        let size = v.len();
        let l = self.last_expr;
        self.last_expr = false;
        for (i, v) in v.into_iter().enumerate() {
            if i == size - 1 && l {
                self.last_expr = true;
            }
            asm.append(&mut self._compile(v, target, used));
        }
        self.last_expr = l;
        asm
    }

    fn compile_lambda(&mut self, exp: Ast, target: Register, _used: &mut HashSet<Register>) -> Vec<ASM> {
        let (args, body) = exp.unwrap_lambda();
        let mut instructions = vec![];
        if self.def.is_some() && self.last_expr {
            instructions.push(ASM::Label(self.def_label.clone()));
        }

        let mut loc = HashMap::new();
        let mut used = HashSet::new();
        for (i, x) in args.into_iter().enumerate() {
            loc.insert(x, i as u8 + 1);
            used.insert(Register(i as u8 + 1));
        }
        mem::swap(&mut self.var_mapping, &mut loc);
        instructions.append(&mut self.compile_sequence(body, Register(0), &mut used));
        mem::swap(&mut self.var_mapping, &mut loc);
        vec![ASM::MakeClosure(target, Box::new(instructions))]
    }

    fn compile_application(&mut self, mut v: Vec<Ast>, target: Register, used: &mut HashSet<Register>) -> Vec<ASM> {
        let op = v.remove(0);
        let mut instructions = vec![];
        let mut used = used.iter().collect::<Vec<_>>();
        for &&r in used.iter() {
            instructions.push(ASM::Save(r));
        }

        let mut i = 1;
        let mut u = HashSet::new();
        // Move any local variables first
        for v in &v {
            if let Ast::Ident(id) = v {
                if let Some(&r) = self.var_mapping.get(id) {
                    if r != i {
                        instructions.push(ASM::Move(Register(i), Register(r)));
                        self.var_mapping.insert(id.clone(), i);
                    }
                }
            }
            i += 1;
        }

        i = 1;
        for v in v {
            if let Ast::Ident(ref id) = v {
                if self.var_mapping.contains_key(id) {
                    i += 1;
                    continue;
                }
            }
            instructions.append(&mut self._compile(v, Register(i), &mut u));
            u.insert(Register(i));
            i += 1;
        }

        if let Ast::Ident(ref i) = op {
            let s = INTERNER.lock().unwrap().get_symbol(i.clone());
            if let Some(d) = self.def {
                if d == s && self.last_expr {
                    instructions.push(ASM::Goto(GotoValue::Label(self.def_label.clone())));
                } else {
                    instructions.append(&mut self._compile(op, Register(0), &mut u));
                    instructions.push(ASM::Call(Register(0)));
                }
            } else {
                instructions.append(&mut self._compile(op, Register(0), &mut u));
                instructions.push(ASM::Call(Register(0)));
            }
        } else {
            instructions.append(&mut self._compile(op, Register(0), &mut u));
            instructions.push(ASM::Call(Register(0)));
        }

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

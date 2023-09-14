use {Ast, IR};

use vm::Value;

use string_interner::{get_symbol, Symbol};

use std::sync::atomic::{AtomicUsize, Ordering};

fn make_label() -> Symbol {
    static LABEL: AtomicUsize = AtomicUsize::new(0);
    let l = LABEL.fetch_add(1, Ordering::SeqCst).to_string();
    get_symbol(l)
}

fn gen_var() -> Symbol {
    static VAR: AtomicUsize = AtomicUsize::new(0);
    let l = format!("x{}", VAR.fetch_add(1, Ordering::SeqCst));
    get_symbol(l)
}

pub fn compile(exp: Ast) -> Vec<IR> {
    let mut c = Compiler;
    let target = gen_var();
    let mut ir = c._compile(exp, target);
    ir.push(IR::Return(target));
    ir
}

struct Compiler;

impl Compiler {
    fn _compile(&mut self, exp: Ast, target: Symbol) -> Vec<IR> {
        match exp {
            Ast::Primitive(p) => self.compile_self_evaluating(p, target),
            Ast::Ident(i) => self.compile_variable(i, target),
            Ast::Define { .. } => self.compile_define(exp, target),
            Ast::If { .. } => self.compile_if(exp, target),
            Ast::Begin(v) => self.compile_sequence(v, target),
            Ast::Lambda { .. } => self.compile_lambda(exp, target),
            Ast::Apply(v) => self.compile_application(v, target),
        }
    }

    fn compile_self_evaluating(&mut self, p: Value, target: Symbol) -> Vec<IR> {
        vec![IR::Primitive(target, p)]
    }

    fn compile_variable(&mut self, i: Symbol, target: Symbol) -> Vec<IR> {
        vec![IR::Lookup(target, i)]
    }

    fn compile_define(&mut self, exp: Ast, target: Symbol) -> Vec<IR> {
        let (name, value) = exp.unwrap_define();
        let mut n = self._compile(value, target);
        n.push(IR::Define(name, target));
        n
    }

    fn compile_if(&mut self, exp: Ast, target: Symbol) -> Vec<IR> {
        let alt_label = make_label();
        let after_if = make_label();

        let (pred, cons, alt) = exp.unwrap_if();
        let pred_var = gen_var();
        let mut pred = self._compile(pred, pred_var);
        pred.push(IR::GotoIfNot(alt_label, pred_var));

        let cons_var = gen_var();
        let mut cons = self._compile(cons, cons_var);
        cons.push(IR::Move(target, cons_var));
        cons.push(IR::Goto(after_if));

        let alt_var = gen_var();
        let mut alt = self._compile(alt, alt_var);
        cons.insert(0, IR::Label(alt_label));
        alt.push(IR::Move(target, alt_var));

        pred.push(IR::Label(after_if));
        pred.push(IR::Phi(target, cons_var, cons, alt_var, alt));
        pred
        /*
        let (pred, cons, alt) = exp.unwrap_if();
        let pred_var = gen_var();
        let mut pred = self._compile(pred, pred_var);
        pred.push(IR::GotoIfNot(alt_label, pred_var));

        let cons_var = gen_var();
        let mut cons = self._compile(cons, cons_var);
        pred.append(&mut cons);
        pred.push(IR::Move(target, cons_var));
        pred.push(IR::Goto(after_if));

        pred.push(IR::Label(alt_label));
        let alt_var = gen_var();
        let mut alt = self._compile(alt, alt_var);
        pred.append(&mut alt);
        pred.push(IR::Move(target, alt_var));

        pred.push(IR::Label(after_if));
        pred.push(IR::Phi(target, cons_var, alt_var));
        pred
        */
    }

    fn compile_sequence(&mut self, v: Vec<Ast>, target: Symbol) -> Vec<IR> {
        let mut ir = Vec::new();
        let size = v.len();
        for (i, v) in v.into_iter().enumerate() {
            if i == size - 1 {
                ir.append(&mut self._compile(v, target));
            } else {
                ir.append(&mut self._compile(v, gen_var()));
            }
        }

        ir
    }

    fn compile_lambda(&mut self, exp: Ast, target: Symbol) -> Vec<IR> {
        let (args, body) = exp.unwrap_lambda();
        let ret = gen_var();
        let mut body = self.compile_sequence(body, ret);
        body.push(IR::Return(ret));
        vec![IR::Fn(target, args, body)]
    }

    fn compile_application(&mut self, mut v: Vec<Ast>, target: Symbol) -> Vec<IR> {
        let op = v.remove(0);
        //let args = v.len();
        let mut ir = Vec::new();
        let mut args = Vec::new();
        for arg in v {
            let arg_symbol = gen_var();
            ir.append(&mut self._compile(arg, arg_symbol));
            args.push(arg_symbol);
            //ir.push(IR::Param(arg_symbol));
        }
        let op_symbol = gen_var();
        ir.append(&mut self._compile(op, op_symbol));
        ir.push(IR::Call(target, op_symbol, args));
        ir
    }
}

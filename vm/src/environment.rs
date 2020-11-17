use {assemble, ASM, Register, Value, VM};

use string_interner::Symbol;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn init_env(vm: &mut VM) -> Environment {
    let env = Environment::new();

    let add = vec![ASM::Add(Register(0), Register(1), Register(2))];
    add_primitive(vm, &env, "+".to_string(), add);

    let sub = vec![ASM::Sub(Register(0), Register(1), Register(2))];
    add_primitive(vm, &env, "-".to_string(), sub);

    let mul = vec![ASM::Mul(Register(0), Register(1), Register(2))];
    add_primitive(vm, &env, "*".to_string(), mul);

    let eq = vec![ASM::Eq(Register(0), Register(1), Register(2))];
    add_primitive(vm, &env, "=".to_string(), eq);

    let lt = vec![ASM::LT(Register(0), Register(1), Register(2))];
    add_primitive(vm, &env, "<".to_string(), lt);

    let cons = vec![ASM::Cons(Register(0), Register(1), Register(2))];
    add_primitive(vm, &env, "cons".to_string(), cons);
    let car = vec![ASM::Car(Register(0), Register(1))];
    add_primitive(vm, &env, "car".to_string(), car);
    let cdr = vec![ASM::Cdr(Register(0), Register(1))];
    add_primitive(vm, &env, "cdr".to_string(), cdr);

    env.define_variable(vm.intern_symbol("pi".to_string()), Value::Float(std::f64::consts::PI));
    env.define_variable(vm.intern_symbol("e".to_string()), Value::Float(std::f64::consts::E));

    env
}

fn add_primitive(vm: &mut VM, env: &Environment, name: String, code: Vec<ASM>) {
    let code = assemble(code);
    // TODO: gc, arity
    env.define_variable(vm.intern_symbol(name), Value::Lambda(env.clone(), 0, code));
}

#[derive(Clone, Default, PartialEq)]
pub struct Environment {
    env: Rc<RefCell<_Environment>>,
}

impl ::std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "<Environment>")
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            env: Rc::new(RefCell::new(_Environment::new())),
        }
    }

    pub fn from_hashmap(map: HashMap<Symbol, Value>) -> Self {
        let env = _Environment {
            bindings: map,
            parent: None,
        };

        Environment {
            env: Rc::new(RefCell::new(env)),
        }
    }

    pub fn extend(&self) -> Self {
        let mut env = _Environment::new();
        env.parent = Some(self.clone());
        Environment {
            env: Rc::new(RefCell::new(env)),
        }
    }

    pub fn lookup_variable_value(&self, name: Symbol) -> Option<Value> {
        self.env.borrow().lookup_variable_value(name)
    }

    pub fn define_variable(&self, name: Symbol, value: Value) {
        self.env.borrow_mut().define_variable(name, value);
    }

    pub fn set_variable_value(&self, name: Symbol, value: Value) -> Value {
        self.env.borrow_mut().set_variable_value(name, value)
    }

    pub fn procedure_local(&self) -> Self {
        let env = self.env.borrow();
        let local = _Environment {
            bindings: env.bindings.clone(),
            parent: env.parent.clone(),
        };
        Environment {
            env: Rc::new(RefCell::new(local)),
        }
    }

    pub fn get_definitions(&self) -> Vec<Symbol> {
        self.env.borrow().get_definitions()
    }
}

#[derive(Default)]
pub struct _Environment {
    bindings: HashMap<Symbol, Value>,
    parent: Option<Environment>,
}

impl PartialEq for _Environment {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl _Environment {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn lookup_variable_value(&self, name: Symbol) -> Option<Value> {
        if let Some(val) = self.bindings.get(&name) {
            Some(val.clone())
        } else if let Some(ref env) = self.parent {
            env.lookup_variable_value(name)
        } else {
            None
        }
    }

    pub fn define_variable(&mut self, name: Symbol, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn set_variable_value(&mut self, name: Symbol, value: Value) -> Value {
        if self.bindings.contains_key(&name) {
            self.bindings.insert(name, value);
            Value::Void
        } else if let Some(ref env) = self.parent {
            env.set_variable_value(name, value)
        } else {
            //Sexp::Error(Error::UnboundVariable(name))
            panic!("");
        }
    }

    pub fn get_definitions(&self) -> Vec<Symbol> {
        let mut definitions: Vec<_> = self.bindings.keys().map(|x| *x).collect();
        if let Some(ref env) = self.parent {
            definitions.append(&mut env.get_definitions());
        }
        definitions
    }
}

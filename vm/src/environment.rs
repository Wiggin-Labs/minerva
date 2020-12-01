use Value;

use string_interner::Symbol;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, PartialEq)]
pub struct Environment<T> {
    env: Rc<RefCell<_Environment<T>>>,
}

impl<T> Clone for Environment<T> {
    fn clone(&self) -> Self {
        Environment {
            env: Rc::clone(&self.env),
        }
    }
}

impl<T> ::std::fmt::Debug for Environment<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "<Environment>")
    }
}

impl<T> Environment<T> {
    pub fn new() -> Self {
        Environment {
            env: Rc::new(RefCell::new(_Environment::new())),
        }
    }

    pub fn from_hashmap(map: HashMap<Symbol, Value<T>>) -> Self {
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

    pub fn lookup_variable_value(&self, name: Symbol) -> Option<Value<T>> {
        self.env.borrow().lookup_variable_value(name)
    }

    pub fn define_variable(&self, name: Symbol, value: Value<T>) {
        self.env.borrow_mut().define_variable(name, value);
    }

    pub fn set_variable_value(&self, name: Symbol, value: Value<T>) -> Value<T> {
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

    pub(crate) fn mark(&self) {
        self.env.borrow().mark()
    }
}

pub struct _Environment<T> {
    bindings: HashMap<Symbol, Value<T>>,
    parent: Option<Environment<T>>,
}

impl<T> Default for _Environment<T> {
    fn default() -> Self {
        _Environment {
            bindings: HashMap::new(),
            parent: None,
        }
    }
}

impl<T> PartialEq for _Environment<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<T> _Environment<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn lookup_variable_value(&self, name: Symbol) -> Option<Value<T>> {
        if let Some(val) = self.bindings.get(&name) {
            Some(val.clone())
        } else if let Some(ref env) = self.parent {
            env.lookup_variable_value(name)
        } else {
            None
        }
    }

    pub fn define_variable(&mut self, name: Symbol, value: Value<T>) {
        self.bindings.insert(name, value);
    }

    pub fn set_variable_value(&mut self, name: Symbol, value: Value<T>) -> Value<T> {
        if self.bindings.contains_key(&name) {
            self.bindings.insert(name, value);
            Value::Void
        } else if let Some(ref env) = self.parent {
            env.set_variable_value(name, value)
        } else {
            self.define_variable(name, value);
            value
        }
    }

    pub fn get_definitions(&self) -> Vec<Symbol> {
        let mut definitions: Vec<_> = self.bindings.keys().map(|x| *x).collect();
        if let Some(ref env) = self.parent {
            definitions.append(&mut env.get_definitions());
        }
        definitions
    }

    pub(crate) fn mark(&self) {
        for v in self.bindings.values() {
            v.mark();
        }
        if let Some(ref env) = self.parent {
            env.mark()
        }
    }
}

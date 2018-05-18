use Error;
use object::{Arity, Object, Primitive};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

macro_rules! init_env {
    ($($key:expr),*) => {
        hashmap!{$(
            $key.0.to_string() =>
                Object::cons(Object::Symbol("primitive".to_string()),
                             Object::cons(
                                 Object::Primitive(Primitive::new($key.0.to_string(), $key.1)),
                                 Object::Nil)),
        )*}
    };
}

pub fn init_env() -> Environment {
    let bindings = init_env!{
        ("eval", Arity::Exactly(1)),
        ("apply", Arity::Exactly(2)),
        ("cons", Arity::Exactly(2)),
        ("null?", Arity::Exactly(1)),
        ("car", Arity::Exactly(1)),
        ("cdr", Arity::Exactly(1)),
        ("set-car!", Arity::Exactly(2)),
        ("set-cdr!", Arity::Exactly(2)),
        ("=", Arity::AtLeast(0)),
        ("+", Arity::AtLeast(0)),
        ("-", Arity::AtLeast(1)),
        ("*", Arity::AtLeast(0)),
        ("/", Arity::AtLeast(1))
    };
    Environment::from_hashmap(bindings)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    env: Rc<RefCell<_Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            env: Rc::new(RefCell::new(_Environment::new())),
        }
    }

    pub fn from_hashmap(map: HashMap<String, Object>) -> Self {
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

    pub fn lookup_variable_value(&self, name: &str) -> Option<Object> {
        self.env.borrow().lookup_variable_value(name)
    }

    pub fn define_variable(&self, name: String, value: Object) {
        self.env.borrow_mut().define_variable(name, value);
    }

    pub fn set_variable_value(&self, name: String, value: Object) -> Object {
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
}

#[derive(Debug, Default)]
pub struct _Environment {
    bindings: HashMap<String, Object>,
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

    pub fn lookup_variable_value(&self, name: &str) -> Option<Object> {
        if let Some(val) = self.bindings.get(name) {
            Some(val.clone())
        } else if let Some(ref env) = self.parent {
            env.lookup_variable_value(name)
        } else {
            None
        }
    }

    pub fn define_variable(&mut self, name: String, value: Object) {
        self.bindings.insert(name, value);
    }

    pub fn set_variable_value(&mut self, name: String, value: Object) -> Object {
        if self.bindings.contains_key(&name) {
            self.bindings.insert(name, value);
            Object::Void
        } else if let Some(ref env) = self.parent {
            env.set_variable_value(name, value)
        } else {
            Object::Error(Error::UnboundVariable(name))
        }
    }
}

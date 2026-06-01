use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::runtime::Value;

pub type ScopeRef = Rc<RefCell<Scope>>;

pub struct Scope {
    parent_scope: Option<ScopeRef>,
    store: HashMap<String, Value>,
}

impl Scope {
    pub fn new(parent_scope: Option<ScopeRef>) -> Scope {
        Scope {
            parent_scope,
            store: HashMap::new(),
        }
    }
    pub fn set(&mut self, name: &str, value: Value) {
        self.store.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.store.get(name) {
            return Some(val.clone());
        }

        self.parent_scope
            .as_ref()
            .and_then(|parent| parent.borrow().get(name))
    }

    pub fn new_child(parent: &ScopeRef) -> ScopeRef {
        Rc::new(RefCell::new(Self {
            parent_scope: Some(Rc::clone(parent)),
            store: HashMap::new(),
        }))
    }
}

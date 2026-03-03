use std::collections::HashMap;

use crate::value::Value;

/// A variable environment with lexical scoping
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    pub fn set(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    /// Assign to an existing variable in any scope (walks up from innermost).
    /// If not found, sets in current scope (like `set`).
    pub fn assign(&mut self, name: String, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name, value);
                return;
            }
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        let mut env = Environment::new();
        env.set("x".into(), Value::Integer(42));
        assert!(matches!(env.get("x"), Some(Value::Integer(42))));
    }

    #[test]
    fn scoping() {
        let mut env = Environment::new();
        env.set("x".into(), Value::Integer(1));
        env.push_scope();
        env.set("x".into(), Value::Integer(2));
        assert!(matches!(env.get("x"), Some(Value::Integer(2))));
        env.pop_scope();
        assert!(matches!(env.get("x"), Some(Value::Integer(1))));
    }

    #[test]
    fn undefined_variable() {
        let env = Environment::new();
        assert!(env.get("undefined").is_none());
    }
}

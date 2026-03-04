use std::collections::{HashMap, HashSet};

use crate::value::Value;

/// A variable environment with lexical scoping
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
    frozen: HashSet<String>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            frozen: HashSet::new(),
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

    /// Set a variable and mark it as frozen (immutable).
    pub fn set_frozen(&mut self, name: String, value: Value) {
        self.set(name.clone(), value);
        self.frozen.insert(name);
    }

    /// Assign to an existing variable in any scope (walks up from innermost).
    /// If not found, sets in current scope (like `set`).
    /// Returns Err if the variable is frozen (let binding).
    pub fn assign(&mut self, name: String, value: Value) -> Result<(), String> {
        if self.frozen.contains(&name) {
            return Err(format!("cannot reassign immutable binding '{}'", name));
        }
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name, value);
                return Ok(());
            }
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
        Ok(())
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Create a deep copy of this environment for closure capture
    pub fn snapshot(&self) -> Self {
        Self {
            scopes: self.scopes.clone(),
            frozen: self.frozen.clone(),
        }
    }

    /// Get all bindings in the current (innermost) scope
    pub fn current_scope_bindings(&self) -> HashMap<String, Value> {
        self.scopes.last().cloned().unwrap_or_default()
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

    #[test]
    fn frozen_prevents_reassign() {
        let mut env = Environment::new();
        env.set_frozen("x".into(), Value::Integer(42));
        let result = env.assign("x".into(), Value::Integer(99));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot reassign"));
    }

    #[test]
    fn frozen_allows_read() {
        let mut env = Environment::new();
        env.set_frozen("x".into(), Value::Integer(42));
        assert!(matches!(env.get("x"), Some(Value::Integer(42))));
    }
}

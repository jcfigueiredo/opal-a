use std::fmt;

use crate::closure::ClosureId;
use crate::function::FunctionId;

/// A runtime value in the Opal interpreter
#[derive(Debug, Clone)]
pub enum Value {
    /// Integer (64-bit signed)
    Integer(i64),
    /// Float (64-bit)
    Float(f64),
    /// String
    String(String),
    /// Boolean
    Bool(bool),
    /// Null
    Null,
    /// User-defined function (ID into the interpreter's function table)
    Function(FunctionId),
    /// List of values
    List(Vec<Value>),
    /// Closure (ID into interpreter's closure table)
    Closure(ClosureId),
    /// Class
    Class(ClassId),
    /// Instance of a class
    Instance(InstanceId),
    /// Module
    Module(ModuleId),
    /// Result Ok variant
    Ok(Box<Value>),
    /// Result Error variant
    Error(Box<Value>),
    /// Option Some variant
    Some(Box<Value>),
    /// Symbol: `:name`
    Symbol(String),
    /// Actor instance
    Actor(ActorId),
    /// AST node (ID into interpreter's AST storage)
    Ast(AstId),
    /// Opaque native object (FFI state)
    NativeObject(NativeObjectId),
    /// Native function from an extern block (ID into interpreter's native_functions table)
    NativeFunction(NativeFunctionId),
    /// Dict: ordered key-value pairs
    Dict(Vec<(String, Value)>),
    /// Range: start..end (exclusive) or start...end (inclusive)
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
}

/// Opaque ID for a stored AST node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AstId(pub usize);

/// Opaque ID for an actor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorId(pub usize);

/// Opaque ID for a class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassId(pub usize);

/// Opaque ID for an instance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstanceId(pub usize);

/// Opaque ID for a module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleId(pub usize);

/// Opaque ID for a native object (FFI state)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeObjectId(pub usize);

/// Opaque ID for a native function (FFI dispatch)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeFunctionId(pub usize);

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Function(id) => write!(f, "<function #{}>", id.0),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Closure(id) => write!(f, "<closure #{}>", id.0),
            Value::Class(id) => write!(f, "<class #{}>", id.0),
            Value::Instance(id) => write!(f, "<instance #{}>", id.0),
            Value::Module(id) => write!(f, "<module #{}>", id.0),
            Value::Ok(v) => write!(f, "Ok({})", v),
            Value::Error(v) => write!(f, "Error({})", v),
            Value::Some(v) => write!(f, "Some({})", v),
            Value::Symbol(s) => write!(f, ":{}", s),
            Value::Actor(id) => write!(f, "<actor #{}>", id.0),
            Value::Ast(id) => write!(f, "<ast #{}>", id.0),
            Value::NativeObject(id) => write!(f, "<native #{}>", id.0),
            Value::NativeFunction(id) => write!(f, "<native fn #{}>", id.0),
            Value::Dict(entries) => {
                write!(f, "{{")?;
                for (i, (key, value)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                if *inclusive {
                    write!(f, "{}...{}", start, end)
                } else {
                    write!(f, "{}..{}", start, end)
                }
            }
        }
    }
}

impl Value {
    /// Check if value is truthy (everything except false and null)
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Bool(false) | Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_values() {
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Float(1.23).to_string(), "1.23");
        assert_eq!(Value::String("hello".into()).to_string(), "hello");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Null.to_string(), "null");
    }

    #[test]
    fn truthiness() {
        assert!(Value::Bool(true).is_truthy());
        assert!(Value::Integer(0).is_truthy());
        assert!(Value::String("".into()).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(!Value::Null.is_truthy());
    }
}

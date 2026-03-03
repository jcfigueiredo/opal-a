use std::fmt;

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
}

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
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
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

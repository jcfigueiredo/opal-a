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
    /// Multiple dispatch group (several functions with the same name, different arities)
    MultiFunction(Vec<FunctionId>),
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
    /// Symbol: `:name`
    Symbol(String),
    /// Actor instance
    Actor(ActorId),
    /// Actor class (definition, not instance)
    ActorClass(ActorDefId),
    /// AST node (ID into interpreter's AST storage)
    Ast(AstId),
    /// Opaque native object (FFI state)
    NativeObject(NativeObjectId),
    /// Native function from an extern block (ID into interpreter's native_functions table)
    NativeFunction(NativeFunctionId),
    /// Macro (ID into interpreter's macro table)
    Macro(MacroId),
    /// Protocol (ID into interpreter's protocol table)
    Protocol(ProtocolId),
    /// Type object (returned by typeof)
    Type(TypeInfo),
    /// Enum variant value
    EnumVariant {
        enum_id: EnumId,
        variant_index: usize,
        fields: Vec<Value>,
    },
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

/// Opaque ID for an actor definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorDefId(pub usize);

/// Opaque ID for a class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassId(pub usize);

/// Opaque ID for an instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Opaque ID for a macro definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacroId(pub usize);

/// Opaque ID for a protocol definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolId(pub usize);

/// Opaque ID for an enum definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumId(pub usize);

/// Well-known builtin enum IDs and variant indices.
/// These match the registration order in `register_builtin_enums()`.
pub const RESULT_ENUM_ID: EnumId = EnumId(0);
pub const RESULT_ERROR_VARIANT: usize = 1;
pub const OPTION_ENUM_ID: EnumId = EnumId(1);

/// Built-in type identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinType {
    Int,
    Float,
    String,
    Bool,
    Null,
    Symbol,
    List,
    Dict,
    Range,
    Fn,
}

/// Runtime type information
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeInfo {
    Builtin(BuiltinType),
    Alias(String),
    Class(ClassId),
    Protocol(ProtocolId),
    Enum(EnumId),
    EnumVariant(EnumId, usize),
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
            Value::Function(id) => write!(f, "<function #{}>", id.0),
            Value::MultiFunction(ids) => write!(f, "<function ({} variants)>", ids.len()),
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
            Value::Symbol(s) => write!(f, ":{}", s),
            Value::Actor(id) => write!(f, "<actor #{}>", id.0),
            Value::ActorClass(id) => write!(f, "<actor class #{}>", id.0),
            Value::Ast(id) => write!(f, "<ast #{}>", id.0),
            Value::NativeObject(id) => write!(f, "<native #{}>", id.0),
            Value::NativeFunction(id) => write!(f, "<native fn #{}>", id.0),
            Value::Macro(id) => write!(f, "<macro #{}>", id.0),
            Value::Protocol(id) => write!(f, "<protocol #{}>", id.0),
            Value::Type(info) => match info {
                TypeInfo::Builtin(b) => write!(
                    f,
                    "{}",
                    match b {
                        BuiltinType::Int => "Int",
                        BuiltinType::Float => "Float",
                        BuiltinType::String => "String",
                        BuiltinType::Bool => "Bool",
                        BuiltinType::Null => "Null",
                        BuiltinType::Symbol => "Symbol",
                        BuiltinType::List => "List",
                        BuiltinType::Dict => "Dict",
                        BuiltinType::Range => "Range",
                        BuiltinType::Fn => "Fn",
                    }
                ),
                TypeInfo::Alias(name) => write!(f, "{}", name),
                TypeInfo::Class(id) => write!(f, "<type class #{}>", id.0),
                TypeInfo::Protocol(id) => write!(f, "<type protocol #{}>", id.0),
                TypeInfo::Enum(id) => write!(f, "<type enum #{}>", id.0),
                TypeInfo::EnumVariant(id, v) => write!(f, "<type enum #{} variant {}>", id.0, v),
            },
            Value::EnumVariant {
                enum_id,
                variant_index,
                fields,
            } => {
                write!(f, "<enum #{}.{}", enum_id.0, variant_index)?;
                if !fields.is_empty() {
                    write!(f, "(")?;
                    for (i, v) in fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", v)?;
                    }
                    write!(f, ")")?;
                }
                write!(f, ">")
            }
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
        match self {
            Value::Bool(false) | Value::Null => false,
            // Legacy Result::Error variant is falsy
            Value::EnumVariant {
                enum_id,
                variant_index,
                ..
            } if *enum_id == RESULT_ENUM_ID && *variant_index == RESULT_ERROR_VARIANT => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_primitives() {
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Float(1.0).to_string(), "1.0");
        assert_eq!(Value::Float(1.23).to_string(), "1.23");
        assert_eq!(Value::String("hello".into()).to_string(), "hello");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
        assert_eq!(Value::Null.to_string(), "null");
    }

    #[test]
    fn display_ids() {
        assert_eq!(Value::Function(FunctionId(0)).to_string(), "<function #0>");
        assert_eq!(
            Value::MultiFunction(vec![FunctionId(0), FunctionId(1)]).to_string(),
            "<function (2 variants)>"
        );
        assert_eq!(Value::Closure(ClosureId(3)).to_string(), "<closure #3>");
        assert_eq!(Value::Class(ClassId(1)).to_string(), "<class #1>");
        assert_eq!(Value::Instance(InstanceId(5)).to_string(), "<instance #5>");
        assert_eq!(Value::Module(ModuleId(0)).to_string(), "<module #0>");
        assert_eq!(Value::Actor(ActorId(2)).to_string(), "<actor #2>");
        assert_eq!(
            Value::ActorClass(ActorDefId(0)).to_string(),
            "<actor class #0>"
        );
        assert_eq!(Value::Ast(AstId(0)).to_string(), "<ast #0>");
        assert_eq!(
            Value::NativeObject(NativeObjectId(0)).to_string(),
            "<native #0>"
        );
        assert_eq!(
            Value::NativeFunction(NativeFunctionId(0)).to_string(),
            "<native fn #0>"
        );
        assert_eq!(Value::Macro(MacroId(0)).to_string(), "<macro #0>");
        assert_eq!(Value::Protocol(ProtocolId(0)).to_string(), "<protocol #0>");
    }

    #[test]
    fn display_symbol() {
        assert_eq!(Value::Symbol("ok".into()).to_string(), ":ok");
    }

    #[test]
    fn display_collections() {
        // List
        assert_eq!(Value::List(vec![]).to_string(), "[]");
        assert_eq!(
            Value::List(vec![Value::Integer(1), Value::Integer(2)]).to_string(),
            "[1, 2]"
        );
        // Dict
        assert_eq!(
            Value::Dict(vec![
                ("a".into(), Value::Integer(1)),
                ("b".into(), Value::Integer(2)),
            ])
            .to_string(),
            "{a: 1, b: 2}"
        );
        // Range
        assert_eq!(
            Value::Range {
                start: 1,
                end: 10,
                inclusive: false
            }
            .to_string(),
            "1..10"
        );
        assert_eq!(
            Value::Range {
                start: 1,
                end: 10,
                inclusive: true
            }
            .to_string(),
            "1...10"
        );
    }

    #[test]
    fn display_enum_variant() {
        // No fields
        assert_eq!(
            Value::EnumVariant {
                enum_id: EnumId(2),
                variant_index: 0,
                fields: vec![],
            }
            .to_string(),
            "<enum #2.0>"
        );
        // With fields
        assert_eq!(
            Value::EnumVariant {
                enum_id: EnumId(0),
                variant_index: 1,
                fields: vec![Value::String("err".into())],
            }
            .to_string(),
            "<enum #0.1(err)>"
        );
    }

    #[test]
    fn display_type_info() {
        assert_eq!(
            Value::Type(TypeInfo::Builtin(BuiltinType::Int)).to_string(),
            "Int"
        );
        assert_eq!(
            Value::Type(TypeInfo::Builtin(BuiltinType::Float)).to_string(),
            "Float"
        );
        assert_eq!(
            Value::Type(TypeInfo::Builtin(BuiltinType::String)).to_string(),
            "String"
        );
        assert_eq!(
            Value::Type(TypeInfo::Builtin(BuiltinType::Bool)).to_string(),
            "Bool"
        );
        assert_eq!(
            Value::Type(TypeInfo::Builtin(BuiltinType::Null)).to_string(),
            "Null"
        );
        assert_eq!(
            Value::Type(TypeInfo::Builtin(BuiltinType::Symbol)).to_string(),
            "Symbol"
        );
        assert_eq!(
            Value::Type(TypeInfo::Builtin(BuiltinType::List)).to_string(),
            "List"
        );
        assert_eq!(
            Value::Type(TypeInfo::Builtin(BuiltinType::Dict)).to_string(),
            "Dict"
        );
        assert_eq!(
            Value::Type(TypeInfo::Builtin(BuiltinType::Range)).to_string(),
            "Range"
        );
        assert_eq!(
            Value::Type(TypeInfo::Builtin(BuiltinType::Fn)).to_string(),
            "Fn"
        );
        assert_eq!(
            Value::Type(TypeInfo::Alias("MyType".into())).to_string(),
            "MyType"
        );
        assert_eq!(
            Value::Type(TypeInfo::Class(ClassId(0))).to_string(),
            "<type class #0>"
        );
        assert_eq!(
            Value::Type(TypeInfo::Protocol(ProtocolId(0))).to_string(),
            "<type protocol #0>"
        );
        assert_eq!(
            Value::Type(TypeInfo::Enum(EnumId(0))).to_string(),
            "<type enum #0>"
        );
        assert_eq!(
            Value::Type(TypeInfo::EnumVariant(EnumId(0), 1)).to_string(),
            "<type enum #0 variant 1>"
        );
    }

    #[test]
    fn truthiness() {
        assert!(Value::Bool(true).is_truthy());
        assert!(Value::Integer(0).is_truthy());
        assert!(Value::String("".into()).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(!Value::Null.is_truthy());

        // Legacy Result::Error variant should be falsy
        let error_val = Value::EnumVariant {
            enum_id: RESULT_ENUM_ID,
            variant_index: RESULT_ERROR_VARIANT,
            fields: vec![Value::String("test error".into())],
        };
        assert!(!error_val.is_truthy(), "Error should be falsy");

        // Result::Ok should still be truthy
        let ok_val = Value::EnumVariant {
            enum_id: RESULT_ENUM_ID,
            variant_index: 0,
            fields: vec![Value::Integer(42)],
        };
        assert!(ok_val.is_truthy(), "Ok should be truthy");

        // Other enum variants should remain truthy
        let some_val = Value::EnumVariant {
            enum_id: OPTION_ENUM_ID,
            variant_index: 0,
            fields: vec![Value::Integer(42)],
        };
        assert!(some_val.is_truthy(), "Other enum variants should be truthy");
    }
}

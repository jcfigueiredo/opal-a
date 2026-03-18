pub mod http;
pub mod plugin;

pub use plugin::{NativeFunction, PluginRegistry};

use opal_runtime::Value;

/// Result of calling a builtin function
pub enum BuiltinResult {
    Value(Value),
    Void,
}

/// Call a builtin function by name. Returns None if the name is not a builtin.
pub fn call_builtin(
    name: &str,
    args: &[Value],
    writer: &mut dyn std::io::Write,
) -> Option<Result<BuiltinResult, String>> {
    match name {
        "print" => Some(builtin_print(args, writer)),
        "println" => Some(builtin_println(args, writer)),
        _ => None,
    }
}

fn builtin_print(args: &[Value], writer: &mut dyn std::io::Write) -> Result<BuiltinResult, String> {
    let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    writeln!(writer, "{}", output.join(" ")).map_err(|e| e.to_string())?;
    Ok(BuiltinResult::Void)
}

fn builtin_println(
    args: &[Value],
    writer: &mut dyn std::io::Write,
) -> Result<BuiltinResult, String> {
    let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    writeln!(writer, "{}", output.join(" ")).map_err(|e| e.to_string())?;
    Ok(BuiltinResult::Void)
}

/// Get list of builtin function names
pub fn builtin_names() -> &'static [&'static str] {
    &["print", "println"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_print() {
        let mut buf = Vec::new();
        let result = call_builtin("print", &[Value::String("hello".into())], &mut buf);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), Ok(BuiltinResult::Void)));
        assert_eq!(String::from_utf8(buf).unwrap(), "hello\n");
    }

    #[test]
    fn call_println() {
        let mut buf = Vec::new();
        let result = call_builtin("println", &[Value::Integer(42)], &mut buf);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), Ok(BuiltinResult::Void)));
        assert_eq!(String::from_utf8(buf).unwrap(), "42\n");
    }

    #[test]
    fn call_print_multiple_args() {
        let mut buf = Vec::new();
        let args = vec![Value::String("a".into()), Value::Integer(1)];
        call_builtin("print", &args, &mut buf).unwrap().unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "a 1\n");
    }

    #[test]
    fn call_unknown_builtin_returns_none() {
        let mut buf = Vec::new();
        assert!(call_builtin("nonexistent", &[], &mut buf).is_none());
    }

    #[test]
    fn builtin_names_returns_expected() {
        let names = builtin_names();
        assert!(names.contains(&"print"));
        assert!(names.contains(&"println"));
        assert_eq!(names.len(), 2);
    }
}

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

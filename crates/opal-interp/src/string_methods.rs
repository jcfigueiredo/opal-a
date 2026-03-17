use std::io::Write;

use opal_runtime::Value;

use crate::eval::{EvalError, Interpreter, PanicKind};

impl<W: Write> Interpreter<W> {
    pub(crate) fn call_string_method(
        &mut self,
        s: &str,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, EvalError> {
        match method {
            "length" => Ok(Value::Integer(s.len() as i64)),
            "split" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "split() takes exactly 1 argument".into(),
                    ));
                }
                let sep = match &args[0] {
                    Value::String(sep) => sep.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "split() argument must be a string".into(),
                        ));
                    }
                };
                let parts: Vec<Value> = s
                    .split(&sep)
                    .map(|p| Value::String(p.to_string()))
                    .collect();
                Ok(Value::List(parts))
            }
            "trim" => Ok(Value::String(s.trim().to_string())),
            "contains" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "contains() takes exactly 1 argument".into(),
                    ));
                }
                let sub = match &args[0] {
                    Value::String(sub) => sub.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "contains() argument must be a string".into(),
                        ));
                    }
                };
                Ok(Value::Bool(s.contains(&sub)))
            }
            "replace" => {
                if args.len() != 2 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "replace() takes exactly 2 arguments".into(),
                    ));
                }
                let old = match &args[0] {
                    Value::String(o) => o.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "replace() first argument must be a string".into(),
                        ));
                    }
                };
                let new = match &args[1] {
                    Value::String(n) => n.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "replace() second argument must be a string".into(),
                        ));
                    }
                };
                Ok(Value::String(s.replace(&old, &new)))
            }
            "starts_with" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "starts_with() takes exactly 1 argument".into(),
                    ));
                }
                let prefix = match &args[0] {
                    Value::String(p) => p.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "starts_with() argument must be a string".into(),
                        ));
                    }
                };
                Ok(Value::Bool(s.starts_with(&prefix)))
            }
            "ends_with" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "ends_with() takes exactly 1 argument".into(),
                    ));
                }
                let suffix = match &args[0] {
                    Value::String(sf) => sf.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "ends_with() argument must be a string".into(),
                        ));
                    }
                };
                Ok(Value::Bool(s.ends_with(&suffix)))
            }
            "to_upper" => Ok(Value::String(s.to_uppercase())),
            "to_lower" => Ok(Value::String(s.to_lowercase())),
            "chars" => {
                let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
                Ok(Value::List(chars))
            }
            "to_int" => match s.trim().parse::<i64>() {
                Ok(n) => Ok(Value::Integer(n)),
                Err(_) => Ok(Value::Null),
            },
            "to_float" => match s.trim().parse::<f64>() {
                Ok(n) => Ok(Value::Float(n)),
                Err(_) => Ok(Value::Null),
            },
            "reverse" => Ok(Value::String(s.chars().rev().collect())),
            "upcase" => Ok(Value::String(s.to_uppercase())),
            "downcase" => Ok(Value::String(s.to_lowercase())),
            "slice" => {
                if args.len() != 2 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "slice() takes exactly 2 arguments (start, end)".into(),
                    ));
                }
                let start = match &args[0] {
                    Value::Integer(n) => *n as usize,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "slice() start must be an integer".into(),
                        ));
                    }
                };
                let end = match &args[1] {
                    Value::Integer(n) => *n as usize,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "slice() end must be an integer".into(),
                        ));
                    }
                };
                let result: String = s
                    .chars()
                    .skip(start)
                    .take(end.saturating_sub(start))
                    .collect();
                Ok(Value::String(result))
            }
            "index" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "index() takes exactly 1 argument".into(),
                    ));
                }
                let substr = match &args[0] {
                    Value::String(sub) => sub.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "index() argument must be a string".into(),
                        ));
                    }
                };
                match s.find(&substr) {
                    Some(byte_pos) => {
                        // Convert byte position to char position for Unicode safety
                        let char_pos = s[..byte_pos].chars().count();
                        Ok(Value::Integer(char_pos as i64))
                    }
                    None => Ok(Value::Null),
                }
            }
            "empty?" => Ok(Value::Bool(s.is_empty())),
            _ => Err(EvalError::Panic(
                PanicKind::TypeError,
                format!("no method '{}' on String", method),
            )),
        }
    }
}

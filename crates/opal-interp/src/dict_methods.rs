use std::io::Write;

use opal_runtime::Value;

use crate::eval::{EvalError, Interpreter, PanicKind};

impl<W: Write> Interpreter<W> {
    pub(crate) fn call_dict_method(
        &mut self,
        entries: &[(String, Value)],
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, EvalError> {
        match method {
            "length" => Ok(Value::Integer(entries.len() as i64)),
            "get" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "get() takes exactly 1 argument".into(),
                    ));
                }
                let key = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "dict key must be a string".into(),
                        ));
                    }
                };
                Ok(entries
                    .iter()
                    .find(|(k, _)| k == &key)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Value::Null))
            }
            "keys" => {
                let keys: Vec<Value> = entries
                    .iter()
                    .map(|(k, _)| Value::String(k.clone()))
                    .collect();
                Ok(Value::List(keys))
            }
            "values" => {
                let values: Vec<Value> = entries.iter().map(|(_, v)| v.clone()).collect();
                Ok(Value::List(values))
            }
            "set" => {
                if args.len() != 2 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "set() takes exactly 2 arguments (key, value)".into(),
                    ));
                }
                let key = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "dict key must be a string".into(),
                        ));
                    }
                };
                let value = args[1].clone();
                let mut new_entries = entries.to_vec();
                if let Some(entry) = new_entries.iter_mut().find(|(k, _)| k == &key) {
                    entry.1 = value;
                } else {
                    new_entries.push((key, value));
                }
                Ok(Value::Dict(new_entries))
            }
            "has_key" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "has_key() takes exactly 1 argument".into(),
                    ));
                }
                let key = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "has_key() argument must be a string".into(),
                        ));
                    }
                };
                Ok(Value::Bool(entries.iter().any(|(k, _)| k == &key)))
            }
            "merge" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "merge() takes exactly 1 argument".into(),
                    ));
                }
                let other = match &args[0] {
                    Value::Dict(d) => d.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "merge() argument must be a dict".into(),
                        ));
                    }
                };
                let mut merged = entries.to_vec();
                for (key, value) in other {
                    if let Some(entry) = merged.iter_mut().find(|(k, _)| k == &key) {
                        entry.1 = value;
                    } else {
                        merged.push((key, value));
                    }
                }
                Ok(Value::Dict(merged))
            }
            _ => Err(EvalError::Panic(
                PanicKind::TypeError,
                format!("no method '{}' on Dict", method),
            )),
        }
    }
}

use std::io::Write;

use opal_runtime::Value;

use crate::eval::{EvalError, Interpreter, PanicKind, value_compare, values_equal};

impl<W: Write> Interpreter<W> {
    pub(crate) fn call_list_method(
        &mut self,
        items: &[Value],
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, EvalError> {
        match method {
            "length" => Ok(Value::Integer(items.len() as i64)),
            "push" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "push() takes exactly 1 argument".into(),
                    ));
                }
                let mut new_list = items.to_vec();
                new_list.push(args.into_iter().next().unwrap());
                Ok(Value::List(new_list))
            }
            "get" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "get() takes exactly 1 argument".into(),
                    ));
                }
                match &args[0] {
                    Value::Integer(idx) => {
                        let idx = *idx as usize;
                        Ok(items.get(idx).cloned().unwrap_or(Value::Null))
                    }
                    _ => Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "list index must be an integer".into(),
                    )),
                }
            }
            "map" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "map() takes exactly 1 argument (a closure)".into(),
                    ));
                }
                let closure_id = match &args[0] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "map() argument must be a closure".into(),
                        ));
                    }
                };
                let mut result = Vec::new();
                for item in items.iter().cloned() {
                    result.push(self.call_closure(closure_id, vec![item])?);
                }
                Ok(Value::List(result))
            }
            "filter" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "filter() takes exactly 1 argument (a closure)".into(),
                    ));
                }
                let closure_id = match &args[0] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "filter() argument must be a closure".into(),
                        ));
                    }
                };
                let mut result = Vec::new();
                for item in items.iter().cloned() {
                    let keep = self.call_closure(closure_id, vec![item.clone()])?;
                    if keep.is_truthy() {
                        result.push(item);
                    }
                }
                Ok(Value::List(result))
            }
            "reduce" => {
                if args.len() != 2 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "reduce() takes 2 arguments (initial, closure)".into(),
                    ));
                }
                let initial = args[0].clone();
                let closure_id = match &args[1] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "reduce() second argument must be a closure".into(),
                        ));
                    }
                };
                let mut acc = initial;
                for item in items.iter().cloned() {
                    acc = self.call_closure(closure_id, vec![acc, item])?;
                }
                Ok(acc)
            }
            "sort" => {
                if args.is_empty() {
                    // Natural sort
                    let mut sorted = items.to_vec();
                    sorted.sort_by(value_compare);
                    Ok(Value::List(sorted))
                } else if args.len() == 1 {
                    // Custom comparator
                    let closure_id = match &args[0] {
                        Value::Closure(id) => *id,
                        _ => {
                            return Err(EvalError::Panic(
                                PanicKind::TypeError,
                                "sort() argument must be a closure".into(),
                            ));
                        }
                    };
                    let mut sorted = items.to_vec();
                    let mut sort_error: Option<EvalError> = None;
                    sorted.sort_by(|a, b| {
                        if sort_error.is_some() {
                            return std::cmp::Ordering::Equal;
                        }
                        match self.call_closure(closure_id, vec![a.clone(), b.clone()]) {
                            Ok(Value::Integer(n)) => {
                                if n < 0 {
                                    std::cmp::Ordering::Less
                                } else if n > 0 {
                                    std::cmp::Ordering::Greater
                                } else {
                                    std::cmp::Ordering::Equal
                                }
                            }
                            Ok(_) => {
                                sort_error = Some(EvalError::Panic(
                                    PanicKind::TypeError,
                                    "sort comparator must return an integer".into(),
                                ));
                                std::cmp::Ordering::Equal
                            }
                            Err(e) => {
                                sort_error = Some(e);
                                std::cmp::Ordering::Equal
                            }
                        }
                    });
                    if let Some(e) = sort_error {
                        return Err(e);
                    }
                    Ok(Value::List(sorted))
                } else {
                    Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "sort() takes 0 or 1 arguments".into(),
                    ))
                }
            }
            "reverse" => {
                let mut reversed = items.to_vec();
                reversed.reverse();
                Ok(Value::List(reversed))
            }
            "find" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "find() takes exactly 1 argument (a closure)".into(),
                    ));
                }
                let closure_id = match &args[0] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "find() argument must be a closure".into(),
                        ));
                    }
                };
                for item in items.iter().cloned() {
                    let result = self.call_closure(closure_id, vec![item.clone()])?;
                    if result.is_truthy() {
                        return Ok(item);
                    }
                }
                Ok(Value::Null)
            }
            "any" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "any() takes exactly 1 argument (a closure)".into(),
                    ));
                }
                let closure_id = match &args[0] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "any() argument must be a closure".into(),
                        ));
                    }
                };
                for item in items.iter().cloned() {
                    let result = self.call_closure(closure_id, vec![item])?;
                    if result.is_truthy() {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }
            "all" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "all() takes exactly 1 argument (a closure)".into(),
                    ));
                }
                let closure_id = match &args[0] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "all() argument must be a closure".into(),
                        ));
                    }
                };
                for item in items.iter().cloned() {
                    let result = self.call_closure(closure_id, vec![item])?;
                    if !result.is_truthy() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            "count" => {
                if args.is_empty() {
                    Ok(Value::Integer(items.len() as i64))
                } else if args.len() == 1 {
                    let closure_id = match &args[0] {
                        Value::Closure(id) => *id,
                        _ => {
                            return Err(EvalError::Panic(
                                PanicKind::TypeError,
                                "count() argument must be a closure".into(),
                            ));
                        }
                    };
                    let mut count = 0i64;
                    for item in items.iter().cloned() {
                        let result = self.call_closure(closure_id, vec![item])?;
                        if result.is_truthy() {
                            count += 1;
                        }
                    }
                    Ok(Value::Integer(count))
                } else {
                    Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "count() takes 0 or 1 arguments".into(),
                    ))
                }
            }
            "each" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "each() takes exactly 1 argument (a closure)".into(),
                    ));
                }
                let closure_id = match &args[0] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "each() argument must be a closure".into(),
                        ));
                    }
                };
                for item in items.iter().cloned() {
                    self.call_closure(closure_id, vec![item])?;
                }
                Ok(Value::Null)
            }
            "take" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "take() takes exactly 1 argument".into(),
                    ));
                }
                let n = match &args[0] {
                    Value::Integer(n) => *n as usize,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "take() argument must be an integer".into(),
                        ));
                    }
                };
                let taken: Vec<Value> = items.iter().take(n).cloned().collect();
                Ok(Value::List(taken))
            }
            "drop" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "drop() takes exactly 1 argument".into(),
                    ));
                }
                let n = match &args[0] {
                    Value::Integer(n) => *n as usize,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "drop() argument must be an integer".into(),
                        ));
                    }
                };
                let remaining: Vec<Value> = items.iter().skip(n).cloned().collect();
                Ok(Value::List(remaining))
            }
            "flatten" => {
                let mut result = Vec::new();
                for item in items {
                    match item {
                        Value::List(inner) => result.extend(inner.clone()),
                        other => result.push(other.clone()),
                    }
                }
                Ok(Value::List(result))
            }
            "zip" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "zip() takes exactly 1 argument".into(),
                    ));
                }
                let other = match &args[0] {
                    Value::List(l) => l.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "zip() argument must be a list".into(),
                        ));
                    }
                };
                let pairs: Vec<Value> = items
                    .iter()
                    .zip(other.iter())
                    .map(|(a, b)| Value::List(vec![a.clone(), b.clone()]))
                    .collect();
                Ok(Value::List(pairs))
            }
            "group_by" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "group_by() takes exactly 1 argument (a closure)".into(),
                    ));
                }
                let closure_id = match &args[0] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "group_by() argument must be a closure".into(),
                        ));
                    }
                };
                let mut groups: Vec<(String, Value)> = Vec::new();
                for item in items.iter().cloned() {
                    let key_val = self.call_closure(closure_id, vec![item.clone()])?;
                    let key = self.format_value(&key_val);
                    if let Some(entry) = groups.iter_mut().find(|(k, _)| k == &key) {
                        if let Value::List(ref mut list) = entry.1 {
                            list.push(item);
                        }
                    } else {
                        groups.push((key, Value::List(vec![item])));
                    }
                }
                Ok(Value::Dict(groups))
            }
            "join" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "join() takes exactly 1 argument".into(),
                    ));
                }
                let sep = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(EvalError::Panic(
                            PanicKind::TypeError,
                            "join() argument must be a string".into(),
                        ));
                    }
                };
                let parts: Vec<String> = items.iter().map(|v| self.format_value(v)).collect();
                Ok(Value::String(parts.join(&sep)))
            }
            "empty?" => Ok(Value::Bool(items.is_empty())),
            "contains" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "contains() takes exactly 1 argument".into(),
                    ));
                }
                let needle = &args[0];
                Ok(Value::Bool(
                    items.iter().any(|item| values_equal(item, needle)),
                ))
            }
            "first" => Ok(items.first().cloned().unwrap_or(Value::Null)),
            "last" => Ok(items.last().cloned().unwrap_or(Value::Null)),
            "min" => {
                if items.is_empty() {
                    return Ok(Value::Null);
                }
                let mut min_val = items[0].clone();
                for item in &items[1..] {
                    if value_compare(item, &min_val) == std::cmp::Ordering::Less {
                        min_val = item.clone();
                    }
                }
                Ok(min_val)
            }
            "max" => {
                if items.is_empty() {
                    return Ok(Value::Null);
                }
                let mut max_val = items[0].clone();
                for item in &items[1..] {
                    if value_compare(item, &max_val) == std::cmp::Ordering::Greater {
                        max_val = item.clone();
                    }
                }
                Ok(max_val)
            }
            "index" => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "index() takes exactly 1 argument".into(),
                    ));
                }
                let needle = &args[0];
                match items.iter().position(|item| values_equal(item, needle)) {
                    Some(pos) => Ok(Value::Integer(pos as i64)),
                    None => Ok(Value::Null),
                }
            }
            _ => Err(EvalError::Panic(
                PanicKind::TypeError,
                format!("no method '{}' on List", method),
            )),
        }
    }
}

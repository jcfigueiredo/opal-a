use std::collections::HashMap;
use std::io::Write;

use opal_runtime::{ClassId, InstanceId, Value};

use crate::eval::{EvalError, FlowSignal, Interpreter, PanicKind, StoredInstance, Visibility};

impl<W: Write> Interpreter<W> {
    pub(crate) fn call_instance_method(
        &mut self,
        instance_id: &InstanceId,
        method: &str,
        args: Vec<Value>,
        named_args: &[(Option<String>, Value)],
    ) -> Result<Value, EvalError> {
        let instance = self.instances[instance_id.0].clone();
        let class = self.classes[instance.class_id.0].clone();

        // Container built-in methods
        if class.name == "Container" {
            match method {
                "register" => {
                    let proto_name = match &args[0] {
                        Value::Protocol(pid) => self.protocols[pid.0].name.clone(),
                        Value::Class(cid) => self.classes[cid.0].name.clone(),
                        _ => {
                            return Err(EvalError::Panic(
                                PanicKind::TypeError,
                                "register() first arg must be a Protocol or Class".into(),
                            ));
                        }
                    };
                    self.container_registrations
                        .entry(*instance_id)
                        .or_default()
                        .insert(proto_name, args[1].clone());
                    return Ok(Value::Null);
                }
                "resolve" => {
                    let (target_class_id, target_class) = match &args[0] {
                        Value::Class(cid) => (*cid, self.classes[cid.0].clone()),
                        _ => {
                            return Err(EvalError::Panic(
                                PanicKind::TypeError,
                                "resolve() arg must be a Class".into(),
                            ));
                        }
                    };
                    let regs = self
                        .container_registrations
                        .get(instance_id)
                        .cloned()
                        .unwrap_or_default();

                    let mut fields: HashMap<String, Value> = HashMap::new();
                    for (need_name, type_ann, default) in &target_class.needs {
                        if let Some(type_name) = type_ann
                            && let Some(val) = regs.get(type_name)
                        {
                            fields.insert(need_name.clone(), val.clone());
                            continue;
                        }
                        if let Some(default_expr) = default {
                            let val = self.eval_expr(default_expr)?;
                            fields.insert(need_name.clone(), val);
                            continue;
                        }
                        return Err(EvalError::Fail(Value::String(format!(
                            "Container cannot resolve '{}' for {}.new() — no registration for {}",
                            need_name,
                            target_class.name,
                            type_ann.as_deref().unwrap_or("unknown")
                        ))));
                    }

                    let new_instance_id = InstanceId(self.instances.len());
                    self.instances.push(StoredInstance {
                        class_id: target_class_id,
                        fields,
                    });
                    return Ok(Value::Instance(new_instance_id));
                }
                "resolve_name" => {
                    let name = match &args[0] {
                        Value::String(s) => s.clone(),
                        _ => {
                            return Err(EvalError::Panic(
                                PanicKind::TypeError,
                                "resolve_name() arg must be a String".into(),
                            ));
                        }
                    };
                    let regs = self
                        .container_registrations
                        .get(instance_id)
                        .cloned()
                        .unwrap_or_default();
                    if let Some(val) = regs.get(&name) {
                        return Ok(Value::clone(val));
                    }
                    return Err(EvalError::Fail(Value::String(format!(
                        "No registration for {}",
                        name
                    ))));
                }
                _ => {}
            }
        }

        // Auto-methods for model instances
        if self.model_classes.contains_key(&instance.class_id) {
            if method == "to_dict" {
                let entries: Vec<(String, Value)> = class
                    .needs
                    .iter()
                    .map(|(name, _, _)| {
                        let val = instance.fields.get(name).cloned().unwrap_or(Value::Null);
                        (name.clone(), val)
                    })
                    .collect();
                return Ok(Value::Dict(entries));
            }
            if method == "copy" {
                // Create a new instance with fields overridden by named args
                let mut new_fields = instance.fields.clone();
                for (name, val) in named_args {
                    if let Some(name) = name {
                        new_fields.insert(name.clone(), val.clone());
                    }
                }
                let new_instance_id = InstanceId(self.instances.len());
                self.instances.push(StoredInstance {
                    class_id: instance.class_id,
                    fields: new_fields,
                });
                // Run validators on the new instance
                if let Some(validators) = self.model_classes.get(&instance.class_id).cloned() {
                    for (field_name, validator_expr) in &validators {
                        let field_val = self.instances[new_instance_id.0]
                            .fields
                            .get(field_name)
                            .cloned()
                            .unwrap_or(Value::Null);
                        let validator_fn = self.eval_expr(validator_expr)?;
                        let result = self.call_value(validator_fn, vec![(None, field_val)])?;
                        if !result.is_truthy() {
                            return Err(EvalError::Panic(
                                PanicKind::RuntimeError,
                                format!(
                                    "validation failed for field '{}' in {}.copy()",
                                    field_name, class.name
                                ),
                            ));
                        }
                    }
                }
                self.frozen_instances.insert(new_instance_id);
                return Ok(Value::Instance(new_instance_id));
            }
        }

        // Find method in class — dispatch by name + arity + type
        let method_fn: Option<(crate::eval::StoredFunction, ClassId)> = class
            .methods
            .iter()
            // 1. Exact type + arity match
            .find(|m| {
                m.name == method
                    && m.params.len() == args.len()
                    && self.args_match_types(&args, &m.param_types)
            })
            // 2. Arity match (untyped)
            .or_else(|| {
                class.methods.iter().find(|m| {
                    m.name == method
                        && m.params.len() == args.len()
                        && m.param_types.iter().all(|t| t.is_none())
                })
            })
            // 3. Any arity match
            .or_else(|| {
                class
                    .methods
                    .iter()
                    .find(|m| m.name == method && m.params.len() == args.len())
            })
            // 4. Fallback to name match
            .or_else(|| class.methods.iter().find(|m| m.name == method))
            .map(|f| (f.clone(), instance.class_id));

        // Walk parent chain if method not found on this class
        let method_fn = method_fn.or_else(|| {
            let mut current = class.parent;
            while let Some(pid) = current {
                let parent_class = &self.classes[pid.0];
                if let Some(f) = parent_class.methods.iter().find(|m| m.name == method) {
                    return Some((f.clone(), pid));
                }
                current = parent_class.parent;
            }
            None
        });

        if let Some((func, found_class_id)) = method_fn {
            // Enforce visibility: private methods only callable from same class
            if func.visibility == Visibility::Private {
                let caller_class = self.current_self.map(|id| self.instances[id.0].class_id);
                if caller_class != Some(instance.class_id) {
                    return Err(EvalError::Panic(
                        PanicKind::RuntimeError,
                        format!(
                            "private method '{}' cannot be called from outside the class",
                            method
                        ),
                    ));
                }
            }
            if args.len() != func.params.len() {
                return Err(EvalError::Panic(
                    PanicKind::TypeError,
                    format!(
                        "{}() expected {} arguments, got {}",
                        method,
                        func.params.len(),
                        args.len()
                    ),
                ));
            }

            // Set self, method tracking, and push scope
            let prev_self = self.current_self;
            let prev_method = self.current_method_name.take();
            let prev_class = self.current_class_id.take();
            self.current_self = Some(*instance_id);
            self.current_method_name = Some(method.to_string());
            self.current_class_id = Some(found_class_id);
            self.env.push_scope();
            for (param_name, arg_val) in func.params.iter().zip(args) {
                self.env.set(String::clone(param_name), arg_val);
            }

            let result = self.eval_block(&func.body);
            self.env.pop_scope();
            self.current_self = prev_self;
            self.current_method_name = prev_method;
            self.current_class_id = prev_class;

            match result {
                Ok(val) => self.maybe_auto_throw(val),
                Err(EvalError::Flow(FlowSignal::Return(val))) => self.maybe_auto_throw(val),
                Err(e) => Err(e),
            }
        } else {
            Err(EvalError::Panic(
                PanicKind::TypeError,
                format!("no method '{}' on instance of class", method),
            ))
        }
    }
}

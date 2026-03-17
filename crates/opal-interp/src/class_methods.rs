use std::collections::HashMap;
use std::io::Write;

use opal_runtime::{ActorId, ClassId, InstanceId, Value};

use crate::eval::{
    EvalError, FlowSignal, Interpreter, PanicKind, StoredActorInstance, StoredInstance,
};

impl<W: Write> Interpreter<W> {
    pub(crate) fn call_class_method(
        &mut self,
        class_id: &ClassId,
        method: &str,
        args: Vec<Value>,
        named_args: &[(Option<String>, Value)],
    ) -> Result<Value, EvalError> {
        match method {
            "new" => {
                let class = self.classes[class_id.0].clone();
                let all_needs = self.gather_all_needs(*class_id);
                let mut fields = HashMap::new();

                // Match named args to needs declarations (includes inherited needs)
                for (need_name, type_ann, default) in &all_needs {
                    // Try named arg first
                    let value = named_args
                        .iter()
                        .find(|(name, _)| name.as_deref() == Some(need_name.as_str()))
                        .map(|(_, v)| v.clone());
                    let val = if let Some(val) = value {
                        val
                    } else {
                        // Try positional
                        let idx = all_needs
                            .iter()
                            .position(|(n, _, _)| n == need_name)
                            .unwrap();
                        if idx < args.len() {
                            args[idx].clone()
                        } else if let Some(default_expr) = default {
                            self.eval_expr(default_expr)?
                        } else {
                            return Err(EvalError::Panic(
                                PanicKind::TypeError,
                                format!("missing required field '{}' in .new()", need_name),
                            ));
                        }
                    };

                    // Protocol conformance check
                    if let Some(type_name) = type_ann
                        && let Some(Value::Protocol(_proto_id)) = self.env.get(type_name).cloned()
                    {
                        let conforms = match &val {
                            Value::Instance(iid) => {
                                let inst_class_id = self.instances[iid.0].class_id;
                                self.class_implements_protocol(inst_class_id, type_name)
                            }
                            _ => true,
                        };
                        if !conforms {
                            return Err(EvalError::Fail(Value::String(format!(
                                "{}.new() — '{}' must implement {}",
                                class.name, need_name, type_name
                            ))));
                        }
                    }

                    fields.insert(need_name.clone(), val);
                }

                let instance_id = InstanceId(self.instances.len());
                self.instances.push(StoredInstance {
                    class_id: *class_id,
                    fields,
                });

                // Call init() if defined on class or parent chain
                {
                    let mut init_fn = None;
                    // Search the class itself
                    if let Some(f) = class.methods.iter().find(|m| m.name == "init") {
                        init_fn = Some((f.clone(), *class_id));
                    }
                    // Walk parent chain if not found
                    if init_fn.is_none() {
                        let mut current = class.parent;
                        while let Some(pid) = current {
                            let parent_class = &self.classes[pid.0];
                            if let Some(f) = parent_class.methods.iter().find(|m| m.name == "init")
                            {
                                init_fn = Some((f.clone(), pid));
                                break;
                            }
                            current = parent_class.parent;
                        }
                    }
                    if let Some((func, found_class_id)) = init_fn {
                        let prev_self = self.current_self;
                        let prev_method = self.current_method_name.take();
                        let prev_class = self.current_class_id.take();
                        self.current_self = Some(instance_id);
                        self.current_method_name = Some("init".to_string());
                        self.current_class_id = Some(found_class_id);
                        self.env.push_scope();
                        self.env
                            .set("self".to_string(), Value::Instance(instance_id));
                        let result = self.eval_block(&func.body);
                        self.env.pop_scope();
                        self.current_self = prev_self;
                        self.current_method_name = prev_method;
                        self.current_class_id = prev_class;
                        match result {
                            Ok(_) => {}
                            Err(EvalError::Flow(FlowSignal::Return(_))) => {}
                            Err(e) => return Err(e),
                        }
                    }
                }

                // If this is a model class, run validators and freeze
                if let Some(validators) = self.model_classes.get(class_id).cloned() {
                    for (field_name, validator_expr) in &validators {
                        let field_val = self.instances[instance_id.0]
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
                                    "validation failed for field '{}' in {}.new()",
                                    field_name, class.name
                                ),
                            ));
                        }
                    }
                    self.frozen_instances.insert(instance_id);
                }

                Ok(Value::Instance(instance_id))
            }

            // Static methods on class (def self.method())
            _ => {
                // Search class and parent chain for static methods
                let mut found = None;
                let mut search = Some(*class_id);
                while let Some(cid) = search {
                    let c = &self.classes[cid.0];
                    if let Some(f) = c.static_methods.iter().find(|m| m.name == method) {
                        found = Some(f.clone());
                        break;
                    }
                    search = c.parent;
                }
                if let Some(func) = found {
                    self.env.push_scope();
                    // Bind Self to the class
                    self.env.set("Self".to_string(), Value::Class(*class_id));
                    for (i, param) in func.params.iter().enumerate() {
                        // Try named args first, then positional
                        let val = named_args
                            .iter()
                            .find(|(name, _)| name.as_deref() == Some(param.as_str()))
                            .map(|(_, v)| v.clone())
                            .or_else(|| args.get(i).cloned());
                        if let Some(v) = val {
                            self.env.set(param.clone(), v);
                        } else if let Some(default_expr) = &func.param_defaults[i] {
                            let v = self.eval_expr(default_expr)?;
                            self.env.set(param.clone(), v);
                        }
                    }
                    let result = self.eval_block(&func.body);
                    self.env.pop_scope();
                    return match result {
                        Ok(val) => self.maybe_auto_throw(val),
                        Err(EvalError::Flow(FlowSignal::Return(v))) => self.maybe_auto_throw(v),
                        Err(e) => Err(e),
                    };
                }
                Err(EvalError::Panic(
                    PanicKind::RuntimeError,
                    format!("undefined static method '{}' on class", method),
                ))
            }
        }
    }

    pub(crate) fn call_actor_method(
        &mut self,
        obj: &Value,
        method: &str,
        args: Vec<Value>,
        named_args: &[(Option<String>, Value)],
    ) -> Result<Value, EvalError> {
        match (obj, method) {
            // Actor .new()
            (Value::ActorClass(def_id), "new") => {
                let def_idx = def_id.0;
                let def = self.actor_defs[def_idx].clone();
                let actor_id = ActorId(self.actors.len());

                // Resolve needs
                let mut fields = HashMap::new();
                for (need_name, _type_ann, default) in &def.needs {
                    let value = named_args
                        .iter()
                        .find(|(name, _)| name.as_deref() == Some(need_name.as_str()))
                        .map(|(_, v)| v.clone());
                    if let Some(val) = value {
                        fields.insert(need_name.clone(), val);
                    } else if let Some(default_expr) = default {
                        let val = self.eval_expr(default_expr)?;
                        fields.insert(need_name.clone(), val);
                    } else {
                        let idx = def
                            .needs
                            .iter()
                            .position(|(n, _, _)| n == need_name)
                            .unwrap();
                        if idx < args.len() {
                            fields.insert(need_name.clone(), args[idx].clone());
                        } else {
                            return Err(EvalError::Panic(
                                PanicKind::TypeError,
                                format!("missing required field '{}' in actor .new()", need_name),
                            ));
                        }
                    }
                }

                self.actors.push(StoredActorInstance { def_idx, fields });
                // Run init if present
                if let Some(init_body) = &def.init {
                    let prev_actor = self.current_actor;
                    self.current_actor = Some(actor_id);
                    self.env.push_scope();
                    let result = self.eval_block(init_body);
                    self.env.pop_scope();
                    self.current_actor = prev_actor;
                    result?;
                }
                Ok(Value::Actor(actor_id))
            }

            // Actor .send(:msg)
            (Value::Actor(actor_id), "send") => {
                if args.len() != 1 {
                    return Err(EvalError::Panic(
                        PanicKind::TypeError,
                        "send() takes exactly 1 argument".into(),
                    ));
                }
                let msg = args[0].clone();
                let def_idx = self.actors[actor_id.0].def_idx;
                let cases = self.actor_defs[def_idx].receive_cases.clone();

                let prev_actor = self.current_actor;
                self.current_actor = Some(*actor_id);
                self.env.push_scope();

                let mut reply_val = Value::Null;
                for case in &cases {
                    if let Some(bindings) = self.match_pattern(&case.pattern, &msg) {
                        for (name, val) in bindings {
                            self.env.set(name, val);
                        }
                        match self.eval_block(&case.body) {
                            Ok(_) => {}
                            Err(EvalError::Flow(FlowSignal::Reply(val))) => {
                                reply_val = val;
                            }
                            Err(e) => {
                                self.env.pop_scope();
                                self.current_actor = prev_actor;
                                return Err(e);
                            }
                        }
                        break;
                    }
                }

                self.env.pop_scope();
                self.current_actor = prev_actor;
                Ok(reply_val)
            }

            _ => Err(EvalError::Panic(
                PanicKind::TypeError,
                format!("no method '{}' on {:?}", method, obj),
            )),
        }
    }
}

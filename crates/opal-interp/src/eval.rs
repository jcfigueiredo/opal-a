use std::io::Write;

use std::collections::HashMap;

use opal_parser::ast::*;
#[allow(unused_imports)]
use opal_runtime::{
    ActorDefId, ActorId, AstId, BuiltinType, ClassId, ClosureId, EnumId, Environment, FunctionId,
    InstanceId, MacroId, ModuleId, NativeFunctionId, NativeObjectId, ProtocolId, TypeInfo, Value,
};
use thiserror::Error;

use crate::loader;

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("NameError: undefined variable '{0}'")]
    UndefinedVariable(String),
    #[error("TypeError: {0}")]
    TypeError(String),
    #[error("RuntimeError: {0}")]
    RuntimeError(String),
    #[error("return")]
    Return(Value),
    #[error("{0}")]
    Raise(Value),
    #[error("requires failed")]
    RequiresFailed(Value),
    #[error("reply")]
    Reply(Value),
    #[error("break")]
    Break,
    #[error("next")]
    Next,
}

/// A stored user-defined function
#[derive(Clone)]
struct StoredFunction {
    #[allow(dead_code)]
    name: String,
    params: Vec<String>,
    param_types: Vec<Option<String>>,
    param_defaults: Vec<Option<Expr>>,
    body: Vec<Stmt>,
    /// Captured environment from defining scope (for module-level functions)
    captured_env: Option<Environment>,
    /// Annotations from @[...] declarations
    annotations: Vec<Vec<(String, Value)>>,
}

/// A stored closure
#[derive(Clone)]
struct StoredClosure {
    params: Vec<String>,
    body: Vec<Stmt>,
    captured_env: Environment,
}

/// A stored class definition
#[derive(Clone)]
struct StoredClass {
    #[allow(dead_code)]
    name: String,
    needs: Vec<(String, Option<String>)>,
    methods: Vec<StoredFunction>,
}

/// A stored protocol definition
#[derive(Clone)]
struct StoredProtocol {
    #[allow(dead_code)]
    name: String,
    /// Default methods (with bodies) that get copied to implementing classes
    default_methods: Vec<StoredFunction>,
    /// Required method names (no bodies) that classes must define
    required_methods: Vec<String>,
}

/// A stored instance
#[derive(Clone)]
struct StoredInstance {
    class_id: ClassId,
    fields: HashMap<String, Value>,
}

/// A stored module
#[derive(Clone)]
struct StoredModule {
    #[allow(dead_code)]
    name: String,
    bindings: HashMap<String, Value>,
}

/// A stored macro definition
#[derive(Clone)]
struct StoredMacro {
    params: Vec<String>,
    body: Vec<Stmt>,
}

/// A stored actor definition
#[derive(Clone)]
struct StoredActorDef {
    #[allow(dead_code)]
    name: String,
    init: Option<Vec<Stmt>>,
    receive_cases: Vec<MatchCase>,
}

/// A stored actor instance
#[derive(Clone)]
struct StoredActorInstance {
    def_idx: usize,
    fields: HashMap<String, Value>,
}

/// A stored enum definition
#[derive(Clone)]
#[allow(dead_code)]
struct StoredEnum {
    name: String,
    variants: Vec<StoredEnumVariant>,
    methods: Vec<StoredFunction>,
}

/// A stored enum variant
#[derive(Clone)]
#[allow(dead_code)]
struct StoredEnumVariant {
    name: String,
    fields: Vec<(String, Option<String>)>,
}

pub struct Interpreter<W: Write> {
    env: Environment,
    writer: W,
    functions: Vec<StoredFunction>,
    closures: Vec<StoredClosure>,
    classes: Vec<StoredClass>,
    instances: Vec<StoredInstance>,
    modules: Vec<StoredModule>,
    actor_defs: Vec<StoredActorDef>,
    actors: Vec<StoredActorInstance>,
    /// Current `self` instance for method calls
    current_self: Option<InstanceId>,
    /// Current actor for receive handlers
    current_actor: Option<ActorId>,
    /// True when loading a file-based module (functions should capture env)
    loading_module: bool,
    macros: Vec<StoredMacro>,
    protocols: Vec<StoredProtocol>,
    enums: Vec<StoredEnum>,
    type_aliases: HashMap<String, TypeExpr>,
    ast_nodes: Vec<Vec<Stmt>>,
    /// Registry of FFI plugins
    plugin_registry: opal_stdlib::PluginRegistry,
    /// Storage for opaque native objects (FFI state)
    native_objects: Vec<Box<dyn std::any::Any>>,
    /// Maps NativeFunctionId → "plugin:function" key for dispatch
    native_functions: Vec<String>,
    /// File-based module loader (set when running from a file)
    module_loader: Option<loader::ModuleLoader>,
}

impl Interpreter<std::io::Stdout> {
    pub fn new() -> Self {
        let mut interp = Self {
            env: Environment::new(),
            writer: std::io::stdout(),
            functions: Vec::new(),
            closures: Vec::new(),
            classes: Vec::new(),
            instances: Vec::new(),
            modules: Vec::new(),
            actor_defs: Vec::new(),
            actors: Vec::new(),
            current_self: None,
            current_actor: None,
            loading_module: false,
            macros: Vec::new(),
            protocols: Vec::new(),
            enums: Vec::new(),
            type_aliases: HashMap::new(),
            ast_nodes: Vec::new(),
            plugin_registry: opal_stdlib::PluginRegistry::new(),
            native_objects: Vec::new(),
            native_functions: Vec::new(),
            module_loader: None,
        };
        interp.register_builtin_enums();
        interp
    }

    pub fn with_base_dir(base_dir: &std::path::Path) -> Self {
        let mut interp = Self::new();
        interp.module_loader = Some(loader::ModuleLoader::new(base_dir));
        interp
    }
}

impl<W: Write> Interpreter<W> {
    pub fn with_writer(writer: W) -> Self {
        let mut interp = Self {
            env: Environment::new(),
            writer,
            functions: Vec::new(),
            closures: Vec::new(),
            classes: Vec::new(),
            instances: Vec::new(),
            modules: Vec::new(),
            actor_defs: Vec::new(),
            actors: Vec::new(),
            current_self: None,
            current_actor: None,
            loading_module: false,
            macros: Vec::new(),
            protocols: Vec::new(),
            enums: Vec::new(),
            type_aliases: HashMap::new(),
            ast_nodes: Vec::new(),
            plugin_registry: opal_stdlib::PluginRegistry::new(),
            native_objects: Vec::new(),
            native_functions: Vec::new(),
            module_loader: None,
        };
        interp.register_builtin_enums();
        interp
    }

    pub fn with_base_dir_writer(writer: W, base_dir: &std::path::Path) -> Self {
        let mut interp = Self::with_writer(writer);
        interp.module_loader = Some(loader::ModuleLoader::new(base_dir));
        interp
    }

    /// Register Result(Ok, Err) and Option(Some, None) as built-in enums
    fn register_builtin_enums(&mut self) {
        // Result enum at index 0: Ok(value), Err(value)
        self.enums.push(StoredEnum {
            name: "Result".to_string(),
            variants: vec![
                StoredEnumVariant {
                    name: "Ok".to_string(),
                    fields: vec![("value".to_string(), None)],
                },
                StoredEnumVariant {
                    name: "Error".to_string(),
                    fields: vec![("value".to_string(), None)],
                },
            ],
            methods: vec![],
        });
        // Option enum at index 1: Some(value), None
        self.enums.push(StoredEnum {
            name: "Option".to_string(),
            variants: vec![
                StoredEnumVariant {
                    name: "Some".to_string(),
                    fields: vec![("value".to_string(), None)],
                },
                StoredEnumVariant {
                    name: "None".to_string(),
                    fields: vec![],
                },
            ],
            methods: vec![],
        });
    }

    /// Register an FFI plugin with its native functions.
    pub fn register_plugin(
        &mut self,
        name: &str,
        functions: HashMap<String, opal_stdlib::NativeFunction>,
    ) {
        self.plugin_registry.register_plugin(name, functions);
    }

    /// Store a native object and return its opaque ID.
    pub fn store_native_object<T: 'static>(&mut self, obj: T) -> NativeObjectId {
        let id = NativeObjectId(self.native_objects.len());
        self.native_objects.push(Box::new(obj));
        id
    }

    /// Retrieve a reference to a stored native object by ID.
    pub fn get_native_object<T: 'static>(&self, id: NativeObjectId) -> Option<&T> {
        self.native_objects.get(id.0)?.downcast_ref::<T>()
    }

    /// Retrieve a mutable reference to a stored native object by ID.
    pub fn get_native_object_mut<T: 'static>(&mut self, id: NativeObjectId) -> Option<&mut T> {
        self.native_objects.get_mut(id.0)?.downcast_mut::<T>()
    }

    pub fn run(&mut self, program: &Program) -> Result<(), EvalError> {
        self.register_stdlib_modules();
        for stmt in &program.statements {
            self.eval_stmt(stmt)?;
        }
        Ok(())
    }

    fn register_stdlib_modules(&mut self) {
        // Math module — handled as a special builtin module
        let mut math_bindings = HashMap::new();
        // Store pi as a float value — Math.pi() will be handled specially
        math_bindings.insert("pi".into(), Value::Float(std::f64::consts::PI));

        let math_module_id = ModuleId(self.modules.len());
        self.modules.push(StoredModule {
            name: "Math".into(),
            bindings: math_bindings,
        });
        self.env.set("Math".into(), Value::Module(math_module_id));

        // HTTP plugin — provides create_app, add_route, get_routes, serve
        self.register_plugin("http", opal_stdlib::http::register_http_plugin());
    }

    /// Ensure a module is loaded into the environment, trying file-based loading if needed.
    /// Returns the Value for the module.
    fn ensure_module_loaded(
        &mut self,
        module_key: &str,
        module_path: &[String],
    ) -> Result<Value, EvalError> {
        // Check if already in scope
        if let Some(val) = self.env.get(module_key).cloned() {
            return Ok(val);
        }

        // Try file-based loading
        let file_path = self
            .module_loader
            .as_ref()
            .and_then(|loader| loader.resolve(module_path));

        if let Some(file_path) = file_path {
            let loader = self.module_loader.as_mut().unwrap();
            if loader.is_loaded(module_key) {
                // Already loaded, should be in env
                if let Some(val) = self.env.get(module_key).cloned() {
                    return Ok(val);
                }
            }

            if !loader.mark_loading(module_key) {
                return Err(EvalError::RuntimeError(format!(
                    "circular dependency: {}",
                    module_key
                )));
            }

            let source = std::fs::read_to_string(&file_path)
                .map_err(|e| EvalError::RuntimeError(e.to_string()))?;
            let program = opal_parser::parse(&source).map_err(|e| {
                EvalError::RuntimeError(format!("parse error in {}: {}", file_path.display(), e))
            })?;

            // Evaluate in a new scope, capture bindings as module
            let was_loading = self.loading_module;
            self.loading_module = true;
            self.env.push_scope();
            for stmt in &program.statements {
                self.eval_stmt(stmt)?;
            }
            let bindings = self.env.current_scope_bindings();
            self.env.pop_scope();
            self.loading_module = was_loading;

            let module_id = ModuleId(self.modules.len());
            self.modules.push(StoredModule {
                name: module_key.to_string(),
                bindings,
            });
            self.env
                .set(module_key.to_string(), Value::Module(module_id));

            if let Some(loader) = self.module_loader.as_mut() {
                loader.mark_loaded(module_key);
            }

            return Ok(Value::Module(module_id));
        }

        Err(EvalError::UndefinedVariable(module_key.to_string()))
    }

    /// Apply import bindings from a module value according to the import kind.
    fn apply_import(
        &mut self,
        kind: &ImportKind,
        module_key: &str,
        module_val: Value,
    ) -> Result<(), EvalError> {
        let module_id = match module_val {
            Value::Module(id) => id,
            _ => {
                return Err(EvalError::TypeError(format!(
                    "'{}' is not a module",
                    module_key
                )));
            }
        };

        match kind {
            ImportKind::Selective(items) => {
                let module = self.modules[module_id.0].clone();
                for item in items {
                    if let Some(val) = module.bindings.get(&item.name) {
                        let bind_name = item.alias.as_ref().unwrap_or(&item.name).clone();
                        self.env.set(bind_name, val.clone());
                    } else {
                        return Err(EvalError::UndefinedVariable(format!(
                            "{}.{}",
                            module_key, item.name
                        )));
                    }
                }
            }
            ImportKind::Module => {
                let bind_name = module_key
                    .rsplit('.')
                    .next()
                    .unwrap_or(module_key)
                    .to_string();
                self.env.set(bind_name, Value::Module(module_id));
            }
            ImportKind::ModuleAlias(alias) => {
                self.env.set(alias.clone(), Value::Module(module_id));
            }
        }
        Ok(())
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<(), EvalError> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                self.eval_expr(expr)?;
            }
            StmtKind::Assign { name, value } => {
                let val = self.eval_expr(value)?;
                self.env.assign(name.clone(), val);
            }
            StmtKind::CompoundAssign { name, op, value } => {
                let current = self.env.get(name).cloned()
                    .ok_or_else(|| EvalError::UndefinedVariable(name.clone()))?;
                let rhs = self.eval_expr(value)?;
                let result = eval_binary_op(*op, current, rhs)?;
                self.env.assign(name.clone(), result);
            }
            StmtKind::IndexAssign {
                object,
                index,
                value,
            } => {
                let idx = self.eval_expr(index)?;
                let val = self.eval_expr(value)?;
                if let ExprKind::Identifier(name) = &object.kind {
                    let mut obj = self
                        .env
                        .get(name)
                        .cloned()
                        .ok_or_else(|| EvalError::UndefinedVariable(name.clone()))?;
                    match (&mut obj, &idx) {
                        (Value::List(items), Value::Integer(i)) => {
                            let i = if *i < 0 {
                                items.len() as i64 + i
                            } else {
                                *i
                            } as usize;
                            if i < items.len() {
                                items[i] = val;
                            } else {
                                return Err(EvalError::RuntimeError(format!(
                                    "index {} out of bounds for list of length {}",
                                    i,
                                    items.len()
                                )));
                            }
                        }
                        (Value::Dict(entries), Value::String(key)) => {
                            if let Some(entry) = entries.iter_mut().find(|(k, _)| k == key) {
                                entry.1 = val;
                            } else {
                                entries.push((key.clone(), val));
                            }
                        }
                        _ => {
                            return Err(EvalError::TypeError(
                                "invalid index assignment".into(),
                            ));
                        }
                    }
                    self.env.assign(name.clone(), obj);
                } else {
                    return Err(EvalError::TypeError(
                        "index assignment target must be a variable".into(),
                    ));
                }
            }
            StmtKind::Let { name, value } => {
                let val = self.eval_expr(value)?;
                self.env.set(name.clone(), val);
            }
            StmtKind::FuncDef {
                name, params, body, ..
            } => {
                let id = FunctionId(self.functions.len());
                let captured = if self.loading_module {
                    Some(self.env.snapshot())
                } else {
                    None
                };
                self.functions.push(StoredFunction {
                    name: name.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    param_types: params.iter().map(|p| p.type_annotation.clone()).collect(),
                    param_defaults: params.iter().map(|p| p.default.clone()).collect(),
                    body: body.clone(),
                    captured_env: captured,
                    annotations: vec![],
                });

                // Support multiple dispatch: if name already bound to a function,
                // create or extend a dispatch group
                match self.env.get(name).cloned() {
                    Some(Value::Function(existing_id)) => {
                        // Promote to multi-function
                        self.env
                            .set(name.clone(), Value::MultiFunction(vec![existing_id, id]));
                    }
                    Some(Value::MultiFunction(mut ids)) => {
                        ids.push(id);
                        self.env.set(name.clone(), Value::MultiFunction(ids));
                    }
                    _ => {
                        self.env.set(name.clone(), Value::Function(id));
                    }
                }
            }
            StmtKind::Return(expr) => {
                let val = match expr {
                    Some(e) => self.eval_expr(e)?,
                    None => Value::Null,
                };
                return Err(EvalError::Return(val));
            }
            StmtKind::For {
                var,
                iterable,
                body,
            } => {
                let iter_val = self.eval_expr(iterable)?;
                match iter_val {
                    Value::List(items) => {
                        for item in items {
                            self.env.push_scope();
                            self.env.set(var.clone(), item);
                            let result = self.eval_block(body);
                            self.env.pop_scope();
                            match result {
                                Err(EvalError::Break) => break,
                                Err(EvalError::Next) => continue,
                                other => { other?; }
                            }
                        }
                    }
                    Value::Range {
                        start,
                        end,
                        inclusive,
                    } => {
                        let end_val = if inclusive { end + 1 } else { end };
                        for i in start..end_val {
                            self.env.push_scope();
                            self.env.set(var.clone(), Value::Integer(i));
                            let result = self.eval_block(body);
                            self.env.pop_scope();
                            match result {
                                Err(EvalError::Break) => break,
                                Err(EvalError::Next) => continue,
                                other => { other?; }
                            }
                        }
                    }
                    _ => {
                        return Err(EvalError::TypeError(
                            "for loop requires a list or range".into(),
                        ));
                    }
                }
            }
            StmtKind::While { condition, body } => loop {
                let cond = self.eval_expr(condition)?;
                if !cond.is_truthy() {
                    break;
                }
                match self.eval_block(body) {
                    Err(EvalError::Break) => break,
                    Err(EvalError::Next) => continue,
                    other => { other?; }
                }
            },
            StmtKind::ClassDef {
                name,
                needs,
                methods,
                implements,
            } => {
                let mut stored_methods = Vec::new();
                for method_stmt in methods {
                    if let StmtKind::FuncDef {
                        name: mname,
                        params,
                        body,
                        ..
                    } = &method_stmt.kind
                    {
                        stored_methods.push(StoredFunction {
                            name: mname.clone(),
                            params: params.iter().map(|p| p.name.clone()).collect(),
                            param_types: params.iter().map(|p| p.type_annotation.clone()).collect(),
                            param_defaults: params.iter().map(|p| p.default.clone()).collect(),
                            body: body.clone(),
                            captured_env: None,
                            annotations: vec![],
                        });
                    }
                }

                // Apply protocol defaults and check required methods
                let class_method_names: Vec<String> =
                    stored_methods.iter().map(|m| m.name.clone()).collect();

                for proto_name in implements {
                    let proto_id = match self.env.get(proto_name) {
                        Some(Value::Protocol(id)) => *id,
                        _ => {
                            return Err(EvalError::UndefinedVariable(format!(
                                "protocol {}",
                                proto_name
                            )));
                        }
                    };
                    let proto = self.protocols[proto_id.0].clone();

                    // Copy default methods that aren't already defined
                    for default in &proto.default_methods {
                        if !class_method_names.contains(&default.name) {
                            stored_methods.push(default.clone());
                        }
                    }

                    // Check all required methods are implemented
                    for required in &proto.required_methods {
                        let has_it = stored_methods.iter().any(|m| m.name == *required);
                        if !has_it {
                            return Err(EvalError::RuntimeError(format!(
                                "class '{}' implements '{}' but missing required method '{}'",
                                name, proto_name, required
                            )));
                        }
                    }
                }

                let class_id = ClassId(self.classes.len());
                self.classes.push(StoredClass {
                    name: name.clone(),
                    needs: needs
                        .iter()
                        .map(|n| (n.name.clone(), n.type_annotation.clone()))
                        .collect(),
                    methods: stored_methods,
                });
                self.env.set(name.clone(), Value::Class(class_id));
            }
            StmtKind::ProtocolDef { name, methods } => {
                let mut default_methods = Vec::new();
                let mut required_methods = Vec::new();

                for method in methods {
                    if let Some(body) = &method.body {
                        default_methods.push(StoredFunction {
                            name: method.name.clone(),
                            params: method.params.iter().map(|p| p.name.clone()).collect(),
                            param_types: method.params.iter().map(|p| p.type_annotation.clone()).collect(),
                            param_defaults: method.params.iter().map(|p| p.default.clone()).collect(),
                            body: body.clone(),
                            captured_env: None,
                            annotations: vec![],
                        });
                    } else {
                        required_methods.push(method.name.clone());
                    }
                }

                let proto_id = ProtocolId(self.protocols.len());
                self.protocols.push(StoredProtocol {
                    name: name.clone(),
                    default_methods,
                    required_methods,
                });
                self.env.set(name.clone(), Value::Protocol(proto_id));
            }
            StmtKind::ModuleDef { name, body } => {
                // Evaluate body in a new scope, capture bindings
                self.env.push_scope();
                for stmt in body {
                    self.eval_stmt(stmt)?;
                }
                // Collect all bindings from the module scope
                let bindings = self.env.current_scope_bindings();
                self.env.pop_scope();

                let module_id = ModuleId(self.modules.len());
                self.modules.push(StoredModule {
                    name: name.clone(),
                    bindings,
                });
                self.env.set(name.clone(), Value::Module(module_id));
            }
            StmtKind::FromImport { module_path, names } => {
                let module_val = self
                    .env
                    .get(module_path)
                    .cloned()
                    .ok_or_else(|| EvalError::UndefinedVariable(module_path.clone()))?;
                let module_id = match module_val {
                    Value::Module(id) => id,
                    _ => {
                        return Err(EvalError::TypeError(format!(
                            "'{}' is not a module",
                            module_path
                        )));
                    }
                };
                let module = self.modules[module_id.0].clone();
                for name in names {
                    if let Some(val) = module.bindings.get(name) {
                        self.env.set(name.clone(), val.clone());
                    } else {
                        return Err(EvalError::UndefinedVariable(format!(
                            "{}.{}",
                            module_path, name
                        )));
                    }
                }
            }
            StmtKind::Import(imp) => {
                let module_key = imp.path.join(".");
                let module_val = self.ensure_module_loaded(&module_key, &imp.path)?;
                self.apply_import(&imp.kind, &module_key, module_val)?;
            }
            StmtKind::Annotated { annotations, statement } => {
                // Evaluate annotation values and store them
                let mut stored_anns: Vec<Vec<(String, Value)>> = Vec::new();
                for ann in annotations {
                    let mut entries = Vec::new();
                    for entry in &ann.entries {
                        let val = match &entry.value {
                            Some(expr) => self.eval_expr(expr)?,
                            None => Value::Bool(true),
                        };
                        entries.push((entry.key.clone(), val));
                    }
                    stored_anns.push(entries);
                }
                // Evaluate the inner statement
                self.eval_stmt(statement)?;
                // Attach annotations to the last defined function
                if let StmtKind::FuncDef { name, .. } = &statement.kind {
                    if let Some(val) = self.env.get(name).cloned() {
                        match val {
                            Value::Function(id) => {
                                self.functions[id.0].annotations = stored_anns;
                            }
                            Value::MultiFunction(ids) => {
                                if let Some(id) = ids.last() {
                                    self.functions[id.0].annotations = stored_anns;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            StmtKind::TypeAlias { name, definition } => {
                self.type_aliases.insert(name.clone(), definition.clone());
            }
            StmtKind::EnumDef {
                name,
                variants,
                methods,
                ..
            } => {
                let enum_id = EnumId(self.enums.len());
                let stored_variants: Vec<StoredEnumVariant> = variants
                    .iter()
                    .map(|v| StoredEnumVariant {
                        name: v.name.clone(),
                        fields: v.fields.iter().map(|f| (f.name.clone(), f.type_annotation.clone())).collect(),
                    })
                    .collect();
                let stored_methods: Vec<StoredFunction> = methods
                    .iter()
                    .filter_map(|m| {
                        if let StmtKind::FuncDef { name, params, body, .. } = &m.kind {
                            Some(StoredFunction {
                                name: name.clone(),
                                params: params.iter().map(|p| p.name.clone()).collect(),
                                param_types: params.iter().map(|p| p.type_annotation.clone()).collect(),
                                param_defaults: params.iter().map(|p| p.default.clone()).collect(),
                                body: body.clone(),
                                captured_env: None,
                                annotations: vec![],
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                self.enums.push(StoredEnum {
                    name: name.clone(),
                    variants: stored_variants,
                    methods: stored_methods,
                });
                self.env.set(name.clone(), Value::Type(TypeInfo::Enum(enum_id)));
            }
            StmtKind::ExportBlock(_) => {
                // Export blocks are metadata; no runtime effect for now
            }
            StmtKind::NeedsDecl(_) => {
                // Handled during class definition parsing, not at runtime
            }
            StmtKind::Requires { condition, message } => {
                let cond = self.eval_expr(condition)?;
                if !cond.is_truthy() {
                    let msg = match message {
                        Some(m) => self.eval_expr(m)?,
                        None => Value::String("requires condition failed".into()),
                    };
                    return Err(EvalError::RequiresFailed(msg));
                }
            }
            StmtKind::Raise(expr) => {
                let val = self.eval_expr(expr)?;
                return Err(EvalError::Raise(val));
            }
            StmtKind::ActorDef {
                name,
                init,
                receive_cases,
                ..
            } => {
                let def_idx = self.actor_defs.len();
                self.actor_defs.push(StoredActorDef {
                    name: name.clone(),
                    init: init.clone(),
                    receive_cases: receive_cases.clone(),
                });
                self.env
                    .set(name.clone(), Value::ActorClass(ActorDefId(def_idx)));
            }
            StmtKind::Break => {
                return Err(EvalError::Break);
            }
            StmtKind::Next => {
                return Err(EvalError::Next);
            }
            StmtKind::Reply(expr) => {
                let val = self.eval_expr(expr)?;
                return Err(EvalError::Reply(val));
            }
            StmtKind::InstanceAssign { field, value } => {
                let val = self.eval_expr(value)?;
                // Write to current actor or instance
                if let Some(actor_id) = self.current_actor {
                    self.actors[actor_id.0].fields.insert(field.clone(), val);
                } else if let Some(instance_id) = self.current_self {
                    self.instances[instance_id.0]
                        .fields
                        .insert(field.clone(), val);
                } else {
                    return Err(EvalError::RuntimeError(
                        "instance variable assignment outside of instance/actor context".into(),
                    ));
                }
            }
            StmtKind::MacroDef { name, params, body } => {
                let macro_id = MacroId(self.macros.len());
                self.macros.push(StoredMacro {
                    params: params.clone(),
                    body: body.clone(),
                });
                self.env.set(name.clone(), Value::Macro(macro_id));
            }
            StmtKind::MacroInvoke { name, args, block } => {
                let macro_id = match self.env.get(name) {
                    Some(Value::Macro(id)) => *id,
                    _ => {
                        return Err(EvalError::UndefinedVariable(format!("@{}", name)));
                    }
                };
                let mac = self.macros[macro_id.0].clone();

                // Build AST argument map: param name -> AST value
                let mut ast_bindings = HashMap::new();
                for (i, param) in mac.params.iter().enumerate() {
                    if i < args.len() {
                        // Store argument expression as AST
                        let ast_id = AstId(self.ast_nodes.len());
                        self.ast_nodes.push(vec![Stmt {
                            kind: StmtKind::Expr(args[i].clone()),
                            span: args[i].span,
                        }]);
                        ast_bindings.insert(param.clone(), ast_id);
                    } else if i == args.len() && block.is_some() {
                        // Block becomes the last parameter
                        let ast_id = AstId(self.ast_nodes.len());
                        self.ast_nodes.push(block.clone().unwrap());
                        ast_bindings.insert(param.clone(), ast_id);
                    }
                }

                // Evaluate macro body with AST bindings in scope
                self.env.push_scope();
                for (param, ast_id) in &ast_bindings {
                    self.env.set(param.clone(), Value::Ast(*ast_id));
                }
                let result = self.eval_block(&mac.body);
                self.env.pop_scope();

                // If the macro body returns an AST, evaluate it
                match result {
                    Ok(Value::Ast(ast_id)) => {
                        let stmts = self.ast_nodes[ast_id.0].clone();
                        self.eval_block(&stmts)?;
                    }
                    Ok(_) => {} // macro returned non-AST, ignore
                    Err(EvalError::Return(Value::Ast(ast_id))) => {
                        let stmts = self.ast_nodes[ast_id.0].clone();
                        self.eval_block(&stmts)?;
                    }
                    Err(e) => return Err(e),
                }
            }
            StmtKind::ExternDef {
                lib_name,
                declarations,
            } => {
                // Register each declared function from the extern block as a NativeFunction
                for decl in declarations {
                    if self
                        .plugin_registry
                        .get_function(lib_name, &decl.name)
                        .is_some()
                    {
                        let nf_id = NativeFunctionId(self.native_functions.len());
                        self.native_functions
                            .push(format!("{}:{}", lib_name, decl.name));
                        self.env
                            .set(decl.name.clone(), Value::NativeFunction(nf_id));
                    }
                    // If plugin or function not found, silently skip (soft failure)
                }
            }
        }
        Ok(())
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, EvalError> {
        match &expr.kind {
            ExprKind::Integer(n) => Ok(Value::Integer(*n)),
            ExprKind::Float(n) => Ok(Value::Float(*n)),
            ExprKind::String(s) => Ok(Value::String(s.clone())),
            ExprKind::Bool(b) => Ok(Value::Bool(*b)),
            ExprKind::Null => Ok(Value::Null),

            ExprKind::Identifier(name) => {
                if name == "None" {
                    return Ok(Self::make_none());
                }
                self.env
                    .get(name)
                    .cloned()
                    .ok_or_else(|| EvalError::UndefinedVariable(name.clone()))
            }

            ExprKind::FString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        FStringPart::Literal(s) => result.push_str(s),
                        FStringPart::Expr(e) => {
                            let val = self.eval_expr(e)?;
                            result.push_str(&self.format_value(&val));
                        }
                    }
                }
                Ok(Value::String(result))
            }

            ExprKind::Symbol(name) => Ok(Value::Symbol(name.clone())),

            ExprKind::Await(inner) => {
                // Synchronous actors: await is a passthrough
                self.eval_expr(inner)
            }

            ExprKind::AstBlock(body) => {
                // Construct AST with $var substitutions
                let substituted = self.substitute_splices(body);
                let ast_id = AstId(self.ast_nodes.len());
                self.ast_nodes.push(substituted);
                Ok(Value::Ast(ast_id))
            }

            ExprKind::Splice(name) => {
                // $var — resolve to the AST value bound in scope
                self.env
                    .get(name)
                    .cloned()
                    .ok_or_else(|| EvalError::UndefinedVariable(format!("${}", name)))
            }

            ExprKind::List(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.eval_expr(elem)?);
                }
                Ok(Value::List(values))
            }

            ExprKind::Closure { params, body } => {
                let id = ClosureId(self.closures.len());
                self.closures.push(StoredClosure {
                    params: params.clone(),
                    body: body.clone(),
                    captured_env: self.env.snapshot(),
                });
                Ok(Value::Closure(id))
            }

            ExprKind::Call { function, args } => self.eval_call(function, args),

            ExprKind::BinaryOp { left, op, right } => {
                // Special handling for pipe operator
                if *op == BinOp::Pipe {
                    return self.eval_pipe(left, right);
                }
                // Special handling for `is` / `is not` — RHS is a type name, not evaluated
                // Special handling for `??` — short-circuit: if left is non-null, return it
                if *op == BinOp::NullCoalesce {
                    let left_val = self.eval_expr(left)?;
                    if !matches!(left_val, Value::Null) {
                        return Ok(left_val);
                    }
                    return self.eval_expr(right);
                }
                // Special handling for `in` / `not in`
                if *op == BinOp::In || *op == BinOp::NotIn {
                    let left_val = self.eval_expr(left)?;
                    let right_val = self.eval_expr(right)?;
                    let result = match &right_val {
                        Value::List(items) => items.iter().any(|item| values_equal(&left_val, item)),
                        Value::Dict(entries) => {
                            if let Value::String(key) = &left_val {
                                entries.iter().any(|(k, _)| k == key)
                            } else {
                                false
                            }
                        }
                        Value::String(s) => {
                            if let Value::String(sub) = &left_val {
                                s.contains(sub.as_str())
                            } else {
                                false
                            }
                        }
                        Value::Range { start, end, inclusive } => {
                            if let Value::Integer(n) = &left_val {
                                if *inclusive {
                                    *n >= *start && *n <= *end
                                } else {
                                    *n >= *start && *n < *end
                                }
                            } else {
                                false
                            }
                        }
                        _ => return Err(EvalError::TypeError(format!(
                            "cannot use 'in' with {:?}", right_val
                        ))),
                    };
                    return Ok(Value::Bool(if *op == BinOp::In { result } else { !result }));
                }
                // Special handling for `is` / `is not` — RHS is a type name, not evaluated
                if *op == BinOp::Is || *op == BinOp::IsNot {
                    let left_val = self.eval_expr(left)?;
                    let type_name = match &right.kind {
                        ExprKind::Identifier(name) => name.clone(),
                        _ => return Err(EvalError::TypeError("is operator requires a type name".into())),
                    };
                    let result = self.value_is_type(&left_val, &type_name);
                    return Ok(Value::Bool(if *op == BinOp::Is { result } else { !result }));
                }
                let lval = self.eval_expr(left)?;
                let rval = self.eval_expr(right)?;
                eval_binary_op(*op, lval, rval)
            }

            ExprKind::UnaryOp { op, operand } => {
                let val = self.eval_expr(operand)?;
                eval_unary_op(*op, val)
            }

            ExprKind::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
            } => {
                let cond = self.eval_expr(condition)?;
                if cond.is_truthy() {
                    self.eval_block(then_branch)
                } else {
                    for (elsif_cond, elsif_body) in elsif_branches {
                        let cond = self.eval_expr(elsif_cond)?;
                        if cond.is_truthy() {
                            return self.eval_block(elsif_body);
                        }
                    }
                    if let Some(else_body) = else_branch {
                        self.eval_block(else_body)
                    } else {
                        Ok(Value::Null)
                    }
                }
            }

            ExprKind::Grouped(inner) => self.eval_expr(inner),

            ExprKind::InstanceVar(field) => {
                // Check actor first, then instance
                if let Some(actor_id) = self.current_actor {
                    return self.actors[actor_id.0]
                        .fields
                        .get(field)
                        .cloned()
                        .ok_or_else(|| EvalError::UndefinedVariable(format!(".{}", field)));
                }
                let instance_id = self
                    .current_self
                    .ok_or_else(|| EvalError::RuntimeError("no self in scope".into()))?;
                let instance = &self.instances[instance_id.0];
                instance
                    .fields
                    .get(field)
                    .cloned()
                    .ok_or_else(|| EvalError::UndefinedVariable(format!(".{}", field)))
            }

            ExprKind::Dict(entries) => {
                let mut pairs = Vec::new();
                for (key_expr, val_expr) in entries {
                    let key = match &key_expr.kind {
                        ExprKind::Identifier(name) => name.clone(),
                        _ => {
                            let k = self.eval_expr(key_expr)?;
                            match k {
                                Value::String(s) => s,
                                _ => {
                                    return Err(EvalError::TypeError(
                                        "dict key must be a string or identifier".into(),
                                    ));
                                }
                            }
                        }
                    };
                    let val = self.eval_expr(val_expr)?;
                    pairs.push((key, val));
                }
                Ok(Value::Dict(pairs))
            }

            ExprKind::Range {
                start,
                end,
                inclusive,
            } => {
                let start_val = self.eval_expr(start)?;
                let end_val = self.eval_expr(end)?;
                match (&start_val, &end_val) {
                    (Value::Integer(s), Value::Integer(e)) => Ok(Value::Range {
                        start: *s,
                        end: *e,
                        inclusive: *inclusive,
                    }),
                    _ => Err(EvalError::TypeError("range bounds must be integers".into())),
                }
            }

            ExprKind::Index { object, index } => {
                let obj = self.eval_expr(object)?;
                let idx = self.eval_expr(index)?;
                match (&obj, &idx) {
                    (Value::List(items), Value::Integer(i)) => {
                        let i = if *i < 0 {
                            items.len() as i64 + i
                        } else {
                            *i
                        } as usize;
                        Ok(items.get(i).cloned().unwrap_or(Value::Null))
                    }
                    (Value::Dict(entries), Value::String(key)) => Ok(entries
                        .iter()
                        .find(|(k, _)| k == key)
                        .map(|(_, v)| v.clone())
                        .unwrap_or(Value::Null)),
                    (Value::String(s), Value::Integer(i)) => {
                        let i = if *i < 0 {
                            s.len() as i64 + i
                        } else {
                            *i
                        } as usize;
                        Ok(s.chars()
                            .nth(i)
                            .map(|c| Value::String(c.to_string()))
                            .unwrap_or(Value::Null))
                    }
                    _ => Err(EvalError::TypeError("invalid index operation".into())),
                }
            }

            ExprKind::MemberAccess { object, field } => {
                let obj = self.eval_expr(object)?;
                match &obj {
                    Value::Instance(id) => {
                        let instance = &self.instances[id.0];
                        if let Some(val) = instance.fields.get(field) {
                            Ok(val.clone())
                        } else {
                            Err(EvalError::UndefinedVariable(format!(
                                "instance has no field '{}'",
                                field
                            )))
                        }
                    }
                    Value::Module(id) => {
                        let module = &self.modules[id.0];
                        if let Some(val) = module.bindings.get(field) {
                            Ok(val.clone())
                        } else {
                            Err(EvalError::UndefinedVariable(format!(
                                "module has no member '{}'",
                                field
                            )))
                        }
                    }
                    Value::Type(TypeInfo::Enum(enum_id)) => {
                        let e = &self.enums[enum_id.0].clone();
                        // Check if field is a variant name
                        if let Some((vi, variant)) = e.variants.iter().enumerate().find(|(_, v)| v.name == *field) {
                            if variant.fields.is_empty() {
                                // Singleton variant — return directly
                                return Ok(Value::EnumVariant {
                                    enum_id: *enum_id,
                                    variant_index: vi,
                                    fields: vec![],
                                });
                            } else {
                                // Data-carrying variant — return a Type so call_method can construct
                                return Ok(Value::Type(TypeInfo::EnumVariant(*enum_id, vi)));
                            }
                        }
                        // Fall through to Type methods (.name, .fields)
                        match field.as_str() {
                            "name" => Ok(Value::String(e.name.clone())),
                            _ => Err(EvalError::TypeError(format!("enum '{}' has no variant or field '{}'", e.name, field))),
                        }
                    }
                    Value::Type(info) => {
                        match field.as_str() {
                            "name" => Ok(Value::String(self.type_info_name(info))),
                            "fields" => {
                                match info {
                                    TypeInfo::Class(id) => {
                                        let class = &self.classes[id.0];
                                        let field_list: Vec<Value> = class.needs.iter().map(|(name, type_ann)| {
                                            Value::List(vec![
                                                Value::Symbol(name.clone()),
                                                Value::String(type_ann.clone().unwrap_or_else(|| "Any".to_string())),
                                            ])
                                        }).collect();
                                        Ok(Value::List(field_list))
                                    }
                                    _ => Ok(Value::List(vec![])),
                                }
                            }
                            _ => Err(EvalError::TypeError(format!("Type has no field '{}'", field))),
                        }
                    }
                    _ => Err(EvalError::TypeError(format!(
                        "cannot access field '{}' on this value",
                        field
                    ))),
                }
            }

            ExprKind::NullSafeMemberAccess { object, field } => {
                let obj = self.eval_expr(object)?;
                if matches!(obj, Value::Null) {
                    return Ok(Value::Null);
                }
                // Re-use MemberAccess evaluation by wrapping the already-evaluated object
                // as a literal identifier that resolves to the same value.
                // Simpler: just handle the key cases inline.
                match &obj {
                    Value::Instance(id) => {
                        let instance = &self.instances[id.0];
                        if let Some(val) = instance.fields.get(field) {
                            Ok(val.clone())
                        } else {
                            Ok(Value::Null)
                        }
                    }
                    Value::Dict(entries) => Ok(entries
                        .iter()
                        .find(|(k, _)| k == field)
                        .map(|(_, v)| v.clone())
                        .unwrap_or(Value::Null)),
                    _ => Ok(Value::Null),
                }
            }

            ExprKind::Match { subject, cases } => {
                let val = self.eval_expr(subject)?;
                for case in cases {
                    if let Some(bindings) = self.match_pattern(&case.pattern, &val) {
                        self.env.push_scope();
                        for (name, bound_val) in bindings {
                            self.env.set(name, bound_val);
                        }
                        let result = self.eval_block(&case.body);
                        self.env.pop_scope();
                        return result;
                    }
                }
                Ok(Value::Null) // no match
            }

            ExprKind::TryCatch {
                body,
                catches,
                ensure,
            } => {
                let result = self.eval_block(body);

                let value = match result {
                    Err(EvalError::Raise(val) | EvalError::RequiresFailed(val)) => {
                        if let Some(catch) = catches.first() {
                            self.env.push_scope();
                            if let Some(var) = &catch.var_name {
                                self.env.set(var.clone(), val.clone());
                            }
                            let catch_result = self.eval_block(&catch.body);
                            self.env.pop_scope();
                            catch_result?
                        } else {
                            if let Some(ensure_body) = ensure {
                                self.eval_block(ensure_body)?;
                            }
                            return Err(EvalError::Raise(val));
                        }
                    }
                    Err(e) => {
                        if let Some(ensure_body) = ensure {
                            self.eval_block(ensure_body)?;
                        }
                        return Err(e);
                    }
                    Ok(v) => v,
                };

                if let Some(ensure_body) = ensure {
                    self.eval_block(ensure_body)?;
                }
                Ok(value)
            }
        }
    }

    /// Substitute $var splices in AST with stored AST nodes
    fn substitute_splices(&self, stmts: &[Stmt]) -> Vec<Stmt> {
        stmts
            .iter()
            .flat_map(|stmt| self.substitute_stmt(stmt))
            .collect()
    }

    fn substitute_stmt(&self, stmt: &Stmt) -> Vec<Stmt> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                // Check if the expression is a splice
                if let ExprKind::Splice(name) = &expr.kind {
                    if let Some(Value::Ast(ast_id)) = self.env.get(name) {
                        return self.ast_nodes[ast_id.0].clone();
                    }
                }
                vec![Stmt {
                    kind: StmtKind::Expr(self.substitute_expr(expr)),
                    span: stmt.span,
                }]
            }
            _ => {
                // For other statement types, substitute expressions within them
                vec![self.substitute_stmt_inner(stmt)]
            }
        }
    }

    fn substitute_stmt_inner(&self, stmt: &Stmt) -> Stmt {
        let kind = match &stmt.kind {
            StmtKind::Expr(expr) => StmtKind::Expr(self.substitute_expr(expr)),
            StmtKind::Raise(expr) => StmtKind::Raise(self.substitute_expr(expr)),
            StmtKind::Return(Some(expr)) => StmtKind::Return(Some(self.substitute_expr(expr))),
            StmtKind::Assign { name, value } => StmtKind::Assign {
                name: name.clone(),
                value: self.substitute_expr(value),
            },
            StmtKind::CompoundAssign { name, op, value } => StmtKind::CompoundAssign {
                name: name.clone(),
                op: *op,
                value: self.substitute_expr(value),
            },
            StmtKind::Let { name, value } => StmtKind::Let {
                name: name.clone(),
                value: self.substitute_expr(value),
            },
            StmtKind::InstanceAssign { field, value } => StmtKind::InstanceAssign {
                field: field.clone(),
                value: self.substitute_expr(value),
            },
            StmtKind::IndexAssign {
                object,
                index,
                value,
            } => StmtKind::IndexAssign {
                object: self.substitute_expr(object),
                index: self.substitute_expr(index),
                value: self.substitute_expr(value),
            },
            StmtKind::MacroInvoke { name, args, block } => StmtKind::MacroInvoke {
                name: name.clone(),
                args: args.iter().map(|a| self.substitute_expr(a)).collect(),
                block: block
                    .as_ref()
                    .map(|b| self.substitute_splices(b)),
            },
            StmtKind::For { var, iterable, body } => StmtKind::For {
                var: var.clone(),
                iterable: self.substitute_expr(iterable),
                body: self.substitute_splices(body),
            },
            StmtKind::While { condition, body } => StmtKind::While {
                condition: self.substitute_expr(condition),
                body: self.substitute_splices(body),
            },
            other => other.clone(),
        };
        Stmt {
            kind,
            span: stmt.span,
        }
    }

    fn substitute_expr(&self, expr: &Expr) -> Expr {
        match &expr.kind {
            ExprKind::Splice(name) => {
                if let Some(Value::Ast(ast_id)) = self.env.get(name) {
                    let stmts = &self.ast_nodes[ast_id.0];
                    if stmts.len() == 1 {
                        if let StmtKind::Expr(inner) = &stmts[0].kind {
                            return inner.clone();
                        }
                    }
                }
                expr.clone()
            }
            ExprKind::UnaryOp { op, operand } => Expr {
                kind: ExprKind::UnaryOp {
                    op: *op,
                    operand: Box::new(self.substitute_expr(operand)),
                },
                span: expr.span,
            },
            ExprKind::BinaryOp { left, op, right } => Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(self.substitute_expr(left)),
                    op: *op,
                    right: Box::new(self.substitute_expr(right)),
                },
                span: expr.span,
            },
            ExprKind::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
            } => Expr {
                kind: ExprKind::If {
                    condition: Box::new(self.substitute_expr(condition)),
                    then_branch: self.substitute_splices(then_branch),
                    elsif_branches: elsif_branches.clone(),
                    else_branch: else_branch.as_ref().map(|b| self.substitute_splices(b)),
                },
                span: expr.span,
            },
            ExprKind::FString(parts) => Expr {
                kind: ExprKind::FString(
                    parts
                        .iter()
                        .map(|part| match part {
                            FStringPart::Literal(s) => FStringPart::Literal(s.clone()),
                            FStringPart::Expr(e) => FStringPart::Expr(self.substitute_expr(e)),
                        })
                        .collect(),
                ),
                span: expr.span,
            },
            ExprKind::Grouped(inner) => Expr {
                kind: ExprKind::Grouped(Box::new(self.substitute_expr(inner))),
                span: expr.span,
            },
            ExprKind::Call { function, args } => Expr {
                kind: ExprKind::Call {
                    function: Box::new(self.substitute_expr(function)),
                    args: args
                        .iter()
                        .map(|a| Arg {
                            name: a.name.clone(),
                            value: self.substitute_expr(&a.value),
                        })
                        .collect(),
                },
                span: expr.span,
            },
            ExprKind::MemberAccess { object, field } => Expr {
                kind: ExprKind::MemberAccess {
                    object: Box::new(self.substitute_expr(object)),
                    field: field.clone(),
                },
                span: expr.span,
            },
            ExprKind::Index { object, index } => Expr {
                kind: ExprKind::Index {
                    object: Box::new(self.substitute_expr(object)),
                    index: Box::new(self.substitute_expr(index)),
                },
                span: expr.span,
            },
            ExprKind::Await(inner) => Expr {
                kind: ExprKind::Await(Box::new(self.substitute_expr(inner))),
                span: expr.span,
            },
            ExprKind::List(items) => Expr {
                kind: ExprKind::List(
                    items.iter().map(|e| self.substitute_expr(e)).collect(),
                ),
                span: expr.span,
            },
            ExprKind::TryCatch {
                body,
                catches,
                ensure,
            } => Expr {
                kind: ExprKind::TryCatch {
                    body: self.substitute_splices(body),
                    catches: catches
                        .iter()
                        .map(|c| opal_parser::CatchClause {
                            error_type: c.error_type.clone(),
                            var_name: c.var_name.clone(),
                            body: self.substitute_splices(&c.body),
                        })
                        .collect(),
                    ensure: ensure.as_ref().map(|b| self.substitute_splices(b)),
                },
                span: expr.span,
            },
            _ => expr.clone(),
        }
    }

    /// Try to match a value against a pattern. Returns bindings on success.
    fn match_pattern(&self, pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
        match pattern {
            Pattern::Wildcard => Some(vec![]),
            Pattern::Identifier(name) => Some(vec![(name.clone(), value.clone())]),
            Pattern::Constructor(name, sub_patterns) => {
                match name.as_str() {
                    "Ok" => {
                        if let Some(inner) = Self::is_ok(value) {
                            if sub_patterns.len() == 1 {
                                return self.match_pattern(&sub_patterns[0], inner);
                            }
                        }
                        None
                    }
                    "Error" | "Err" => {
                        if let Some(inner) = Self::is_err(value) {
                            if sub_patterns.len() == 1 {
                                return self.match_pattern(&sub_patterns[0], inner);
                            }
                        }
                        None
                    }
                    "Some" => {
                        if let Value::EnumVariant { enum_id, variant_index: 0, fields } = value {
                            if enum_id.0 == 1 && sub_patterns.len() == 1 {
                                return self.match_pattern(&sub_patterns[0], &fields[0]);
                            }
                        }
                        None
                    }
                    "None" if sub_patterns.is_empty() => {
                        if let Value::EnumVariant { enum_id, variant_index: 1, fields } = value {
                            if enum_id.0 == 1 && fields.is_empty() {
                                return Some(vec![]);
                            }
                        }
                        None
                    }
                    _ => None,
                }
            }
            Pattern::List(element_patterns, rest_pattern) => {
                if let Value::List(items) = value {
                    if let Some(rest) = rest_pattern {
                        // [head, ... | tail] pattern — need at least as many items as element patterns
                        if items.len() < element_patterns.len() {
                            return None;
                        }
                        let mut all_bindings = vec![];
                        for (pat, val) in element_patterns.iter().zip(items.iter()) {
                            match self.match_pattern(pat, val) {
                                Some(bindings) => all_bindings.extend(bindings),
                                None => return None,
                            }
                        }
                        // Rest gets the remaining elements
                        let tail = Value::List(items[element_patterns.len()..].to_vec());
                        match self.match_pattern(rest, &tail) {
                            Some(bindings) => {
                                all_bindings.extend(bindings);
                                Some(all_bindings)
                            }
                            None => None,
                        }
                    } else {
                        // Exact match — must have same number of elements
                        if items.len() != element_patterns.len() {
                            return None;
                        }
                        let mut all_bindings = vec![];
                        for (pat, val) in element_patterns.iter().zip(items.iter()) {
                            match self.match_pattern(pat, val) {
                                Some(bindings) => all_bindings.extend(bindings),
                                None => return None,
                            }
                        }
                        Some(all_bindings)
                    }
                } else {
                    None
                }
            }
            Pattern::EnumVariant(enum_name, variant_name, sub_patterns) => {
                if let Value::EnumVariant { enum_id, variant_index, fields } = value {
                    let e = &self.enums[enum_id.0];
                    if e.name != *enum_name {
                        return None;
                    }
                    let v = &e.variants[*variant_index];
                    if v.name != *variant_name {
                        return None;
                    }
                    if sub_patterns.len() != fields.len() {
                        return None;
                    }
                    let mut all_bindings = vec![];
                    for (pat, val) in sub_patterns.iter().zip(fields.iter()) {
                        match self.match_pattern(pat, val) {
                            Some(bindings) => all_bindings.extend(bindings),
                            None => return None,
                        }
                    }
                    Some(all_bindings)
                } else {
                    None
                }
            }
            Pattern::Literal(expr) => {
                // Compare literal values
                match &expr.kind {
                    ExprKind::Integer(n) => match value {
                        Value::Integer(v) if v == n => Some(vec![]),
                        _ => None,
                    },
                    ExprKind::Float(n) => match value {
                        Value::Float(v) if v == n => Some(vec![]),
                        _ => None,
                    },
                    ExprKind::String(s) => match value {
                        Value::String(v) if v == s => Some(vec![]),
                        _ => None,
                    },
                    ExprKind::Bool(b) => match value {
                        Value::Bool(v) if v == b => Some(vec![]),
                        _ => None,
                    },
                    ExprKind::Null => match value {
                        Value::Null => Some(vec![]),
                        _ => None,
                    },
                    ExprKind::Symbol(s) => match value {
                        Value::Symbol(v) if v == s => Some(vec![]),
                        _ => None,
                    },
                    _ => None,
                }
            }
        }
    }

    fn eval_call(&mut self, function: &Expr, args: &[Arg]) -> Result<Value, EvalError> {
        // Null-safe method call: expr?.method(args) — if null, return null
        if let ExprKind::NullSafeMemberAccess { object, field } = &function.kind {
            let obj = self.eval_expr(object)?;
            if matches!(obj, Value::Null) {
                return Ok(Value::Null);
            }
            let mut eval_args = Vec::new();
            for arg in args {
                eval_args.push((arg.name.clone(), self.eval_expr(&arg.value)?));
            }
            return self.call_method(obj, field, eval_args);
        }
        // Method call: expr.method(args)
        if let ExprKind::MemberAccess { object, field } = &function.kind {
            let obj = self.eval_expr(object)?;
            let mut eval_args = Vec::new();
            for arg in args {
                eval_args.push((arg.name.clone(), self.eval_expr(&arg.value)?));
            }
            return self.call_method(obj, field, eval_args);
        }

        // Self method call: .method(args) inside a class method
        if let ExprKind::InstanceVar(method_name) = &function.kind {
            let instance_id = self
                .current_self
                .ok_or_else(|| EvalError::RuntimeError("no self in scope".into()))?;
            let obj = Value::Instance(instance_id);
            let mut eval_args = Vec::new();
            for arg in args {
                eval_args.push((arg.name.clone(), self.eval_expr(&arg.value)?));
            }
            return self.call_method(obj, method_name, eval_args);
        }

        // Regular function call: name(args)
        let func_name = match &function.kind {
            ExprKind::Identifier(name) => name.clone(),
            _ => {
                return Err(EvalError::TypeError(
                    "only named function calls supported".into(),
                ));
            }
        };

        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(&arg.value)?);
        }

        // Builtin constructors and functions
        match func_name.as_str() {
            "Ok" if arg_values.len() == 1 => {
                return Ok(Self::make_ok(arg_values.into_iter().next().unwrap()));
            }
            "Error" | "Err" if arg_values.len() == 1 => {
                return Ok(Self::make_err(arg_values.into_iter().next().unwrap()));
            }
            "Some" if arg_values.len() == 1 => {
                return Ok(Self::make_some(arg_values.into_iter().next().unwrap()));
            }
            "typeof" if arg_values.len() == 1 => {
                let type_info = self.value_type_info(&arg_values[0]);
                return Ok(Value::Type(type_info));
            }
            "eval" if arg_values.len() == 1 => {
                // Opal AST evaluator — only accepts Value::Ast (not strings).
                // This is a metaprogramming primitive like Elixir's Code.eval_quoted.
                // Evaluates in a child scope: reads parent vars but writes don't leak.
                match &arg_values[0] {
                    Value::Ast(ast_id) => {
                        let stmts = self.ast_nodes[ast_id.0].clone();
                        // Snapshot env so assignments in eval don't leak
                        let saved_env = self.env.snapshot();
                        self.env.push_scope();
                        let mut result = Value::Null;
                        let mut err = None;
                        for stmt in &stmts {
                            match &stmt.kind {
                                StmtKind::Expr(expr) => {
                                    match self.eval_expr(expr) {
                                        Ok(v) => result = v,
                                        Err(e) => { err = Some(e); break; }
                                    }
                                }
                                _ => {
                                    match self.eval_stmt(stmt) {
                                        Ok(()) => result = Value::Null,
                                        Err(e) => { err = Some(e); break; }
                                    }
                                }
                            }
                        }
                        // Restore env (discarding eval's mutations)
                        self.env = saved_env;
                        if let Some(e) = err {
                            return Err(e);
                        }
                        return Ok(result);
                    }
                    _ => {
                        return Err(EvalError::TypeError(
                            "eval() requires an AST value (from ast ... end block)".into(),
                        ));
                    }
                }
            }
            "annotations" if arg_values.len() == 1 => {
                let anns = match &arg_values[0] {
                    Value::Function(id) => self.functions[id.0].annotations.clone(),
                    Value::MultiFunction(ids) => {
                        if let Some(id) = ids.last() {
                            self.functions[id.0].annotations.clone()
                        } else {
                            vec![]
                        }
                    }
                    _ => vec![],
                };
                let ann_list: Vec<Value> = anns.iter().map(|entries| {
                    Value::Dict(entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                }).collect();
                return Ok(Value::List(ann_list));
            }
            _ => {}
        }

        // Intercept print/println to use format_value for proper display
        if func_name == "print" || func_name == "println" {
            let output: Vec<String> = arg_values.iter().map(|v| self.format_value(v)).collect();
            writeln!(self.writer, "{}", output.join(" ")).ok();
            return Ok(Value::Null);
        }

        // Try stdlib builtins
        if let Some(result) = opal_stdlib::call_builtin(&func_name, &arg_values, &mut self.writer) {
            return match result {
                Ok(opal_stdlib::BuiltinResult::Value(v)) => Ok(v),
                Ok(opal_stdlib::BuiltinResult::Void) => Ok(Value::Null),
                Err(e) => Err(EvalError::RuntimeError(e)),
            };
        }

        // Try native functions (from extern blocks)
        if let Some(Value::NativeFunction(nf_id)) = self.env.get(&func_name).cloned() {
            let key = self.native_functions[nf_id.0].clone();

            // Special handling for http:serve — needs interpreter access for closures
            if key == "http:serve" {
                return self.serve_http(&arg_values);
            }

            let parts: Vec<&str> = key.split(':').collect();
            let (plugin, func) = (parts[0], parts[1]);
            if let Some(native_fn) = self.plugin_registry.get_function(plugin, func) {
                // Reborrow: native_fn borrows plugin_registry immutably, writer needs &mut.
                // Since NativeFunction is behind &, we call it directly — plugin_registry
                // and writer are disjoint fields so Rust allows this split borrow.
                return match native_fn(&arg_values, &mut self.writer) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(EvalError::RuntimeError(e)),
                };
            }
        }

        // Try user-defined functions or closures
        if let Some(val) = self.env.get(&func_name).cloned() {
            match val {
                Value::Function(id) => {
                    return self.call_function(id, &func_name, arg_values).map_err(|e| match e {
                        EvalError::RequiresFailed(v) => EvalError::Raise(v),
                        other => other,
                    });
                }
                Value::MultiFunction(ids) => {
                    return self.dispatch_multi(&ids, &func_name, arg_values)
                }
                Value::Closure(id) => return self.call_closure(id, arg_values),
                _ => {}
            }
        }

        Err(EvalError::UndefinedVariable(func_name))
    }

    fn call_method(
        &mut self,
        obj: Value,
        method: &str,
        named_args: Vec<(Option<String>, Value)>,
    ) -> Result<Value, EvalError> {
        // Extract positional values for most methods
        let args: Vec<Value> = named_args.iter().map(|(_, v)| v.clone()).collect();
        match (&obj, method) {
            // List methods
            (Value::List(items), "length") => Ok(Value::Integer(items.len() as i64)),
            (Value::List(items), "push") => {
                if args.len() != 1 {
                    return Err(EvalError::TypeError(
                        "push() takes exactly 1 argument".into(),
                    ));
                }
                let mut new_list = items.clone();
                new_list.push(args.into_iter().next().unwrap());
                Ok(Value::List(new_list))
            }
            (Value::List(items), "get") => {
                if args.len() != 1 {
                    return Err(EvalError::TypeError(
                        "get() takes exactly 1 argument".into(),
                    ));
                }
                match &args[0] {
                    Value::Integer(idx) => {
                        let idx = *idx as usize;
                        Ok(items.get(idx).cloned().unwrap_or(Value::Null))
                    }
                    _ => Err(EvalError::TypeError("list index must be an integer".into())),
                }
            }
            (Value::List(items), "map") => {
                if args.len() != 1 {
                    return Err(EvalError::TypeError(
                        "map() takes exactly 1 argument (a closure)".into(),
                    ));
                }
                let closure_id = match &args[0] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::TypeError(
                            "map() argument must be a closure".into(),
                        ));
                    }
                };
                let mut result = Vec::new();
                for item in items.clone() {
                    result.push(self.call_closure(closure_id, vec![item])?);
                }
                Ok(Value::List(result))
            }
            (Value::List(items), "filter") => {
                if args.len() != 1 {
                    return Err(EvalError::TypeError(
                        "filter() takes exactly 1 argument (a closure)".into(),
                    ));
                }
                let closure_id = match &args[0] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::TypeError(
                            "filter() argument must be a closure".into(),
                        ));
                    }
                };
                let mut result = Vec::new();
                for item in items.clone() {
                    let keep = self.call_closure(closure_id, vec![item.clone()])?;
                    if keep.is_truthy() {
                        result.push(item);
                    }
                }
                Ok(Value::List(result))
            }
            (Value::List(items), "reduce") => {
                if args.len() != 2 {
                    return Err(EvalError::TypeError(
                        "reduce() takes 2 arguments (initial, closure)".into(),
                    ));
                }
                let initial = args[0].clone();
                let closure_id = match &args[1] {
                    Value::Closure(id) => *id,
                    _ => {
                        return Err(EvalError::TypeError(
                            "reduce() second argument must be a closure".into(),
                        ));
                    }
                };
                let mut acc = initial;
                for item in items.clone() {
                    acc = self.call_closure(closure_id, vec![acc, item])?;
                }
                Ok(acc)
            }
            // String methods
            (Value::String(s), "length") => Ok(Value::Integer(s.len() as i64)),
            (Value::String(s), "split") => {
                if args.len() != 1 {
                    return Err(EvalError::TypeError(
                        "split() takes exactly 1 argument".into(),
                    ));
                }
                let sep = match &args[0] {
                    Value::String(sep) => sep.clone(),
                    _ => {
                        return Err(EvalError::TypeError(
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
            (Value::String(s), "trim") => Ok(Value::String(s.trim().to_string())),
            (Value::String(s), "contains") => {
                if args.len() != 1 {
                    return Err(EvalError::TypeError(
                        "contains() takes exactly 1 argument".into(),
                    ));
                }
                let sub = match &args[0] {
                    Value::String(sub) => sub.clone(),
                    _ => {
                        return Err(EvalError::TypeError(
                            "contains() argument must be a string".into(),
                        ));
                    }
                };
                Ok(Value::Bool(s.contains(&sub)))
            }
            (Value::String(s), "replace") => {
                if args.len() != 2 {
                    return Err(EvalError::TypeError(
                        "replace() takes exactly 2 arguments".into(),
                    ));
                }
                let old = match &args[0] {
                    Value::String(o) => o.clone(),
                    _ => {
                        return Err(EvalError::TypeError(
                            "replace() first argument must be a string".into(),
                        ));
                    }
                };
                let new = match &args[1] {
                    Value::String(n) => n.clone(),
                    _ => {
                        return Err(EvalError::TypeError(
                            "replace() second argument must be a string".into(),
                        ));
                    }
                };
                Ok(Value::String(s.replace(&old, &new)))
            }
            (Value::String(s), "starts_with") => {
                if args.len() != 1 {
                    return Err(EvalError::TypeError(
                        "starts_with() takes exactly 1 argument".into(),
                    ));
                }
                let prefix = match &args[0] {
                    Value::String(p) => p.clone(),
                    _ => {
                        return Err(EvalError::TypeError(
                            "starts_with() argument must be a string".into(),
                        ));
                    }
                };
                Ok(Value::Bool(s.starts_with(&prefix)))
            }
            (Value::String(s), "ends_with") => {
                if args.len() != 1 {
                    return Err(EvalError::TypeError(
                        "ends_with() takes exactly 1 argument".into(),
                    ));
                }
                let suffix = match &args[0] {
                    Value::String(sf) => sf.clone(),
                    _ => {
                        return Err(EvalError::TypeError(
                            "ends_with() argument must be a string".into(),
                        ));
                    }
                };
                Ok(Value::Bool(s.ends_with(&suffix)))
            }
            (Value::String(s), "to_upper") => Ok(Value::String(s.to_uppercase())),
            (Value::String(s), "to_lower") => Ok(Value::String(s.to_lowercase())),
            (Value::String(s), "chars") => {
                let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
                Ok(Value::List(chars))
            }

            // Dict methods
            (Value::Dict(entries), "length") => Ok(Value::Integer(entries.len() as i64)),
            (Value::Dict(entries), "get") => {
                if args.len() != 1 {
                    return Err(EvalError::TypeError(
                        "get() takes exactly 1 argument".into(),
                    ));
                }
                let key = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(EvalError::TypeError("dict key must be a string".into())),
                };
                Ok(entries
                    .iter()
                    .find(|(k, _)| k == &key)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Value::Null))
            }
            (Value::Dict(entries), "keys") => {
                let keys: Vec<Value> = entries
                    .iter()
                    .map(|(k, _)| Value::String(k.clone()))
                    .collect();
                Ok(Value::List(keys))
            }
            (Value::Dict(entries), "values") => {
                let values: Vec<Value> = entries.iter().map(|(_, v)| v.clone()).collect();
                Ok(Value::List(values))
            }
            (Value::Dict(entries), "set") => {
                if args.len() != 2 {
                    return Err(EvalError::TypeError(
                        "set() takes exactly 2 arguments (key, value)".into(),
                    ));
                }
                let key = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(EvalError::TypeError("dict key must be a string".into())),
                };
                let value = args[1].clone();
                let mut new_entries = entries.clone();
                if let Some(entry) = new_entries.iter_mut().find(|(k, _)| k == &key) {
                    entry.1 = value;
                } else {
                    new_entries.push((key, value));
                }
                Ok(Value::Dict(new_entries))
            }

            // Range methods
            (
                Value::Range {
                    start,
                    end,
                    inclusive,
                },
                "to_list",
            ) => {
                let end_val = if *inclusive { end + 1 } else { *end };
                let items: Vec<Value> = (*start..end_val).map(Value::Integer).collect();
                Ok(Value::List(items))
            }

            // Module method calls (e.g., Math.pi())
            (Value::Module(module_id), _) => {
                let module = self.modules[module_id.0].clone();
                if let Some(val) = module.bindings.get(method) {
                    return match val {
                        Value::Function(fn_id) => self.call_function(*fn_id, method, args),
                        // For non-function values like Math.pi(), return the value directly
                        other => Ok(other.clone()),
                    };
                }
                return Err(EvalError::UndefinedVariable(format!(
                    "{}.{}",
                    module.name, method
                )));
            }

            // Actor .new()
            (Value::ActorClass(def_id), "new") => {
                let def_idx = def_id.0;
                let def = self.actor_defs[def_idx].clone();
                let actor_id = ActorId(self.actors.len());
                self.actors.push(StoredActorInstance {
                    def_idx,
                    fields: HashMap::new(),
                });
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
                return Ok(Value::Actor(actor_id));
            }

            // Actor .send(:msg)
            (Value::Actor(actor_id), "send") => {
                if args.len() != 1 {
                    return Err(EvalError::TypeError(
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
                            Err(EvalError::Reply(val)) => {
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
                return Ok(reply_val);
            }

            // Class methods
            (Value::Class(class_id), "new") => {
                let class = self.classes[class_id.0].clone();
                let mut fields = HashMap::new();

                // Match named args to needs declarations
                for (need_name, _) in &class.needs {
                    // Try named arg first
                    let value = named_args
                        .iter()
                        .find(|(name, _)| name.as_deref() == Some(need_name.as_str()))
                        .map(|(_, v)| v.clone());
                    if let Some(val) = value {
                        fields.insert(need_name.clone(), val);
                    } else {
                        // Try positional
                        let idx = class
                            .needs
                            .iter()
                            .position(|(n, _)| n == need_name)
                            .unwrap();
                        if idx < args.len() {
                            fields.insert(need_name.clone(), args[idx].clone());
                        } else {
                            return Err(EvalError::TypeError(format!(
                                "missing required field '{}' in .new()",
                                need_name
                            )));
                        }
                    }
                }

                let instance_id = InstanceId(self.instances.len());
                self.instances.push(StoredInstance {
                    class_id: *class_id,
                    fields,
                });
                Ok(Value::Instance(instance_id))
            }

            // Instance methods — dispatch to class
            (Value::Instance(instance_id), _) => {
                let instance = self.instances[instance_id.0].clone();
                let class = self.classes[instance.class_id.0].clone();

                // Find method in class — dispatch by name + arity + type
                let method_fn = class
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
                    .or_else(|| class.methods.iter().find(|m| m.name == method));
                if let Some(func) = method_fn {
                    let func = func.clone();
                    if args.len() != func.params.len() {
                        return Err(EvalError::TypeError(format!(
                            "{}() expected {} arguments, got {}",
                            method,
                            func.params.len(),
                            args.len()
                        )));
                    }

                    // Set self and push scope
                    let prev_self = self.current_self;
                    self.current_self = Some(*instance_id);
                    self.env.push_scope();
                    for (param_name, arg_val) in func.params.iter().zip(args) {
                        self.env.set(String::clone(param_name), arg_val);
                    }

                    let result = self.eval_block(&func.body);
                    self.env.pop_scope();
                    self.current_self = prev_self;

                    match result {
                        Ok(val) => Ok(val),
                        Err(EvalError::Return(val)) => Ok(val),
                        Err(e) => Err(e),
                    }
                } else {
                    Err(EvalError::TypeError(format!(
                        "no method '{}' on instance of class",
                        method
                    )))
                }
            }

            // Enum variant construction: EnumName.Variant(args)
            (Value::Type(TypeInfo::Enum(enum_id)), _) => {
                let e = self.enums[enum_id.0].clone();
                if let Some((vi, variant)) = e.variants.iter().enumerate().find(|(_, v)| v.name == method) {
                    if variant.fields.is_empty() && args.is_empty() {
                        return Ok(Value::EnumVariant {
                            enum_id: *enum_id,
                            variant_index: vi,
                            fields: vec![],
                        });
                    }
                    if args.len() != variant.fields.len() {
                        return Err(EvalError::TypeError(format!(
                            "{}.{}() expected {} arguments, got {}",
                            e.name, variant.name, variant.fields.len(), args.len()
                        )));
                    }
                    return Ok(Value::EnumVariant {
                        enum_id: *enum_id,
                        variant_index: vi,
                        fields: args,
                    });
                }
                return Err(EvalError::TypeError(format!("enum '{}' has no variant '{}'", e.name, method)));
            }
            // Enum variant method calls
            (Value::EnumVariant { enum_id, .. }, _) => {
                let e = self.enums[enum_id.0].clone();
                if let Some(func) = e.methods.iter().find(|m| m.name == method) {
                    let func = func.clone();
                    // Bind `self` to the enum variant value
                    self.env.push_scope();
                    self.env.set("self".to_string(), obj.clone());
                    for (param_name, arg_val) in func.params.iter().zip(args) {
                        self.env.set(param_name.clone(), arg_val);
                    }
                    let result = self.eval_block(&func.body);
                    self.env.pop_scope();
                    return match result {
                        Ok(val) => Ok(val),
                        Err(EvalError::Return(val)) => Ok(val),
                        Err(e) => Err(e),
                    };
                }
                return Err(EvalError::TypeError(format!("enum '{}' has no method '{}'", e.name, method)));
            }
            // Type methods
            (Value::Type(info), _) => {
                match method {
                    "name" => {
                        let name = self.type_info_name(info);
                        Ok(Value::String(name))
                    }
                    "fields" => {
                        match info {
                            TypeInfo::Class(id) => {
                                let class = &self.classes[id.0];
                                let field_list: Vec<Value> = class.needs.iter().map(|(name, type_ann)| {
                                    Value::List(vec![
                                        Value::Symbol(name.clone()),
                                        Value::String(type_ann.clone().unwrap_or_else(|| "Any".to_string())),
                                    ])
                                }).collect();
                                Ok(Value::List(field_list))
                            }
                            _ => Ok(Value::List(vec![])),
                        }
                    }
                    _ => Err(EvalError::RuntimeError(format!("Type has no method '{}'", method))),
                }
            }

            _ => Err(EvalError::TypeError(format!(
                "no method '{}' on {:?}",
                method, obj
            ))),
        }
    }

    fn eval_pipe(&mut self, left: &Expr, right: &Expr) -> Result<Value, EvalError> {
        let arg = self.eval_expr(left)?;
        // right should be an identifier (function name) or a call
        match &right.kind {
            ExprKind::Identifier(name) => {
                // a |> f  =>  f(a)
                // Try builtins
                if let Some(result) =
                    opal_stdlib::call_builtin(name, &[arg.clone()], &mut self.writer)
                {
                    return match result {
                        Ok(opal_stdlib::BuiltinResult::Value(v)) => Ok(v),
                        Ok(opal_stdlib::BuiltinResult::Void) => Ok(Value::Null),
                        Err(e) => Err(EvalError::RuntimeError(e)),
                    };
                }
                if let Some(val) = self.env.get(name).cloned() {
                    match val {
                        Value::Function(id) => self.call_function(id, name, vec![arg]),
                        Value::MultiFunction(ids) => {
                            self.dispatch_multi(&ids, name, vec![arg])
                        }
                        Value::Closure(id) => self.call_closure(id, vec![arg]),
                        _ => Err(EvalError::TypeError(format!(
                            "pipe target '{}' is not a function",
                            name
                        ))),
                    }
                } else {
                    Err(EvalError::UndefinedVariable(name.clone()))
                }
            }
            ExprKind::Call { function, args } => {
                // a |> f(b)  =>  f(a, b)
                let func_name = match &function.kind {
                    ExprKind::Identifier(name) => name.clone(),
                    _ => {
                        return Err(EvalError::TypeError(
                            "pipe target must be a function call".into(),
                        ));
                    }
                };
                let mut arg_values = vec![arg];
                for a in args {
                    arg_values.push(self.eval_expr(&a.value)?);
                }
                if let Some(val) = self.env.get(&func_name).cloned() {
                    match val {
                        Value::Function(id) => self.call_function(id, &func_name, arg_values),
                        Value::MultiFunction(ids) => {
                            self.dispatch_multi(&ids, &func_name, arg_values)
                        }
                        _ => Err(EvalError::TypeError(format!(
                            "pipe target '{}' is not a function",
                            func_name
                        ))),
                    }
                } else {
                    Err(EvalError::UndefinedVariable(func_name))
                }
            }
            _ => Err(EvalError::TypeError(
                "pipe operator requires a function on the right side".into(),
            )),
        }
    }

    fn dispatch_multi(
        &mut self,
        ids: &[FunctionId],
        name: &str,
        arg_values: Vec<Value>,
    ) -> Result<Value, EvalError> {
        // Filter to arity-matching variants
        let arity_matches: Vec<FunctionId> = ids
            .iter()
            .filter(|id| self.functions[id.0].params.len() == arg_values.len())
            .copied()
            .collect();

        if arity_matches.is_empty() {
            let arities: Vec<String> = ids
                .iter()
                .map(|id| self.functions[id.0].params.len().to_string())
                .collect();
            return Err(EvalError::TypeError(format!(
                "{}() no variant accepts {} arguments (available: {})",
                name,
                arg_values.len(),
                arities.join(", ")
            )));
        }

        // 1. Try exact class type match (most specific)
        for id in &arity_matches {
            let stored = &self.functions[id.0];
            if stored.param_types.iter().any(|t| t.is_some())
                && self.args_match_types_exact(&arg_values, &stored.param_types)
            {
                match self.call_function(*id, name, arg_values.clone()) {
                    Err(EvalError::RequiresFailed(_)) => continue,
                    result => return result,
                }
            }
        }

        // 2. Try protocol/wider type match
        for id in &arity_matches {
            let stored = &self.functions[id.0];
            if stored.param_types.iter().any(|t| t.is_some())
                && self.args_match_types(&arg_values, &stored.param_types)
            {
                match self.call_function(*id, name, arg_values.clone()) {
                    Err(EvalError::RequiresFailed(_)) => continue,
                    result => return result,
                }
            }
        }

        // 3. Try untyped variant (no type annotations — catch-all)
        for id in &arity_matches {
            let stored = &self.functions[id.0];
            if stored.param_types.iter().all(|t| t.is_none()) {
                return self.call_function(*id, name, arg_values);
            }
        }

        // 4. Fall back to first arity match
        self.call_function(arity_matches[0], name, arg_values)
    }

    /// Check if argument values match exactly (class name, no protocol widening)
    fn args_match_types_exact(&self, args: &[Value], param_types: &[Option<String>]) -> bool {
        for (arg, expected_type) in args.iter().zip(param_types.iter()) {
            if let Some(type_name) = expected_type {
                if !self.value_matches_type_exact(arg, type_name) {
                    return false;
                }
            }
        }
        true
    }

    fn value_matches_type_exact(&self, value: &Value, type_name: &str) -> bool {
        match (value, type_name) {
            (Value::Integer(_), "Int" | "Int32" | "Int64" | "Integer") => true,
            (Value::Float(_), "Float" | "Float32" | "Float64") => true,
            (Value::String(_), "String") => true,
            (Value::Bool(_), "Bool") => true,
            (Value::List(_), "List") => true,
            (Value::Dict(_), "Dict") => true,
            (Value::Symbol(_), "Symbol") => true,
            (Value::Instance(id), _) => {
                let class_id = self.instances[id.0].class_id;
                self.classes[class_id.0].name == type_name
            }
            _ => false,
        }
    }

    /// Check if argument values match the expected parameter types (including protocols)
    fn args_match_types(&self, args: &[Value], param_types: &[Option<String>]) -> bool {
        for (arg, expected_type) in args.iter().zip(param_types.iter()) {
            if let Some(type_name) = expected_type {
                if !self.value_matches_type(arg, type_name) {
                    return false;
                }
            }
        }
        true
    }

    /// Check if a value matches a type name (class name, protocol, or builtin)
    fn value_matches_type(&self, value: &Value, type_name: &str) -> bool {
        match (value, type_name) {
            // Builtin types
            (Value::Integer(_), "Int" | "Int32" | "Int64" | "Integer") => true,
            (Value::Float(_), "Float" | "Float32" | "Float64") => true,
            (Value::String(_), "String") => true,
            (Value::Bool(_), "Bool") => true,
            (Value::List(_), "List") => true,
            (Value::Dict(_), "Dict") => true,
            (Value::Symbol(_), "Symbol") => true,
            // Class instance — check class name
            (Value::Instance(id), _) => {
                let class_id = self.instances[id.0].class_id;
                let class = &self.classes[class_id.0];
                if class.name == type_name {
                    return true;
                }
                // Check if class implements a protocol with this name
                self.class_implements_protocol(class_id, type_name)
            }
            _ => false,
        }
    }

    fn value_is_type(&self, value: &Value, type_name: &str) -> bool {
        match type_name {
            "Int" => matches!(value, Value::Integer(_)),
            "Float" => matches!(value, Value::Float(_)),
            "String" => matches!(value, Value::String(_)),
            "Bool" => matches!(value, Value::Bool(_)),
            "Null" => matches!(value, Value::Null),
            "Symbol" => matches!(value, Value::Symbol(_)),
            "List" => matches!(value, Value::List(_)),
            "Dict" => matches!(value, Value::Dict(_)),
            "Range" => matches!(value, Value::Range { .. }),
            "Fn" => matches!(value, Value::Function(_) | Value::MultiFunction(_) | Value::Closure(_)),
            name => {
                if let Value::Instance(id) = value {
                    let inst = &self.instances[id.0];
                    if self.classes[inst.class_id.0].name == name {
                        return true;
                    }
                    return self.class_implements_protocol(inst.class_id, name);
                }
                if let Value::EnumVariant { enum_id, .. } = value {
                    if self.enums[enum_id.0].name == name {
                        return true;
                    }
                }
                // Check type aliases
                if let Some(type_expr) = self.type_aliases.get(name).cloned() {
                    return self.value_matches_type_expr(value, &type_expr);
                }
                false
            }
        }
    }

    fn value_matches_type_expr(&self, value: &Value, type_expr: &TypeExpr) -> bool {
        match type_expr {
            TypeExpr::Named(name) => self.value_is_type(value, name),
            TypeExpr::SymbolSet(symbols) => {
                if let Value::Symbol(s) = value {
                    symbols.contains(s)
                } else {
                    false
                }
            }
            TypeExpr::Union(parts) => {
                parts.iter().any(|p| self.value_matches_type_expr(value, p))
            }
        }
    }

    /// Check if a class implements a protocol by name
    fn class_implements_protocol(&self, class_id: ClassId, protocol_name: &str) -> bool {
        // Check if the protocol exists and the class has all its required methods
        let proto = self.protocols.iter().find(|p| p.name == protocol_name);
        if let Some(proto) = proto {
            let class = &self.classes[class_id.0];
            proto
                .required_methods
                .iter()
                .all(|req| class.methods.iter().any(|m| m.name == *req))
        } else {
            false
        }
    }

    // Helper constructors for built-in Result/Option enum values
    fn make_ok(val: Value) -> Value {
        Value::EnumVariant { enum_id: EnumId(0), variant_index: 0, fields: vec![val] }
    }
    fn make_err(val: Value) -> Value {
        Value::EnumVariant { enum_id: EnumId(0), variant_index: 1, fields: vec![val] }
    }
    fn make_some(val: Value) -> Value {
        Value::EnumVariant { enum_id: EnumId(1), variant_index: 0, fields: vec![val] }
    }
    fn make_none() -> Value {
        Value::EnumVariant { enum_id: EnumId(1), variant_index: 1, fields: vec![] }
    }

    fn is_ok(val: &Value) -> Option<&Value> {
        match val {
            Value::EnumVariant { enum_id, variant_index: 0, fields } if enum_id.0 == 0 => {
                fields.first()
            }
            _ => None,
        }
    }
    fn is_err(val: &Value) -> Option<&Value> {
        match val {
            Value::EnumVariant { enum_id, variant_index: 1, fields } if enum_id.0 == 0 => {
                fields.first()
            }
            _ => None,
        }
    }

    fn value_type_info(&self, value: &Value) -> TypeInfo {
        match value {
            Value::Integer(_) => TypeInfo::Builtin(BuiltinType::Int),
            Value::Float(_) => TypeInfo::Builtin(BuiltinType::Float),
            Value::String(_) => TypeInfo::Builtin(BuiltinType::String),
            Value::Bool(_) => TypeInfo::Builtin(BuiltinType::Bool),
            Value::Null => TypeInfo::Builtin(BuiltinType::Null),
            Value::Symbol(_) => TypeInfo::Builtin(BuiltinType::Symbol),
            Value::List(_) => TypeInfo::Builtin(BuiltinType::List),
            Value::Dict(_) => TypeInfo::Builtin(BuiltinType::Dict),
            Value::Range { .. } => TypeInfo::Builtin(BuiltinType::Range),
            Value::Function(_) | Value::MultiFunction(_) | Value::Closure(_) | Value::NativeFunction(_) => {
                TypeInfo::Builtin(BuiltinType::Fn)
            }
            Value::Instance(id) => {
                let inst = &self.instances[id.0];
                TypeInfo::Class(inst.class_id)
            }
            Value::EnumVariant { enum_id, .. } => {
                TypeInfo::Enum(*enum_id)
            }
            _ => TypeInfo::Builtin(BuiltinType::Fn),
        }
    }

    fn type_info_name(&self, info: &TypeInfo) -> String {
        match info {
            TypeInfo::Builtin(b) => match b {
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
            }.to_string(),
            TypeInfo::Class(id) => self.classes[id.0].name.clone(),
            TypeInfo::Protocol(id) => self.protocols[id.0].name.clone(),
            TypeInfo::Enum(id) => self.enums[id.0].name.clone(),
            TypeInfo::EnumVariant(id, vi) => {
                format!("{}.{}", self.enums[id.0].name, self.enums[id.0].variants[*vi].name)
            }
        }
    }

    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Type(info) => self.type_info_name(info),
            Value::EnumVariant { enum_id, variant_index, fields } => {
                let e = &self.enums[enum_id.0];
                let v = &e.variants[*variant_index];
                // Special display for Result/Option (indices 0/1)
                if enum_id.0 <= 1 {
                    if fields.is_empty() {
                        return v.name.clone();
                    } else {
                        let args: Vec<String> = fields.iter().map(|f| self.format_value(f)).collect();
                        return format!("{}({})", v.name, args.join(", "));
                    }
                }
                if fields.is_empty() {
                    format!("{}.{}", e.name, v.name)
                } else {
                    let args: Vec<String> = fields.iter().map(|f| self.format_value(f)).collect();
                    format!("{}.{}({})", e.name, v.name, args.join(", "))
                }
            }
            Value::Instance(id) => {
                let inst = &self.instances[id.0];
                let class = &self.classes[inst.class_id.0];
                format!("<{} instance>", class.name)
            }
            Value::List(items) => {
                let parts: Vec<String> = items.iter().map(|v| self.format_value(v)).collect();
                format!("[{}]", parts.join(", "))
            }
            other => other.to_string(),
        }
    }

    fn call_function(
        &mut self,
        id: FunctionId,
        name: &str,
        arg_values: Vec<Value>,
    ) -> Result<Value, EvalError> {
        let stored = self.functions[id.0].clone();
        let mut arg_values = arg_values;
        // Fill missing args with defaults
        if arg_values.len() < stored.params.len() {
            for i in arg_values.len()..stored.params.len() {
                if let Some(default_expr) = &stored.param_defaults[i] {
                    let val = self.eval_expr(default_expr)?;
                    arg_values.push(val);
                } else {
                    return Err(EvalError::TypeError(format!(
                        "{}() expected {} arguments, got {}",
                        name,
                        stored.params.len(),
                        arg_values.len()
                    )));
                }
            }
        } else if arg_values.len() > stored.params.len() {
            return Err(EvalError::TypeError(format!(
                "{}() expected {} arguments, got {}",
                name,
                stored.params.len(),
                arg_values.len()
            )));
        }

        // If function has a captured env (defined in a module), use it
        let saved_env = if let Some(ref captured) = stored.captured_env {
            Some(std::mem::replace(&mut self.env, captured.clone()))
        } else {
            None
        };

        self.env.push_scope();
        for (param_name, arg_val) in stored.params.iter().zip(arg_values) {
            self.env.set(String::clone(param_name), arg_val);
        }

        let result = self.eval_block(&stored.body);
        self.env.pop_scope();

        // Restore original env if we swapped
        if let Some(original) = saved_env {
            self.env = original;
        }

        match result {
            Ok(val) => Ok(val),
            Err(EvalError::Return(val)) => Ok(val),
            Err(e) => Err(e),
        }
    }

    /// Run the HTTP serve loop. Binds a TCP listener and dispatches incoming
    /// requests to Opal closure handlers registered via `add_route`.
    fn serve_http(&mut self, args: &[Value]) -> Result<Value, EvalError> {
        if args.len() < 2 {
            return Err(EvalError::TypeError(
                "serve requires 2 arguments: app_id, port".into(),
            ));
        }
        let app_id = match &args[0] {
            Value::Integer(id) => *id,
            _ => {
                return Err(EvalError::TypeError(
                    "serve: expected integer app id".into(),
                ));
            }
        };
        let port = match &args[1] {
            Value::Integer(p) => *p,
            _ => return Err(EvalError::TypeError("serve: expected integer port".into())),
        };

        // Retrieve routes via the http plugin's get_routes function
        let routes_val =
            if let Some(native_fn) = self.plugin_registry.get_function("http", "get_routes") {
                native_fn(&[Value::Integer(app_id)], &mut self.writer)
                    .map_err(|e| EvalError::RuntimeError(e))?
            } else {
                return Err(EvalError::RuntimeError(
                    "http plugin missing get_routes".into(),
                ));
            };

        // Parse the routes list: [[method, path, handler_id], ...]
        let routes = match routes_val {
            Value::List(items) => items,
            _ => {
                return Err(EvalError::RuntimeError(
                    "get_routes returned non-list".into(),
                ));
            }
        };

        struct ParsedRoute {
            method: String,
            path: String,
            handler_id: usize,
        }

        let mut parsed_routes = Vec::new();
        for route in &routes {
            if let Value::List(parts) = route {
                if parts.len() == 3 {
                    let method = match &parts[0] {
                        Value::String(s) => s.clone(),
                        _ => continue,
                    };
                    let path = match &parts[1] {
                        Value::String(s) => s.clone(),
                        _ => continue,
                    };
                    let handler_id = match &parts[2] {
                        Value::Integer(id) => *id as usize,
                        _ => continue,
                    };
                    parsed_routes.push(ParsedRoute {
                        method,
                        path,
                        handler_id,
                    });
                }
            }
        }

        // Bind TCP listener
        let listener = std::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .map_err(|e| EvalError::RuntimeError(format!("Failed to bind port {}: {}", port, e)))?;

        writeln!(self.writer, "Opal HTTP server listening on port {}", port).ok();

        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Read request
            let mut buf = [0u8; 4096];
            let n = std::io::Read::read(&mut stream, &mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]);

            // Parse the HTTP request line: "METHOD /path HTTP/1.1"
            let first_line = request.lines().next().unwrap_or("");
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let (method, path) = (parts[0], parts[1]);

            // Match against registered routes
            let mut matched = false;
            for route in &parsed_routes {
                if route.method == method {
                    if let Some(params) = match_path(&route.path, path) {
                        matched = true;
                        let closure_id = ClosureId(route.handler_id);

                        // Pass route params as a simple argument if any exist
                        let handler_args = if params.is_empty() {
                            vec![]
                        } else {
                            vec![Value::String(
                                params.values().next().unwrap_or(&String::new()).clone(),
                            )]
                        };

                        let result = self.call_closure(closure_id, handler_args);
                        let body = match result {
                            Ok(val) => val.to_string(),
                            Err(e) => format!("Error: {}", e),
                        };

                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        std::io::Write::write_all(&mut stream, response.as_bytes()).ok();
                        break;
                    }
                }
            }

            if !matched {
                let body = "404 Not Found";
                let response = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                std::io::Write::write_all(&mut stream, response.as_bytes()).ok();
            }
        }

        Ok(Value::Null)
    }

    fn call_closure(&mut self, id: ClosureId, arg_values: Vec<Value>) -> Result<Value, EvalError> {
        let stored = self.closures[id.0].clone();

        // Save current env, switch to captured env
        let saved_env = std::mem::replace(&mut self.env, stored.captured_env.clone());

        self.env.push_scope();
        for (param_name, arg_val) in stored.params.iter().zip(arg_values) {
            self.env.set(String::clone(param_name), arg_val);
        }

        let result = self.eval_block(&stored.body);

        // Restore original env
        self.env = saved_env;

        match result {
            Ok(val) => Ok(val),
            Err(EvalError::Return(val)) => Ok(val),
            Err(e) => Err(e),
        }
    }

    fn eval_block(&mut self, stmts: &[Stmt]) -> Result<Value, EvalError> {
        let mut last = Value::Null;
        for stmt in stmts {
            match &stmt.kind {
                StmtKind::Expr(expr) => {
                    last = self.eval_expr(expr)?;
                }
                _ => {
                    self.eval_stmt(stmt)?;
                    last = Value::Null;
                }
            }
        }
        Ok(last)
    }
}

impl Default for Interpreter<std::io::Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

/// Match a URL path pattern against an actual path.
/// Patterns like `/users/:id` match `/users/42` and extract `{id: "42"}`.
/// Returns `Some(params)` on match, `None` on mismatch.
fn match_path(pattern: &str, actual: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let actual_parts: Vec<&str> = actual.split('/').collect();

    if pattern_parts.len() != actual_parts.len() {
        return None;
    }

    let mut params = HashMap::new();
    for (p, a) in pattern_parts.iter().zip(actual_parts.iter()) {
        if p.starts_with(':') {
            params.insert(p[1..].to_string(), a.to_string());
        } else if p != a {
            return None;
        }
    }
    Some(params)
}

fn eval_binary_op(op: BinOp, left: Value, right: Value) -> Result<Value, EvalError> {
    match (op, &left, &right) {
        // Integer arithmetic
        (BinOp::Add, Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
        (BinOp::Sub, Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
        (BinOp::Mul, Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
        (BinOp::Div, Value::Integer(a), Value::Integer(b)) => {
            if *b == 0 {
                Err(EvalError::RuntimeError("division by zero".into()))
            } else {
                Ok(Value::Integer(a / b))
            }
        }
        (BinOp::Mod, Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a % b)),
        (BinOp::Pow, Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a.pow(*b as u32))),
        (BinOp::Pow, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
        (BinOp::Pow, Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.powi(*b as i32))),
        (BinOp::Pow, Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64).powf(*b))),

        // Float arithmetic
        (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (BinOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),

        // Mixed numeric
        (BinOp::Add, Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
        (BinOp::Add, Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
        (BinOp::Sub, Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
        (BinOp::Sub, Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
        (BinOp::Mul, Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
        (BinOp::Mul, Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
        (BinOp::Div, Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
        (BinOp::Div, Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),

        // String concatenation
        (BinOp::Add, Value::String(a), Value::String(b)) => {
            Ok(Value::String(format!("{}{}", a, b)))
        }

        // Comparison
        (BinOp::Eq, _, _) => Ok(Value::Bool(values_equal(&left, &right))),
        (BinOp::NotEq, _, _) => Ok(Value::Bool(!values_equal(&left, &right))),
        (BinOp::Lt, Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a < b)),
        (BinOp::Gt, Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a > b)),
        (BinOp::LtEq, Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a <= b)),
        (BinOp::GtEq, Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a >= b)),

        // Float comparisons
        (BinOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
        (BinOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
        (BinOp::LtEq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
        (BinOp::GtEq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),

        // Mixed numeric comparisons
        (BinOp::Lt, Value::Integer(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
        (BinOp::Lt, Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(*a < (*b as f64))),
        (BinOp::Gt, Value::Integer(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
        (BinOp::Gt, Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(*a > (*b as f64))),
        (BinOp::LtEq, Value::Integer(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
        (BinOp::LtEq, Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(*a <= (*b as f64))),
        (BinOp::GtEq, Value::Integer(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
        (BinOp::GtEq, Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(*a >= (*b as f64))),

        // Logical
        (BinOp::And, _, _) => {
            if left.is_truthy() {
                Ok(right)
            } else {
                Ok(left)
            }
        }
        (BinOp::Or, _, _) => {
            if left.is_truthy() {
                Ok(left)
            } else {
                Ok(right)
            }
        }

        _ => Err(EvalError::TypeError(format!(
            "unsupported operation {:?} on {:?} and {:?}",
            op, left, right
        ))),
    }
}

fn eval_unary_op(op: UnOp, val: Value) -> Result<Value, EvalError> {
    match (op, &val) {
        (UnOp::Neg, Value::Integer(n)) => Ok(Value::Integer(-n)),
        (UnOp::Neg, Value::Float(n)) => Ok(Value::Float(-n)),
        (UnOp::Not, _) => Ok(Value::Bool(!val.is_truthy())),
        _ => Err(EvalError::TypeError(format!(
            "unsupported unary {:?} on {:?}",
            op, val
        ))),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Symbol(a), Value::Symbol(b)) => a == b,
        (Value::Null, Value::Null) => true,
        (Value::Type(a), Value::Type(b)) => a == b,
        (Value::EnumVariant { enum_id: a_id, variant_index: a_vi, fields: a_f },
         Value::EnumVariant { enum_id: b_id, variant_index: b_vi, fields: b_f }) => {
            a_id == b_id && a_vi == b_vi && a_f.len() == b_f.len() &&
            a_f.iter().zip(b_f.iter()).all(|(a, b)| values_equal(a, b))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(source: &str) -> Result<String, EvalError> {
        let program = opal_parser::parse(source).expect("parse error");
        let mut output = Vec::new();
        {
            let mut interp = Interpreter::with_writer(&mut output);
            interp.run(&program)?;
        }
        Ok(String::from_utf8(output).unwrap().trim_end().to_string())
    }

    // === Slice 1 tests ===
    #[test]
    fn hello_world() {
        let output = run(r#"print("Hello, world!")"#).unwrap();
        assert_eq!(output, "Hello, world!");
    }

    #[test]
    fn variable_and_print() {
        let output = run("name = \"Opal\"\nprint(name)").unwrap();
        assert_eq!(output, "Opal");
    }

    #[test]
    fn fstring() {
        let output = run("name = \"Opal\"\nprint(f\"Hello, {name}!\")").unwrap();
        assert_eq!(output, "Hello, Opal!");
    }

    #[test]
    fn arithmetic() {
        let output = run("print(2 + 3 * 4)").unwrap();
        assert_eq!(output, "14");
    }

    #[test]
    fn undefined_variable_error() {
        let result = run("print(undefined_var)");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("NameError"));
    }

    #[test]
    fn if_expression() {
        let output = run("print(if true then 1 else 2 end)").unwrap();
        assert_eq!(output, "1");
    }

    // === Slice 2 tests ===
    #[test]
    fn simple_function() {
        let output = run("def add(a, b)\n  return a + b\nend\nprint(add(2, 3))").unwrap();
        assert_eq!(output, "5");
    }

    #[test]
    fn factorial() {
        let output = run("def factorial(n: Int) -> Int\n  if n <= 1 then 1 else n * factorial(n - 1) end\nend\nprint(factorial(10))").unwrap();
        assert_eq!(output, "3628800");
    }

    #[test]
    fn fibonacci() {
        let output = run(
            "def fib(n)\n  if n <= 1 then n else fib(n - 1) + fib(n - 2) end\nend\nprint(fib(10))",
        )
        .unwrap();
        assert_eq!(output, "55");
    }

    #[test]
    fn function_implicit_return() {
        let output = run("def double(x)\n  x * 2\nend\nprint(double(21))").unwrap();
        assert_eq!(output, "42");
    }

    // === Slice 3 tests ===
    #[test]
    fn list_literal() {
        let output = run("print([1, 2, 3])").unwrap();
        assert_eq!(output, "[1, 2, 3]");
    }

    #[test]
    fn empty_list() {
        let output = run("print([])").unwrap();
        assert_eq!(output, "[]");
    }

    #[test]
    fn list_length() {
        let output = run("print([1, 2, 3].length())").unwrap();
        assert_eq!(output, "3");
    }

    #[test]
    fn list_map() {
        let output = run("print([1, 2, 3].map(|n| n * 2))").unwrap();
        assert_eq!(output, "[2, 4, 6]");
    }

    #[test]
    fn list_filter() {
        let output = run("print([1, 2, 3, 4].filter(|n| n % 2 == 0))").unwrap();
        assert_eq!(output, "[2, 4]");
    }

    #[test]
    fn list_reduce() {
        let output = run("print([1, 2, 3, 4].reduce(0) do |acc, n|\n  acc + n\nend)").unwrap();
        assert_eq!(output, "10");
    }

    #[test]
    fn data_cruncher() {
        let output = run(r#"
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
evens = numbers.filter(|n| n % 2 == 0)
squares = evens.map(|n| n ** 2)
total = squares.reduce(0) do |acc, n|
  acc + n
end
print(f"Sum of even squares: {total}")
"#)
        .unwrap();
        assert_eq!(output, "Sum of even squares: 220");
    }

    #[test]
    fn for_loop() {
        let output = run("for x in [1, 2, 3]\n  print(x)\nend").unwrap();
        assert_eq!(output, "1\n2\n3");
    }

    #[test]
    fn while_loop() {
        let output = run("x = 0\nwhile x < 3\n  x = x + 1\nend\nprint(x)").unwrap();
        assert_eq!(output, "3");
    }

    #[test]
    fn break_in_for() {
        assert_eq!(
            run("sum = 0\nfor i in 1..10\n  if i > 3\n    break\n  end\n  sum = sum + 1\nend\nprint(sum)")
                .unwrap(),
            "3"
        );
    }

    #[test]
    fn next_in_for() {
        assert_eq!(
            run("sum = 0\nfor i in 1..6\n  if i % 2 == 0\n    next\n  end\n  sum = sum + i\nend\nprint(sum)")
                .unwrap(),
            "9"
        );
    }

    #[test]
    fn break_in_while() {
        assert_eq!(
            run("i = 0\nwhile true\n  i = i + 1\n  if i == 5\n    break\n  end\nend\nprint(i)")
                .unwrap(),
            "5"
        );
    }

    #[test]
    fn pipe_operator() {
        let output = run("def double(x)\n  x * 2\nend\nprint(5 |> double)").unwrap();
        assert_eq!(output, "10");
    }

    #[test]
    fn list_push() {
        let output = run("print([1, 2].push(3))").unwrap();
        assert_eq!(output, "[1, 2, 3]");
    }

    #[test]
    fn closure_as_variable() {
        let output = run("f = |x| x + 1\nprint(f(10))").unwrap();
        assert_eq!(output, "11");
    }

    // === Slice 4: Class tests ===

    #[test]
    fn simple_class() {
        let output = run(
            "class Point\n  needs x: Int\n  needs y: Int\n\n  def sum()\n    .x + .y\n  end\nend\np = Point.new(x: 3, y: 4)\nprint(p.sum())",
        )
        .unwrap();
        assert_eq!(output, "7");
    }

    #[test]
    fn class_new_positional() {
        let output =
            run("class Pair\n  needs a: Int\n  needs b: Int\nend\np = Pair.new(10, 20)\nprint(p)")
                .unwrap();
        assert!(output.contains("instance"));
    }

    #[test]
    fn module_and_import() {
        let output = run(
            "module Shapes\n  class Circle\n    needs radius: Float\n\n    def area()\n      .radius * .radius\n    end\n  end\nend\nimport Shapes.{Circle}\nc = Circle.new(radius: 5.0)\nprint(c.area())",
        )
        .unwrap();
        assert_eq!(output, "25.0");
    }

    #[test]
    fn math_pi() {
        let output = run("print(Math.pi())").unwrap();
        assert!(output.starts_with("3.14159"));
    }

    #[test]
    fn shapes_target_program() {
        let output = run(r#"
module Shapes
  class Circle
    needs radius: Float

    def area() -> Float
      Math.pi() * .radius ** 2
    end
  end

  class Rectangle
    needs width: Float
    needs height: Float

    def area() -> Float
      .width * .height
    end
  end
end

import Shapes.{Circle, Rectangle}

shapes = [Circle.new(radius: 5.0), Rectangle.new(width: 3.0, height: 4.0)]
for shape in shapes
  print(f"Area: {shape.area()}")
end
"#)
        .unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("Area: 78.53"));
        assert_eq!(lines[1], "Area: 12.0");
    }

    // === Slice 5: Error handling tests ===

    #[test]
    fn result_ok() {
        let output = run("print(Ok(42))").unwrap();
        assert_eq!(output, "Ok(42)");
    }

    #[test]
    fn result_error() {
        let output = run("print(Error(\"oops\"))").unwrap();
        assert_eq!(output, "Error(oops)");
    }

    #[test]
    fn match_result() {
        let output = run(r#"
x = Ok(42)
match x
  case Ok(v)
    print(f"Got: {v}")
  case Error(e)
    print(f"Err: {e}")
end
"#)
        .unwrap();
        assert_eq!(output, "Got: 42");
    }

    #[test]
    fn match_error_case() {
        let output = run(r#"
x = Error("bad")
match x
  case Ok(v)
    print(v)
  case Error(e)
    print(f"Error: {e}")
end
"#)
        .unwrap();
        assert_eq!(output, "Error: bad");
    }

    #[test]
    fn requires_pass() {
        let output = run(r#"
def divide(a, b)
  requires b != 0, "division by zero"
  Ok(a / b)
end
match divide(10.0, 2.0)
  case Ok(result)
    print(f"Result: {result}")
  case Error(msg)
    print(f"Error: {msg}")
end
"#)
        .unwrap();
        assert_eq!(output, "Result: 5.0");
    }

    #[test]
    fn requires_fail() {
        let result = run(r#"
def divide(a, b)
  requires b != 0, "division by zero"
  Ok(a / b)
end
divide(10.0, 0)
"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("division by zero"));
    }

    #[test]
    fn try_catch() {
        let output = run(r#"
try
  raise "something went wrong"
catch as e
  print(f"Caught: {e}")
end
"#)
        .unwrap();
        assert_eq!(output, "Caught: something went wrong");
    }

    #[test]
    fn error_handling_target_program() {
        let output = run(r#"
def divide(a: Float, b: Float) -> Result[Float, String]
  requires b != 0.0, "division by zero"
  Ok(a / b)
end

match divide(10.0, 3.0)
  case Ok(result)
    print(f"Result: {result}")
  case Error(msg)
    print(f"Error: {msg}")
end
"#)
        .unwrap();
        assert!(output.starts_with("Result: 3.33"));
    }

    // === Slice 6: Actor tests ===

    #[test]
    fn symbol_literal() {
        let output = run("print(:hello)").unwrap();
        assert_eq!(output, ":hello");
    }

    #[test]
    fn match_symbol() {
        let output = run(r#"
match :ok
  case :ok
    print("matched ok")
  case :error
    print("matched error")
end
"#)
        .unwrap();
        assert_eq!(output, "matched ok");
    }

    #[test]
    fn actor_counter() {
        let output = run(r#"
actor Counter
  def init()
    .count = 0
  end

  receive
    case :increment
      .count = .count + 1
    case :get
      reply .count
  end
end

counter = Counter.new()
counter.send(:increment)
counter.send(:increment)
print(await counter.send(:get))
"#)
        .unwrap();
        assert_eq!(output, "2");
    }

    #[test]
    fn instance_variable_assignment() {
        let output = run(r#"
class Mutable
  needs value: Int

  def set(v)
    .value = v
  end

  def get()
    .value
  end
end
m = Mutable.new(value: 1)
m.set(42)
print(m.get())
"#)
        .unwrap();
        assert_eq!(output, "42");
    }

    // === Slice 7: Macro tests ===

    #[test]
    fn simple_macro() {
        let output = run(r#"
macro unless(condition, body)
  ast
    if not ($condition)
      $body
    end
  end
end

@unless false
  print("This prints!")
end
"#)
        .unwrap();
        assert_eq!(output, "This prints!");
    }

    #[test]
    fn macro_with_true_condition() {
        let output = run(r#"
macro unless(condition, body)
  ast
    if not ($condition)
      $body
    end
  end
end

@unless true
  print("Should not print")
end
print("done")
"#)
        .unwrap();
        assert_eq!(output, "done");
    }

    // === Slice 7.1: Macro regression tests ===

    #[test]
    fn macro_inline_args_no_trailing_block() {
        // Bug: @macro arg1, arg2 inside a function body consumed the
        // enclosing function's body as a trailing block
        let output = run(r#"
macro guard(condition, message)
  ast
    if not ($condition)
      raise $message
    end
  end
end

def withdraw(amount)
  @guard amount > 0, "must be positive"
  print(f"ok: {amount}")
end

withdraw(50)
"#)
        .unwrap();
        assert_eq!(output, "ok: 50");
    }

    #[test]
    fn macro_splice_in_raise() {
        // Bug: $var inside `raise $var` wasn't substituted
        let output = run(r#"
macro check(cond, msg)
  ast
    if not ($cond)
      raise $msg
    end
  end
end

try
  @check false, "boom"
catch as e
  print(e)
end
"#)
        .unwrap();
        assert_eq!(output, "boom");
    }

    #[test]
    fn macro_splice_in_fstring() {
        // Bug: $var inside f"...{$var}..." wasn't substituted
        let output = run(r#"
macro log(label, body)
  ast
    print(f"[{$label}]")
    $body
  end
end

@log "start"
  print("running")
end
"#)
        .unwrap();
        assert_eq!(output, "[start]\nrunning");
    }

    #[test]
    fn mixed_type_comparison() {
        // Bug: Float > Integer and similar comparisons were missing
        let output = run("print(5.0 > 3)").unwrap();
        assert_eq!(output, "true");
    }

    #[test]
    fn mixed_type_comparison_all_ops() {
        let output = run(r#"
print(1.5 < 2)
print(3.0 > 2)
print(2.0 <= 2)
print(3.0 >= 4)
"#)
        .unwrap();
        assert_eq!(output, "true\ntrue\ntrue\nfalse");
    }

    // === Slice 8 tests ===
    #[test]
    fn extern_native_function() {
        let source = r#"
extern "test"
  def add_native(a: Int, b: Int) -> Int
end
print(add_native(3, 4))
"#;
        let program = opal_parser::parse(source).expect("parse error");
        let mut output = Vec::new();
        {
            let mut interp = Interpreter::with_writer(&mut output);
            // Register a test plugin with an add_native function
            let mut fns = std::collections::HashMap::new();
            fns.insert(
                "add_native".to_string(),
                Box::new(
                    |args: &[Value], _writer: &mut dyn std::io::Write| -> Result<Value, String> {
                        match (&args[0], &args[1]) {
                            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                            _ => Err("expected integers".into()),
                        }
                    },
                ) as opal_stdlib::NativeFunction,
            );
            interp.register_plugin("test", fns);
            interp.run(&program).unwrap();
        }
        let out = String::from_utf8(output).unwrap().trim_end().to_string();
        assert_eq!(out, "7");
    }

    #[test]
    fn extern_missing_plugin_soft_failure() {
        // When a plugin is not registered, extern declarations are silently skipped
        let source = r#"
extern "nonexistent"
  def something() -> Null
end
print("ok")
"#;
        let output = run(source).unwrap();
        assert_eq!(output, "ok");
    }

    #[test]
    fn http_plugin_route_registration() {
        // Test that the HTTP plugin's create_app and add_route functions work.
        // We don't call serve here since it would block on TCP listen.
        let source = r#"
extern "http"
  def create_app() -> Int
  def add_route(app: Int, method: String, path: String, handler: Fn) -> Null
end

app = create_app()
add_route(app, "GET", "/hello", |req| "Hello!")
add_route(app, "GET", "/world", |req| "World!")
print("routes registered")
"#;
        let output = run(source).unwrap();
        assert_eq!(output, "routes registered");
    }

    #[test]
    fn http_serve_loop() {
        // Integration test: start the server on port 0 (OS-assigned),
        // send a request from another thread, and verify the response.
        let source = r#"
extern "http"
  def create_app() -> Int
  def add_route(app: Int, method: String, path: String, handler: Fn) -> Null
  def serve(app: Int, port: Int) -> Null
end

app = create_app()
add_route(app, "GET", "/", |req| "Hello from Opal!")
add_route(app, "GET", "/greet/:name", |name| name)
serve(app, __PORT__)
"#;
        // Bind a listener to get a free port, then close it so serve can use it.
        let tmp_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = tmp_listener.local_addr().unwrap().port();
        drop(tmp_listener);

        let source = source.replace("__PORT__", &port.to_string());

        let program = opal_parser::parse(&source).expect("parse error");

        // Run the server in a background thread
        let handle = std::thread::spawn(move || {
            let mut output = Vec::new();
            {
                let mut interp = Interpreter::with_writer(&mut output);
                // The server loop runs indefinitely; the test will just
                // make requests and then the thread will be dropped.
                let _ = interp.run(&program);
            }
            String::from_utf8(output).unwrap()
        });

        // Give the server a moment to bind
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Send a GET / request
        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        std::io::Write::write_all(&mut stream, b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();

        let mut response = String::new();
        std::io::Read::read_to_string(&mut stream, &mut response).unwrap();
        assert!(
            response.contains("Hello from Opal!"),
            "expected 'Hello from Opal!' in response, got: {}",
            response
        );

        // Send a GET /greet/world request (path param)
        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        std::io::Write::write_all(
            &mut stream,
            b"GET /greet/world HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .unwrap();

        let mut response = String::new();
        std::io::Read::read_to_string(&mut stream, &mut response).unwrap();
        assert!(
            response.contains("world"),
            "expected 'world' in response, got: {}",
            response
        );

        // Send a request for a non-existent route
        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        std::io::Write::write_all(
            &mut stream,
            b"GET /missing HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .unwrap();

        let mut response = String::new();
        std::io::Read::read_to_string(&mut stream, &mut response).unwrap();
        assert!(
            response.contains("404 Not Found"),
            "expected 404 response, got: {}",
            response
        );

        // Drop the handle (the server thread will exit when dropped)
        drop(handle);
    }

    // === Dicts ===

    #[test]
    fn dict_literal() {
        let output = run("d = {name: \"Opal\", version: 1}\nprint(d.get(\"name\"))").unwrap();
        assert_eq!(output, "Opal");
    }

    #[test]
    fn dict_keys() {
        let output = run("print({a: 1, b: 2}.keys())").unwrap();
        assert_eq!(output, "[a, b]");
    }

    #[test]
    fn empty_dict() {
        let output = run("print({:})").unwrap();
        assert_eq!(output, "{}");
    }

    // === Ranges ===

    #[test]
    fn range_for_loop() {
        let output = run("for x in 1..4\n  print(x)\nend").unwrap();
        assert_eq!(output, "1\n2\n3");
    }

    #[test]
    fn range_inclusive() {
        let output = run("for x in 1...3\n  print(x)\nend").unwrap();
        assert_eq!(output, "1\n2\n3");
    }

    #[test]
    fn range_to_list() {
        let output = run("print((1..4).to_list())").unwrap();
        assert_eq!(output, "[1, 2, 3]");
    }

    // === String methods ===

    #[test]
    fn string_split() {
        let output = run(r#"print("a,b,c".split(","))"#).unwrap();
        assert_eq!(output, "[a, b, c]");
    }

    #[test]
    fn string_trim() {
        let output = run(r#"print("  hello  ".trim())"#).unwrap();
        assert_eq!(output, "hello");
    }

    #[test]
    fn string_contains() {
        let output = run(r#"print("hello world".contains("world"))"#).unwrap();
        assert_eq!(output, "true");
    }

    #[test]
    fn string_replace() {
        let output = run(r#"print("hello world".replace("world", "opal"))"#).unwrap();
        assert_eq!(output, "hello opal");
    }

    #[test]
    fn string_to_upper() {
        let output = run(r#"print("hello".to_upper())"#).unwrap();
        assert_eq!(output, "HELLO");
    }

    #[test]
    fn string_chars() {
        let output = run(r#"print("abc".chars())"#).unwrap();
        assert_eq!(output, "[a, b, c]");
    }

    // === Closure environment capture ===

    #[test]
    fn closure_captures_env() {
        let output = run("x = 10\nf = |n| n + x\nprint(f(5))").unwrap();
        assert_eq!(output, "15");
    }

    #[test]
    fn closure_captures_at_creation() {
        let output = run("x = 10\nf = |n| n + x\nx = 99\nprint(f(5))").unwrap();
        assert_eq!(output, "15"); // captures x=10 at creation, not x=99
    }

    // === New import syntax tests ===

    #[test]
    fn import_selective_new_syntax() {
        let output = run(r#"
module Utils
  def double(x)
    x * 2
  end
  def triple(x)
    x * 3
  end
end
import Utils.{double}
print(double(5))
"#)
        .unwrap();
        assert_eq!(output, "10");
    }

    #[test]
    fn import_whole_module() {
        let output = run(r#"
module Math2
  def abs(x)
    if x < 0 then -x else x end
  end
end
import Math2
print(Math2.abs(-5))
"#)
        .unwrap();
        assert_eq!(output, "5");
    }

    #[test]
    fn import_alias() {
        let output = run(r#"
module LongModuleName
  def greet()
    "hello"
  end
end
import LongModuleName as L
print(L.greet())
"#)
        .unwrap();
        assert_eq!(output, "hello");
    }

    // === typeof tests ===
    #[test]
    fn typeof_builtin() {
        assert_eq!(run(r#"print(typeof(42).name)"#).unwrap(), "Int");
        assert_eq!(run(r#"print(typeof("hi").name)"#).unwrap(), "String");
        assert_eq!(run(r#"print(typeof(true).name)"#).unwrap(), "Bool");
        assert_eq!(run(r#"print(typeof(null).name)"#).unwrap(), "Null");
        assert_eq!(run(r#"print(typeof(:ok).name)"#).unwrap(), "Symbol");
    }

    #[test]
    fn typeof_equality() {
        assert_eq!(run(r#"print(typeof(1) == typeof(2))"#).unwrap(), "true");
        assert_eq!(run(r#"print(typeof(1) == typeof("hi"))"#).unwrap(), "false");
    }

    #[test]
    fn typeof_class() {
        assert_eq!(
            run("class Foo\n  needs x: Int\nend\nf = Foo.new(x: 1)\nprint(typeof(f).name)").unwrap(),
            "Foo"
        );
    }

    // === is operator tests ===
    #[test]
    fn is_operator_builtins() {
        assert_eq!(run("print(42 is Int)").unwrap(), "true");
        assert_eq!(run("print(42 is String)").unwrap(), "false");
        assert_eq!(run(r#"print("hi" is String)"#).unwrap(), "true");
        assert_eq!(run("print(true is Bool)").unwrap(), "true");
        assert_eq!(run("print(null is Null)").unwrap(), "true");
    }

    #[test]
    fn is_not_operator() {
        assert_eq!(run("print(42 is not String)").unwrap(), "true");
        assert_eq!(run("print(42 is not Int)").unwrap(), "false");
    }

    #[test]
    fn is_operator_class() {
        assert_eq!(
            run("class Foo\n  needs x: Int\nend\nf = Foo.new(x: 1)\nprint(f is Foo)").unwrap(),
            "true"
        );
    }

    // === type alias tests ===
    #[test]
    fn type_alias_basic() {
        assert_eq!(run("type ID = Int\nprint(42 is ID)").unwrap(), "true");
    }

    #[test]
    fn type_alias_symbol_set() {
        assert_eq!(run("type Status = :ok | :error\nprint(:ok is Status)").unwrap(), "true");
        assert_eq!(run("type Status = :ok | :error\nprint(:unknown is Status)").unwrap(), "false");
    }

    // === enum tests ===
    #[test]
    fn enum_singleton() {
        assert_eq!(
            run("enum Dir\n  North\n  South\nend\nprint(Dir.North)").unwrap(),
            "Dir.North"
        );
    }

    #[test]
    fn enum_data_variant() {
        assert_eq!(
            run("enum Shape\n  Circle(r: Float)\nend\nprint(Shape.Circle(5.0))").unwrap(),
            "Shape.Circle(5.0)"
        );
    }

    #[test]
    fn enum_pattern_match() {
        assert_eq!(
            run("enum Shape\n  Circle(r: Float)\n  Rect(w: Float, h: Float)\nend\ns = Shape.Rect(3.0, 4.0)\nresult = match s\n  case Shape.Circle(r)\n    r\n  case Shape.Rect(w, h)\n    w * h\nend\nprint(result)").unwrap(),
            "12.0"
        );
    }

    #[test]
    fn enum_is_check() {
        assert_eq!(
            run("enum Dir\n  N\n  S\nend\nd = Dir.N\nprint(d is Dir)").unwrap(),
            "true"
        );
    }

    // === AST eval tests ===
    #[test]
    fn ast_eval_basic() {
        assert_eq!(run("code = ast\n  2 + 3\nend\nprint(eval(code))").unwrap(), "5");
    }

    #[test]
    fn ast_eval_child_scope() {
        assert_eq!(run("x = 1\ncode = ast\n  x = 99\nend\neval(code)\nprint(x)").unwrap(), "1");
    }

    #[test]
    fn ast_eval_reads_parent() {
        assert_eq!(run("x = 10\ncode = ast\n  x * 2\nend\nprint(eval(code))").unwrap(), "20");
    }

    #[test]
    fn enum_typeof() {
        assert_eq!(
            run("enum Color\n  Red\n  Blue\nend\nc = Color.Red\nprint(typeof(c).name)").unwrap(),
            "Color"
        );
    }

    #[test]
    fn type_alias_union() {
        assert_eq!(run("type NumOrStr = Int | String\nprint(42 is NumOrStr)").unwrap(), "true");
        assert_eq!(run(r#"type NumOrStr = Int | String
print("hi" is NumOrStr)"#).unwrap(), "true");
        assert_eq!(run("type NumOrStr = Int | String\nprint(true is NumOrStr)").unwrap(), "false");
    }

    #[test]
    fn typeof_fields() {
        assert_eq!(
            run("class Foo\n  needs x: Int\n  needs y: String\nend\nf = Foo.new(x: 1, y: \"a\")\nprint(typeof(f).fields)").unwrap(),
            "[[:x, Int], [:y, String]]"
        );
    }

    #[test]
    fn compound_assign_plus() {
        assert_eq!(run("x = 10\nx += 5\nprint(x)").unwrap(), "15");
    }

    #[test]
    fn compound_assign_all_ops() {
        assert_eq!(run("x = 10\nx -= 3\nprint(x)").unwrap(), "7");
        assert_eq!(run("x = 5\nx *= 4\nprint(x)").unwrap(), "20");
        assert_eq!(run("x = 20\nx /= 4\nprint(x)").unwrap(), "5");
    }

    #[test]
    fn default_param_used() {
        assert_eq!(
            run("def greet(name, greeting = 'Hello')\n  f\"{greeting}, {name}!\"\nend\nprint(greet('World'))").unwrap(),
            "Hello, World!"
        );
    }

    #[test]
    fn default_param_overridden() {
        assert_eq!(
            run("def greet(name, greeting = 'Hello')\n  f\"{greeting}, {name}!\"\nend\nprint(greet('World', 'Hi'))").unwrap(),
            "Hi, World!"
        );
    }

    #[test]
    fn list_index() {
        assert_eq!(run("l = [10, 20, 30]\nprint(l[1])").unwrap(), "20");
    }

    #[test]
    fn list_negative_index() {
        assert_eq!(run("l = [10, 20, 30]\nprint(l[-1])").unwrap(), "30");
    }

    #[test]
    fn string_index() {
        assert_eq!(run(r#"print("hello"[0])"#).unwrap(), "h");
    }

    #[test]
    fn list_index_assign() {
        assert_eq!(
            run("l = [1, 2, 3]\nl[0] = 99\nprint(l[0])").unwrap(),
            "99"
        );
    }

    #[test]
    fn in_operator() {
        assert_eq!(run("print(2 in [1, 2, 3])").unwrap(), "true");
        assert_eq!(run("print(4 in [1, 2, 3])").unwrap(), "false");
    }

    #[test]
    fn not_in_operator() {
        assert_eq!(run("print(5 not in [1, 2, 3])").unwrap(), "true");
        assert_eq!(run("print(2 not in [1, 2, 3])").unwrap(), "false");
    }

    #[test]
    fn null_safe_access() {
        assert_eq!(run("val = null\nprint(val?.name)").unwrap(), "null");
    }

    #[test]
    fn null_coalesce() {
        assert_eq!(run(r#"print(null ?? "default")"#).unwrap(), "default");
        assert_eq!(run(r#"print("value" ?? "default")"#).unwrap(), "value");
    }
}

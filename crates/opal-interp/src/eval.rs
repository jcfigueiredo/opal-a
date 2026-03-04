use std::io::Write;

use std::collections::HashMap;

use opal_parser::ast::*;
use opal_runtime::{
    ActorId, AstId, ClassId, ClosureId, Environment, FunctionId, InstanceId, ModuleId, Value,
};
use thiserror::Error;

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
    #[error("reply")]
    Reply(Value),
}

/// A stored user-defined function
#[derive(Clone)]
struct StoredFunction {
    #[allow(dead_code)]
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

/// A stored closure
#[derive(Clone)]
struct StoredClosure {
    params: Vec<String>,
    body: Vec<Stmt>,
}

/// A stored class definition
#[derive(Clone)]
struct StoredClass {
    #[allow(dead_code)]
    name: String,
    needs: Vec<(String, Option<String>)>,
    methods: Vec<StoredFunction>,
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
    macros: HashMap<String, StoredMacro>,
    ast_nodes: Vec<Vec<Stmt>>,
}

impl Interpreter<std::io::Stdout> {
    pub fn new() -> Self {
        Self {
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
            macros: HashMap::new(),
            ast_nodes: Vec::new(),
        }
    }
}

impl<W: Write> Interpreter<W> {
    pub fn with_writer(writer: W) -> Self {
        Self {
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
            macros: HashMap::new(),
            ast_nodes: Vec::new(),
        }
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
            StmtKind::Let { name, value } => {
                let val = self.eval_expr(value)?;
                self.env.set(name.clone(), val);
            }
            StmtKind::FuncDef {
                name, params, body, ..
            } => {
                let id = FunctionId(self.functions.len());
                self.functions.push(StoredFunction {
                    name: name.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    body: body.clone(),
                });
                self.env.set(name.clone(), Value::Function(id));
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
                let items = match iter_val {
                    Value::List(items) => items,
                    _ => return Err(EvalError::TypeError("for loop requires a list".into())),
                };
                for item in items {
                    self.env.push_scope();
                    self.env.set(var.clone(), item);
                    let result = self.eval_block(body);
                    self.env.pop_scope();
                    result?;
                }
            }
            StmtKind::While { condition, body } => loop {
                let cond = self.eval_expr(condition)?;
                if !cond.is_truthy() {
                    break;
                }
                self.eval_block(body)?;
            },
            StmtKind::ClassDef {
                name,
                needs,
                methods,
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
                            body: body.clone(),
                        });
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
                    return Err(EvalError::Raise(msg));
                }
            }
            StmtKind::TryCatch {
                body,
                catches,
                ensure,
            } => {
                let result = self.eval_block(body);

                match result {
                    Err(EvalError::Raise(val)) => {
                        // Use first matching catch (for now, all catches match)
                        if let Some(catch) = catches.first() {
                            self.env.push_scope();
                            if let Some(var) = &catch.var_name {
                                self.env.set(var.clone(), val.clone());
                            }
                            self.eval_block(&catch.body)?;
                            self.env.pop_scope();
                        } else {
                            // Re-raise if no catch matched
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
                    Ok(_) => {}
                }

                if let Some(ensure_body) = ensure {
                    self.eval_block(ensure_body)?;
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
                // Store a class-like value that supports .new()
                self.env
                    .set(name.clone(), Value::Class(ClassId(1000 + def_idx)));
                // Use a convention: ClassId >= 1000 means actor def
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
                self.macros.insert(
                    name.clone(),
                    StoredMacro {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            }
            StmtKind::MacroInvoke { name, args, block } => {
                let mac = self
                    .macros
                    .get(name)
                    .cloned()
                    .ok_or_else(|| EvalError::UndefinedVariable(format!("@{}", name)))?;

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

            ExprKind::Identifier(name) => self
                .env
                .get(name)
                .cloned()
                .ok_or_else(|| EvalError::UndefinedVariable(name.clone())),

            ExprKind::FString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        FStringPart::Literal(s) => result.push_str(s),
                        FStringPart::Expr(e) => {
                            let val = self.eval_expr(e)?;
                            result.push_str(&val.to_string());
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
                });
                Ok(Value::Closure(id))
            }

            ExprKind::Call { function, args } => self.eval_call(function, args),

            ExprKind::BinaryOp { left, op, right } => {
                // Special handling for pipe operator
                if *op == BinOp::Pipe {
                    return self.eval_pipe(left, right);
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

            ExprKind::MemberAccess { .. } => Err(EvalError::TypeError(
                "bare member access not supported — use method call syntax".into(),
            )),

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
            StmtKind::Let { name, value } => StmtKind::Let {
                name: name.clone(),
                value: self.substitute_expr(value),
            },
            StmtKind::InstanceAssign { field, value } => StmtKind::InstanceAssign {
                field: field.clone(),
                value: self.substitute_expr(value),
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
            _ => expr.clone(),
        }
    }

    /// Try to match a value against a pattern. Returns bindings on success.
    fn match_pattern(&self, pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
        match pattern {
            Pattern::Wildcard => Some(vec![]),
            Pattern::Identifier(name) => Some(vec![(name.clone(), value.clone())]),
            Pattern::Constructor(name, sub_patterns) => match (name.as_str(), value) {
                ("Ok", Value::Ok(inner)) => {
                    if sub_patterns.len() == 1 {
                        self.match_pattern(&sub_patterns[0], inner)
                    } else {
                        None
                    }
                }
                ("Error", Value::Error(inner)) => {
                    if sub_patterns.len() == 1 {
                        self.match_pattern(&sub_patterns[0], inner)
                    } else {
                        None
                    }
                }
                ("Some", Value::Some(inner)) => {
                    if sub_patterns.len() == 1 {
                        self.match_pattern(&sub_patterns[0], inner)
                    } else {
                        None
                    }
                }
                ("None", Value::Null) if sub_patterns.is_empty() => Some(vec![]),
                _ => None,
            },
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
        // Method call: expr.method(args)
        if let ExprKind::MemberAccess { object, field } = &function.kind {
            let obj = self.eval_expr(object)?;
            let mut eval_args = Vec::new();
            for arg in args {
                eval_args.push((arg.name.clone(), self.eval_expr(&arg.value)?));
            }
            return self.call_method(obj, field, eval_args);
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

        // Builtin constructors: Ok(), Error(), Some()
        match func_name.as_str() {
            "Ok" if arg_values.len() == 1 => {
                return Ok(Value::Ok(Box::new(arg_values.into_iter().next().unwrap())));
            }
            "Error" if arg_values.len() == 1 => {
                return Ok(Value::Error(Box::new(
                    arg_values.into_iter().next().unwrap(),
                )));
            }
            "Some" if arg_values.len() == 1 => {
                return Ok(Value::Some(Box::new(
                    arg_values.into_iter().next().unwrap(),
                )));
            }
            _ => {}
        }

        // Try stdlib builtins
        if let Some(result) = opal_stdlib::call_builtin(&func_name, &arg_values, &mut self.writer) {
            return match result {
                Ok(opal_stdlib::BuiltinResult::Value(v)) => Ok(v),
                Ok(opal_stdlib::BuiltinResult::Void) => Ok(Value::Null),
                Err(e) => Err(EvalError::RuntimeError(e)),
            };
        }

        // Try user-defined functions or closures
        if let Some(val) = self.env.get(&func_name).cloned() {
            match val {
                Value::Function(id) => return self.call_function(id, &func_name, arg_values),
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

            // Actor .new() — ClassId >= 1000 convention
            (Value::Class(class_id), "new") if class_id.0 >= 1000 => {
                let def_idx = class_id.0 - 1000;
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

                // Find method in class
                let method_fn = class.methods.iter().find(|m| m.name == method);
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

    fn call_function(
        &mut self,
        id: FunctionId,
        name: &str,
        arg_values: Vec<Value>,
    ) -> Result<Value, EvalError> {
        let stored = self.functions[id.0].clone();
        if arg_values.len() != stored.params.len() {
            return Err(EvalError::TypeError(format!(
                "{}() expected {} arguments, got {}",
                name,
                stored.params.len(),
                arg_values.len()
            )));
        }

        self.env.push_scope();
        for (param_name, arg_val) in stored.params.iter().zip(arg_values) {
            self.env.set(String::clone(param_name), arg_val);
        }

        let result = self.eval_block(&stored.body);
        self.env.pop_scope();

        match result {
            Ok(val) => Ok(val),
            Err(EvalError::Return(val)) => Ok(val),
            Err(e) => Err(e),
        }
    }

    fn call_closure(&mut self, id: ClosureId, arg_values: Vec<Value>) -> Result<Value, EvalError> {
        let stored = self.closures[id.0].clone();

        self.env.push_scope();
        for (param_name, arg_val) in stored.params.iter().zip(arg_values) {
            self.env.set(String::clone(param_name), arg_val);
        }

        let result = self.eval_block(&stored.body);
        self.env.pop_scope();

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
        (Value::Null, Value::Null) => true,
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
            "module Shapes\n  class Circle\n    needs radius: Float\n\n    def area()\n      .radius * .radius\n    end\n  end\nend\nfrom Shapes import Circle\nc = Circle.new(radius: 5.0)\nprint(c.area())",
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

from Shapes import Circle, Rectangle

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
}

use std::io::Write;

use opal_parser::ast::*;
use opal_runtime::{ClosureId, Environment, FunctionId, Value};
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

pub struct Interpreter<W: Write> {
    env: Environment,
    writer: W,
    functions: Vec<StoredFunction>,
    closures: Vec<StoredClosure>,
}

impl Interpreter<std::io::Stdout> {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            writer: std::io::stdout(),
            functions: Vec::new(),
            closures: Vec::new(),
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
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), EvalError> {
        for stmt in &program.statements {
            self.eval_stmt(stmt)?;
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

            ExprKind::MemberAccess { .. } => Err(EvalError::TypeError(
                "bare member access not supported — use method call syntax".into(),
            )),
        }
    }

    fn eval_call(&mut self, function: &Expr, args: &[Arg]) -> Result<Value, EvalError> {
        // Method call: expr.method(args)
        if let ExprKind::MemberAccess { object, field } = &function.kind {
            let obj = self.eval_expr(object)?;
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(self.eval_expr(&arg.value)?);
            }
            return self.call_method(obj, field, arg_values);
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
        args: Vec<Value>,
    ) -> Result<Value, EvalError> {
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
}

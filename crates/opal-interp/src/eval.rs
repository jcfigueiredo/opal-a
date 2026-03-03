use std::io::Write;

use opal_parser::ast::*;
use opal_runtime::{Environment, FunctionId, Value};
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
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

pub struct Interpreter<W: Write> {
    env: Environment,
    writer: W,
    functions: Vec<StoredFunction>,
}

impl Interpreter<std::io::Stdout> {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            writer: std::io::stdout(),
            functions: Vec::new(),
        }
    }
}

impl<W: Write> Interpreter<W> {
    pub fn with_writer(writer: W) -> Self {
        Self {
            env: Environment::new(),
            writer,
            functions: Vec::new(),
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
                self.env.set(name.clone(), val);
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

            ExprKind::Call { function, args } => {
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
                if let Some(result) =
                    opal_stdlib::call_builtin(&func_name, &arg_values, &mut self.writer)
                {
                    return match result {
                        Ok(opal_stdlib::BuiltinResult::Value(v)) => Ok(v),
                        Ok(opal_stdlib::BuiltinResult::Void) => Ok(Value::Null),
                        Err(e) => Err(EvalError::RuntimeError(e)),
                    };
                }

                // Try user-defined functions
                if let Some(value) = self.env.get(&func_name).cloned() {
                    if let Value::Function(id) = value {
                        return self.call_function(id, &func_name, arg_values);
                    }
                }

                Err(EvalError::UndefinedVariable(func_name))
            }

            ExprKind::BinaryOp { left, op, right } => {
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
                "member access not yet supported".into(),
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

    #[test]
    fn hello_world() {
        let output = run(r#"print("Hello, world!")"#).unwrap();
        assert_eq!(output, "Hello, world!");
    }

    #[test]
    fn variable_and_print() {
        let output = run(r#"
name = "Opal"
print(name)
"#)
        .unwrap();
        assert_eq!(output, "Opal");
    }

    #[test]
    fn fstring() {
        let output = run(r#"
name = "Opal"
print(f"Hello, {name}!")
"#)
        .unwrap();
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
        let err = result.unwrap_err().to_string();
        assert!(err.contains("NameError"));
    }

    #[test]
    fn string_concatenation() {
        let output = run(r#"print("hello" + " " + "world")"#).unwrap();
        assert_eq!(output, "hello world");
    }

    #[test]
    fn if_expression() {
        let output = run("print(if true then 1 else 2 end)").unwrap();
        assert_eq!(output, "1");
    }

    #[test]
    fn comparison() {
        let output = run("print(3 > 2)").unwrap();
        assert_eq!(output, "true");
    }

    #[test]
    fn let_binding() {
        let output = run("let x = 42\nprint(x)").unwrap();
        assert_eq!(output, "42");
    }

    // === Slice 2: Function tests ===

    #[test]
    fn simple_function() {
        let output = run("def add(a, b)\n  return a + b\nend\nprint(add(2, 3))").unwrap();
        assert_eq!(output, "5");
    }

    #[test]
    fn factorial() {
        let output = run(
            "def factorial(n: Int) -> Int\n  if n <= 1 then 1 else n * factorial(n - 1) end\nend\nprint(factorial(10))",
        )
        .unwrap();
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

    #[test]
    fn function_no_args() {
        let output =
            run("def greeting()\n  return \"hello\"\nend\nprint(greeting())").unwrap();
        assert_eq!(output, "hello");
    }

    #[test]
    fn function_wrong_arg_count() {
        let result = run("def foo(a, b)\n  a + b\nend\nfoo(1)");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("expected 2 arguments"));
    }

    #[test]
    fn nested_function_calls() {
        let output = run(
            "def square(x)\n  x * x\nend\ndef sum_of_squares(a, b)\n  square(a) + square(b)\nend\nprint(sum_of_squares(3, 4))",
        )
        .unwrap();
        assert_eq!(output, "25");
    }
}

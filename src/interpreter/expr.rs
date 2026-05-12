use std::collections::HashMap;
use crate::ast::*;
use crate::errors::ElangError;

#[derive(Debug, Clone)]
pub enum Val {
    Nothing,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Vec<Val>),
    Map(HashMap<String, Val>),
    Fn {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        is_async: bool,
        is_pure: bool,
    },
    Native {
        name: String,
        func: fn(&[Val]) -> Result<Val, String>,
    },
}

#[derive(Debug, Clone)]
pub struct Interpreter {
    pub call_stack: Vec<String>,
    pub last_line: usize,
    pub values: HashMap<String, Val>,
    pub parent: Option<Box<Interpreter>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            call_stack: Vec::new(),
            last_line: 0,
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn child(env: &Interpreter) -> Self {
        Interpreter {
            call_stack: env.call_stack.clone(),
            last_line: 0,
            values: HashMap::new(),
            parent: Some(Box::new(env.clone())),
        }
    }

    pub fn declare(&mut self, name: String, value: Val) {
        self.values.insert(name, value);
    }

    pub fn lookup(&self, name: &str) -> Option<&Val> {
        self.values
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Val, ElangError> {
        self.last_line = expr.line();
        let line = self.last_line;
        match expr {
            Expr::Int { value, .. } => Ok(Val::Int(*value)),
            Expr::Float { value, .. } => Ok(Val::Float(*value)),
            Expr::Str { value, .. } => Ok(Val::Str(value.clone())),
            Expr::Bool { value, .. } => Ok(Val::Bool(*value)),
            Expr::Nothing { .. } => Ok(Val::Nothing),
            Expr::Ident { name, .. } => {
                if let Some(val) = self.lookup(name) {
                    Ok(val.clone())
                } else {
                    Err(self.make_error(&format!("undefined variable '{}'", name)))
                }
            }
            Expr::BinOp { left, op, right, .. } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                apply_binop(op, &l, &r, line, self)
            }
            Expr::Call { callee, args, .. } => {
                let callee_val = self.eval_expr(callee)?;
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.eval_expr(arg)?);
                }
                match callee_val {
                    Val::Fn {
                        name,
                        params,
                        body,
                        ..
                    } => {
                        let call_line = line;
                        self.call_stack
                            .push(format!("in {}() at line {}", name, call_line));
                        let mut local = Interpreter::child(self);
                        for (param, arg) in params.iter().zip(arg_vals.iter()) {
                            local.declare(param.clone(), arg.clone());
                        }
                        let result = local.exec_block(&body);
                        self.call_stack.pop();
                        result
                    }
                    Val::Native { func, .. } => func(&arg_vals).map_err(|msg| {
                        ElangError::RuntimeError {
                            message: msg,
                            line,
                            stack: self.call_stack.clone(),
                        }
                    }),
                    _ => Err(self.make_error("cannot call non-function value")),
                }
            }
            Expr::List { items, .. } => {
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.eval_expr(item)?);
                }
                Ok(Val::List(vals))
            }
            _ => Err(self.make_error("expression not supported")),
        }
    }

    pub fn exec_block(&mut self, stmts: &[Statement]) -> Result<Val, ElangError> {
        for stmt in stmts {
            self.last_line = stmt.line();
            match stmt {
                Statement::ExprStmt { expr, .. } => {
                    self.eval_expr(expr)?;
                }
                Statement::Print { value, .. } => {
                    let val = self.eval_expr(value)?;
                    println!("{}", val);
                }
                Statement::LetDecl { name, value, .. } => {
                    let val = self.eval_expr(value)?;
                    self.declare(name.clone(), val);
                }
                Statement::FnDef {
                    name,
                    params,
                    body,
                    is_async,
                    is_pure,
                    ..
                } => {
                    self.declare(
                        name.clone(),
                        Val::Fn {
                            name: name.clone(),
                            params: params.clone(),
                            body: body.clone(),
                            is_async: *is_async,
                            is_pure: *is_pure,
                        },
                    );
                }
                Statement::Return { value, .. } => {
                    let val = self.eval_expr(value)?;
                    return Ok(val);
                }
                _ => {}
            }
        }
        Ok(Val::Nothing)
    }

    pub fn run(&mut self, program: &Program) -> Result<Val, ElangError> {
        self.call_stack.push("in <main> at line 0".to_string());
        let result = self.exec_block(program);
        self.call_stack.pop();
        result
    }

    pub fn make_error(&self, msg: &str) -> ElangError {
        ElangError::RuntimeError {
            message: msg.to_string(),
            line: self.last_line,
            stack: self.call_stack.clone(),
        }
    }
}

fn apply_binop(
    op: &BinOpKind,
    l: &Val,
    r: &Val,
    line: usize,
    ctx: &Interpreter,
) -> Result<Val, ElangError> {
    match (l, r) {
        (Val::Int(a), Val::Int(b)) => match op {
            BinOpKind::Add => Ok(Val::Int(a + b)),
            BinOpKind::Sub => Ok(Val::Int(a - b)),
            BinOpKind::Mul => Ok(Val::Int(a * b)),
            BinOpKind::Div => {
                if *b == 0 {
                    Err(ElangError::RuntimeError {
                        message: "division by zero".to_string(),
                        line,
                        stack: ctx.call_stack.clone(),
                    })
                } else {
                    Ok(Val::Int(a / b))
                }
            }
            BinOpKind::Mod => Ok(Val::Int(a % b)),
            BinOpKind::Eq => Ok(Val::Bool(a == b)),
            BinOpKind::NotEq => Ok(Val::Bool(a != b)),
            BinOpKind::Lt => Ok(Val::Bool(a < b)),
            BinOpKind::Gt => Ok(Val::Bool(a > b)),
            BinOpKind::LtEq => Ok(Val::Bool(a <= b)),
            BinOpKind::GtEq => Ok(Val::Bool(a >= b)),
            BinOpKind::And => Ok(Val::Bool(*a != 0 && *b != 0)),
            BinOpKind::Or => Ok(Val::Bool(*a != 0 || *b != 0)),
        },
        _ => Err(ElangError::RuntimeError {
            message: "type mismatch in binary operation".to_string(),
            line,
            stack: ctx.call_stack.clone(),
        }),
    }
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Nothing => write!(f, "nothing"),
            Val::Int(n) => write!(f, "{}", n),
            Val::Float(n) => write!(f, "{}", n),
            Val::Bool(b) => write!(f, "{}", b),
            Val::Str(s) => write!(f, "{}", s),
            Val::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Val::Map(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            _ => write!(f, "<function>"),
        }
    }
}

impl Expr {
    pub fn line(&self) -> usize {
        match self {
            Expr::Int { line, .. } => *line,
            Expr::Float { line, .. } => *line,
            Expr::Str { line, .. } => *line,
            Expr::Bool { line, .. } => *line,
            Expr::Nothing { line } => *line,
            Expr::Ident { line, .. } => *line,
            Expr::StrInterp { line, .. } => *line,
            Expr::BinOp { line, .. } => *line,
            Expr::UnaryOp { line, .. } => *line,
            Expr::Call { line, .. } => *line,
            Expr::Index { line, .. } => *line,
            Expr::Field { line, .. } => *line,
            Expr::List { line, .. } => *line,
            Expr::Map { line, .. } => *line,
            Expr::Lambda { line, .. } => *line,
            Expr::Pipe { line, .. } => *line,
            Expr::Await { line, .. } => *line,
        }
    }
}

impl Statement {
    pub fn line(&self) -> usize {
        match self {
            Statement::LetDecl { line, .. } => *line,
            Statement::ConstDecl { line, .. } => *line,
            Statement::VarDecl { line, .. } => *line,
            Statement::Assign { line, .. } => *line,
            Statement::FnDef { line, .. } => *line,
            Statement::ClassDef { line, .. } => *line,
            Statement::Return { line, .. } => *line,
            Statement::If { line, .. } => *line,
            Statement::Loop { line, .. } => *line,
            Statement::ForIn { line, .. } => *line,
            Statement::Match { line, .. } => *line,
            Statement::Try { line, .. } => *line,
            Statement::Import { line, .. } => *line,
            Statement::Export { line, .. } => *line,
            Statement::Break { line } => *line,
            Statement::Continue { line } => *line,
            Statement::ExprStmt { line, .. } => *line,
            Statement::Print { line, .. } => *line,
            Statement::Field { line, .. } => *line,
            Statement::FieldAssign { line, .. } => *line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn run(source: &str) -> Result<Val, ElangError> {
        let tokens = tokenize(source)?;
        let prog = parse(tokens)?;
        let mut interp = Interpreter::new();
        interp.run(&prog)
    }

    #[test]
    fn test_division_by_zero_in_function_shows_name_and_line() {
        let source = "def calculate(x):\n  return 10 / x\nend\nlet y = calculate(0)";
        let result = run(source);
        assert!(result.is_err());
        match result {
            Err(ElangError::RuntimeError { message, line, stack }) => {
                assert_eq!(message, "division by zero");
                assert_eq!(line, 2);
                assert!(
                    stack.len() >= 2,
                    "stack too short: {:?}",
                    stack
                );
                assert!(
                    stack.iter().any(|s| s.contains("calculate")),
                    "no calculate in stack: {:?}",
                    stack
                );
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_nested_call_chain_shows_full_stack_trace() {
        let source =
            "def inner():\n  return 10 / 0\nend\ndef outer():\n  return inner()\nend\nlet x = outer()";
        let result = run(source);
        assert!(result.is_err());
        match result {
            Err(ElangError::RuntimeError { stack, .. }) => {
                assert!(
                    stack.len() >= 2,
                    "stack too short: {:?}",
                    stack
                );
                assert!(
                    stack.iter().any(|s| s.contains("inner")),
                    "no inner in stack: {:?}",
                    stack
                );
                assert!(
                    stack.iter().any(|s| s.contains("outer")),
                    "no outer in stack: {:?}",
                    stack
                );
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_error_at_top_level_empty_stack() {
        let source = "let x = 10 / 0";
        let result = run(source);
        assert!(result.is_err());
        match result {
            Err(ElangError::RuntimeError { message, line, stack }) => {
                assert_eq!(message, "division by zero");
                assert_eq!(line, 1);
                assert!(stack.is_empty() || stack[0].contains("<main>"));
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_valid_program_parses_and_runs() {
        let source = "let x = 10\nlet y = x + 5\nprint y";
        let result = run(source);
        assert!(result.is_ok());
    }
}

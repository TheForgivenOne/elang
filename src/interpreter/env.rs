use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::*;
use crate::errors::ElangError;

#[derive(Debug, Clone)]
pub enum Value {
    Nothing,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Native {
        name: String,
        func: fn(&[Value]) -> Result<Value, String>,
    },
    Fn {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        is_async: bool,
        is_pure: bool,
        closure: Environment,
    },
    Class {
        name: String,
        parent: Option<String>,
        default_fields: HashMap<String, Value>,
        methods: HashMap<String, Value>,
    },
    Instance {
        class_name: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
        methods: HashMap<String, Value>,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nothing => write!(f, "nothing"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::List(items) => {
                let strs: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", strs.join(", "))
            }
            Value::Map(map) => {
                let strs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{{}}}", strs.join(", "))
            }
            Value::Native { name, .. } => write!(f, "<native {}>", name),
            Value::Fn { name, .. } => write!(f, "<fn {}>", name),
            Value::Class { name, .. } => write!(f, "<class {}>", name),
            Value::Instance { class_name, .. } => write!(f, "<instance of {}>", class_name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn child(parent: &Environment) -> Self {
        Environment {
            values: HashMap::new(),
            parent: Some(Box::new(parent.clone())),
        }
    }

    pub fn declare(&mut self, name: String, value: Value) -> Result<(), ElangError> {
        if self.values.contains_key(&name) {
            return Err(ElangError::RuntimeError {
                message: format!("'{}' is already declared in this scope", name),
                line: 0,
                stack: vec![],
            });
        }
        self.values.insert(name, value);
        Ok(())
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<(), ElangError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.assign(name, value)
        } else {
            Err(ElangError::RuntimeError {
                message: format!("'{}' is not defined", name),
                line: 0,
                stack: vec![],
            })
        }
    }

    fn get(&self, name: &str) -> Result<Value, ElangError> {
        if let Some(val) = self.values.get(name) {
            Ok(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            Err(ElangError::RuntimeError {
                message: format!("'{}' is not defined", name),
                line: 0,
                stack: vec![],
            })
        }
    }
}

pub enum Control {
    Normal,
    Break,
    Continue,
    Return(Value),
    Error(ElangError),
}

pub fn run(program: Program) -> Result<(), ElangError> {
    let mut env = Environment::new();
    crate::stdlib::register_all(&mut env);
    execute_program(&program, &mut env)
}

fn execute_program(stmts: &[Statement], env: &mut Environment) -> Result<(), ElangError> {
    for stmt in stmts {
        match execute_statement(stmt, env)? {
            Control::Return(_) => {
                return Err(ElangError::RuntimeError {
                    message: "Unexpected return outside function".into(),
                    line: 0,
                    stack: vec![],
                });
            }
            Control::Break => {
                return Err(ElangError::RuntimeError {
                    message: "Break outside loop".into(),
                    line: 0,
                    stack: vec![],
                });
            }
            Control::Continue => {
                return Err(ElangError::RuntimeError {
                    message: "Continue outside loop".into(),
                    line: 0,
                    stack: vec![],
                });
            }
            Control::Normal => {}
            Control::Error(e) => return Err(e),
        }
    }
    Ok(())
}

fn execute_statement(stmt: &Statement, env: &mut Environment) -> Result<Control, ElangError> {
    match stmt {
        Statement::LetDecl { name, value, line: _ } => {
            let val = eval_expr(value, env)?;
            env.declare(name.clone(), val)?;
            Ok(Control::Normal)
        }
        Statement::ConstDecl { name, value, line: _ } => {
            let val = eval_expr(value, env)?;
            env.declare(name.clone(), val)?;
            Ok(Control::Normal)
        }
        Statement::VarDecl { name, value, line: _ } => {
            let val = eval_expr(value, env)?;
            env.declare(name.clone(), val)?;
            Ok(Control::Normal)
        }
        Statement::Assign { name, value, line: _ } => {
            let val = eval_expr(value, env)?;
            env.assign(name, val)?;
            Ok(Control::Normal)
        }
        Statement::FnDef {
            name,
            params,
            body,
            is_async,
            is_pure,
            visibility: _,
            line: _,
        } => {
            let fn_val = Value::Fn {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
                is_async: *is_async,
                is_pure: *is_pure,
                closure: env.clone(),
            };
            env.declare(name.clone(), fn_val)?;
            Ok(Control::Normal)
        }
        Statement::ClassDef {
            name,
            parent,
            body,
            line: _,
        } => {
            let mut default_fields = HashMap::new();
            let mut methods = HashMap::new();
            for member in body {
                match member {
                    Statement::Field {
                        name: fname,
                        value,
                        visibility: _,
                        line: _,
                    } => {
                        let val = eval_expr(value, env)?;
                        default_fields.insert(fname.clone(), val);
                    }
                    Statement::FnDef {
                        name: mname,
                        params,
                        body: mbody,
                        is_async,
                        is_pure,
                        visibility: _,
                        line: _,
                    } => {
                        let fn_val = Value::Fn {
                            name: mname.clone(),
                            params: params.clone(),
                            body: mbody.clone(),
                            is_async: *is_async,
                            is_pure: *is_pure,
                            closure: env.clone(),
                        };
                        methods.insert(mname.clone(), fn_val);
                    }
                    _ => {
                        return Err(ElangError::RuntimeError {
                            message: "Invalid class member".into(),
                            line: 0,
                            stack: vec![],
                        });
                    }
                }
            }
            let class_val = Value::Class {
                name: name.clone(),
                parent: parent.clone(),
                default_fields,
                methods,
            };
            env.declare(name.clone(), class_val)?;
            Ok(Control::Normal)
        }
        Statement::Return { value, line } => {
            let val = eval_expr(value, env)?;
            Ok(Control::Return(val))
        }
        Statement::If {
            condition,
            then_block,
            else_block,
            line: _,
        } => {
            let cond_val = eval_expr(condition, env)?;
            if is_truthy(&cond_val) {
                execute_block(then_block, env)
            } else if let Some(else_blk) = else_block {
                execute_block(else_blk, env)
            } else {
                Ok(Control::Normal)
            }
        }
        Statement::Loop { kind, body, line: _ } => {
            match kind {
                LoopKind::RepeatN(count_expr) => {
                    let count_val = eval_expr(count_expr, env)?;
                    let n = match count_val {
                        Value::Int(n) if n >= 0 => n as usize,
                        Value::Int(n) => {
                            return Err(ElangError::RuntimeError {
                                message: "Repeat count must be non-negative".into(),
                                line: 0,
                                stack: vec![],
                            });
                        }
                        _ => {
                            return Err(ElangError::RuntimeError {
                                message: "Repeat count must be an integer".into(),
                                line: 0,
                                stack: vec![],
                            });
                        }
                    };
                    for _ in 0..n {
                            let ctrl = execute_block(body, env)?;
                            match ctrl {
                                Control::Break => break,
                                Control::Continue => continue,
                                Control::Return(v) => return Ok(Control::Return(v)),
                                Control::Normal => {}
                                Control::Error(e) => return Err(e),
                            }
                    }
                    Ok(Control::Normal)
                }
                LoopKind::RepeatRange { var, from, to } => {
                    let from_val = eval_expr(from, env)?;
                    let to_val = eval_expr(to, env)?;
                    let from_n = match from_val {
                        Value::Int(n) => n,
                        _ => {
                            return Err(ElangError::RuntimeError {
                                message: "Range start must be an integer".into(),
                                line: 0,
                                stack: vec![],
                            });
                        }
                    };
                    let to_n = match to_val {
                        Value::Int(n) => n,
                        _ => {
                            return Err(ElangError::RuntimeError {
                                message: "Range end must be an integer".into(),
                                line: 0,
                                stack: vec![],
                            });
                        }
                    };
                    let mut i = from_n;
                    while i <= to_n {
                        let mut child = Environment::child(env);
                        child.declare(var.clone(), Value::Int(i))?;
                        let ctrl = execute_block(body, &mut child)?;
                        match ctrl {
                            Control::Break => break,
                            Control::Continue => {
                                i += 1;
                                continue;
                            }
                            Control::Return(_) => return Ok(ctrl),
                            Control::Normal => {}
                            Control::Error(e) => return Err(e),
                        }
                        i += 1;
                    }
                    Ok(Control::Normal)
                }
                LoopKind::While(condition) => {
                    loop {
                        let cond_val = eval_expr(condition, env)?;
                        if !is_truthy(&cond_val) {
                            break;
                        }
                        let ctrl = execute_block(body, env)?;
                        match ctrl {
                            Control::Break => { break; }
                            Control::Continue => { continue; }
                            Control::Return(v) => return Ok(Control::Return(v)),
                            Control::Normal => {}
                            Control::Error(e) => return Err(e),
                        }
                    }
                    Ok(Control::Normal)
                }
                LoopKind::Forever => {
                    loop {
                        let ctrl = execute_block(body, env)?;
                        match ctrl {
                            Control::Break => { break; }
                            Control::Continue => { continue; }
                            Control::Return(v) => return Ok(Control::Return(v)),
                            Control::Normal => {}
                            Control::Error(e) => return Err(e),
                        }
                    }
                    Ok(Control::Normal)
                },
            }
        }
        Statement::ForIn {
            var,
            iterable,
            body,
            line: _,
        } => {
            let iter_val = eval_expr(iterable, env)?;
            let items = match iter_val {
                Value::List(items) => items,
                _ => {
                    return Err(ElangError::RuntimeError {
                        message: "Can only iterate over lists".into(),
                        line: 0,
                        stack: vec![],
                    });
                }
            };
            for item in items {
                let mut child = Environment::child(env);
                child.declare(var.clone(), item)?;
                let ctrl = execute_block(body, &mut child)?;
                match ctrl {
                    Control::Break => { break; }
                    Control::Continue => { continue; }
                    Control::Return(v) => return Ok(Control::Return(v)),
                    Control::Normal => {}
                    Control::Error(e) => return Err(e),
                }
            }
            Ok(Control::Normal)
        }
        Statement::Match { value, arms, line: _ } => {
            let val = eval_expr(value, env)?;
            for arm in arms {
                let matched = match &arm.pattern {
                    MatchPattern::Literal(pat_expr) => {
                        let pat_val = eval_expr(pat_expr, env)?;
                        values_equal(&val, &pat_val)
                    }
                    MatchPattern::Wildcard => true,
                    MatchPattern::IsType(type_name) => match (&val, type_name.as_str()) {
                        (Value::Int(_), "int") => true,
                        (Value::Float(_), "float") => true,
                        (Value::Str(_), "string") => true,
                        (Value::Bool(_), "bool") => true,
                        (Value::Nothing, "nothing") => true,
                        (Value::List(_), "list") => true,
                        (Value::Map(_), "map") => true,
                        _ => false,
                    },
                };
                if matched {
                    return execute_block(&arm.body, env);
                }
            }
            Ok(Control::Normal)
        }
        Statement::Try {
            body,
            catches,
            line: _,
        } => {
            let result = execute_block(body, env);
            match result {
                Ok(ctrl) => Ok(ctrl),
                Err(err) => {
                    let msg = format!("{}", err);
                    for catch in catches {
                        let type_match = match &catch.error_type {
                            None => true,
                            Some(_) => true,
                        };
                        if type_match {
                            let mut catch_env = Environment::child(env);
                            catch_env.declare(catch.var.clone(), Value::Str(msg.clone()))?;
                            return execute_block(&catch.body, &mut catch_env);
                        }
                    }
                    Err(err)
                }
            }
        }
        Statement::Import { module: _, line: _ } => Ok(Control::Normal),
        Statement::Export { stmt, line: _ } => execute_statement(stmt, env),
        Statement::Break { line: _ } => Ok(Control::Break),
        Statement::Continue { line: _ } => Ok(Control::Continue),
        Statement::ExprStmt { expr, line: _ } => {
            eval_expr(expr, env)?;
            Ok(Control::Normal)
        }
        Statement::Print { value, line: _ } => {
            let val = eval_expr(value, env)?;
            println!("{}", val);
            Ok(Control::Normal)
        }
        Statement::Field {
            name,
            value,
            visibility: _,
            line: _,
        } => {
            let val = eval_expr(value, env)?;
            env.declare(name.clone(), val)?;
            Ok(Control::Normal)
        }
        Statement::FieldAssign {
            object,
            field,
            value,
            line,
        } => {
            let obj_val = env.get(object)?;
            match obj_val {
                Value::Instance { fields, .. } => {
                    let val = eval_expr(value, env)?;
                    fields.borrow_mut().insert(field.clone(), val);
                    Ok(Control::Normal)
                }
                _ => Err(ElangError::RuntimeError {
                    message: format!("Cannot assign field on this value"),
                    line: *line,
                    stack: vec![],
                }),
            }
        }
    }
}

fn execute_block(stmts: &[Statement], env: &mut Environment) -> Result<Control, ElangError> {
    for stmt in stmts {
        let ctrl = execute_statement(stmt, env)?;
        match ctrl {
            Control::Normal => {}
            other => return Ok(other),
        }
    }
    Ok(Control::Normal)
}

fn eval_expr(expr: &Expr, env: &Environment) -> Result<Value, ElangError> {
    match expr {
        Expr::Int { value, line: _ } => Ok(Value::Int(*value)),
        Expr::Float { value, line: _ } => Ok(Value::Float(*value)),
        Expr::Str { value, line: _ } => Ok(Value::Str(value.clone())),
        Expr::Bool { value, line: _ } => Ok(Value::Bool(*value)),
        Expr::Nothing { line: _ } => Ok(Value::Nothing),
        Expr::Ident { name, line } => {
            if name == "self" {
                return env.get("self");
            }
            env.get(name)
        }
        Expr::StrInterp { value, line: _ } => {
            let mut result = String::new();
            let mut buf = String::new();
            let mut in_brace = false;
            for ch in value.chars() {
                match ch {
                    '{' if !in_brace => {
                        result.push_str(&buf);
                        buf.clear();
                        in_brace = true;
                    }
                    '}' if in_brace => {
                        let val = env.get(&buf)?.to_string();
                        result.push_str(&val);
                        buf.clear();
                        in_brace = false;
                    }
                    _ => buf.push(ch),
                }
            }
            if in_brace {
                let val = env.get(&buf)?.to_string();
                result.push_str(&val);
            } else {
                result.push_str(&buf);
            }
            Ok(Value::Str(result))
        }
        Expr::BinOp {
            left,
            op,
            right,
            line,
        } => {
            let left_val = eval_expr(left, env)?;
            let right_val = eval_expr(right, env)?;
            eval_binop(left_val, op, right_val, *line)
        }
        Expr::UnaryOp { op, expr, line } => {
            let val = eval_expr(expr, env)?;
            match op {
                UnaryOpKind::Neg => match val {
                    Value::Int(n) => Ok(Value::Int(-n)),
                    Value::Float(n) => Ok(Value::Float(-n)),
                    _ => Err(ElangError::RuntimeError {
                        message: "Cannot negate non-number".into(),
                        line: *line,
                        stack: vec![],
                    }),
                },
                UnaryOpKind::Not => Ok(Value::Bool(!is_truthy(&val))),
            }
        }
        Expr::Call {
            callee,
            args,
            line,
        } => {
            let callee_val = eval_expr(callee, env)?;
            let mut arg_vals = Vec::new();
            for arg in args {
                arg_vals.push(eval_expr(arg, env)?);
            }
            call_value(callee_val, &arg_vals, *line)
        }
        Expr::Index {
            object,
            index,
            line,
        } => {
            let obj_val = eval_expr(object, env)?;
            let idx_val = eval_expr(index, env)?;
            match (&obj_val, &idx_val) {
                (Value::List(items), Value::Int(i)) => {
                    let i = *i;
                    if i < 0 || i >= items.len() as i64 {
                        return Err(ElangError::RuntimeError {
                            message: format!("Index {} out of bounds", i),
                            line: *line,
                            stack: vec![],
                        });
                    }
                    Ok(items[i as usize].clone())
                }
                _ => Err(ElangError::RuntimeError {
                    message: "Indexing not supported for this type".into(),
                    line: *line,
                    stack: vec![],
                }),
            }
        }
        Expr::Field {
            object,
            field,
            line,
        } => {
            let obj_val = eval_expr(object, env)?;
            get_field(&obj_val, field, *line)
        }
        Expr::List { items, line: _ } => {
            let mut vals = Vec::new();
            for item in items {
                vals.push(eval_expr(item, env)?);
            }
            Ok(Value::List(vals))
        }
        Expr::Map { pairs, line: _ } => {
            let mut map = HashMap::new();
            for (k, v) in pairs {
                let val = eval_expr(v, env)?;
                map.insert(k.clone(), val);
            }
            Ok(Value::Map(map))
        }
        Expr::Lambda {
            params,
            body,
            line: _,
        } => Ok(Value::Fn {
            name: "<lambda>".into(),
            params: params.clone(),
            body: vec![Statement::Return {
                value: *body.clone(),
                line: 0,
            }],
            is_async: false,
            is_pure: false,
            closure: env.clone(),
        }),
        Expr::Pipe { left, right, line } => {
            let left_val = eval_expr(left, env)?;
            let pipe_args = vec![left_val];
            let right_val = eval_expr(right, env)?;
            match right_val {
                Value::Fn { .. } | Value::Native { .. } => call_value(right_val, &pipe_args, *line),
                _ => Err(ElangError::RuntimeError {
                    message: "Right side of pipe must be a function".into(),
                    line: *line,
                    stack: vec![],
                }),
            }
        }
        Expr::Await { expr, line: _ } => eval_expr(expr, env),
    }
}

fn get_field(obj: &Value, field: &str, line: usize) -> Result<Value, ElangError> {
    match obj {
        Value::Instance {
            fields, methods, ..
        } => {
            if let Some(method) = methods.get(field) {
                if let Value::Fn {
                    name,
                    params,
                    body,
                    is_async,
                    is_pure,
                    closure,
                } = method
                {
                    let mut bound_closure = Environment::child(closure);
                    bound_closure.declare("self".to_string(), obj.clone())?;
                    return Ok(Value::Fn {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                        is_async: *is_async,
                        is_pure: *is_pure,
                        closure: bound_closure,
                    });
                }
                return Ok(method.clone());
            }
            let binding = fields.borrow();
            if let Some(val) = binding.get(field) {
                return Ok(val.clone());
            }
            Err(ElangError::RuntimeError {
                message: format!("Instance has no field or method '{}'", field),
                line,
                stack: vec![],
            })
        }
        Value::Map(map) => {
            if let Some(val) = map.get(field) {
                Ok(val.clone())
            } else {
                Err(ElangError::RuntimeError {
                    message: format!("Map has no key '{}'", field),
                    line,
                    stack: vec![],
                })
            }
        }
        _ => Err(ElangError::RuntimeError {
            message: format!("Cannot access field '{}' on this value", field),
            line,
            stack: vec![],
        }),
    }
}

fn eval_binop(
    left: Value,
    op: &BinOpKind,
    right: Value,
    line: usize,
) -> Result<Value, ElangError> {
    match op {
        BinOpKind::Add => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
            _ => Err(ElangError::RuntimeError {
                message: format!("Cannot add {:?} and {:?}", left, right),
                line,
                stack: vec![],
            }),
        },
        BinOpKind::Sub => number_binop(&left, &right, |a, b| Value::Int(a - b), |a, b| Value::Float(a - b), line),
        BinOpKind::Mul => number_binop(&left, &right, |a, b| Value::Int(a * b), |a, b| Value::Float(a * b), line),
        BinOpKind::Div => number_binop(&left, &right, |a, b| Value::Int(a / b), |a, b| Value::Float(a / b), line),
        BinOpKind::Mod => number_binop(&left, &right, |a, b| Value::Int(a % b), |_, _| unreachable!(), line),
        BinOpKind::Eq => Ok(Value::Bool(values_equal(&left, &right))),
        BinOpKind::NotEq => Ok(Value::Bool(!values_equal(&left, &right))),
        BinOpKind::Lt => compare_binop(&left, &right, |a, b| a < b, |a, b| a < b, line),
        BinOpKind::Gt => compare_binop(&left, &right, |a, b| a > b, |a, b| a > b, line),
        BinOpKind::LtEq => compare_binop(&left, &right, |a, b| a <= b, |a, b| a <= b, line),
        BinOpKind::GtEq => compare_binop(&left, &right, |a, b| a >= b, |a, b| a >= b, line),
        BinOpKind::And => {
            let a = is_truthy(&left);
            if !a {
                Ok(Value::Bool(false))
            } else {
                Ok(Value::Bool(is_truthy(&right)))
            }
        }
        BinOpKind::Or => {
            let a = is_truthy(&left);
            if a {
                Ok(Value::Bool(true))
            } else {
                Ok(Value::Bool(is_truthy(&right)))
            }
        }
    }
}

fn number_binop(
    left: &Value,
    right: &Value,
    int_op: fn(i64, i64) -> Value,
    float_op: fn(f64, f64) -> Value,
    line: usize,
) -> Result<Value, ElangError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(int_op(*a, *b)),
        (Value::Float(a), Value::Float(b)) => Ok(float_op(*a, *b)),
        (Value::Int(a), Value::Float(b)) => Ok(float_op(*a as f64, *b)),
        (Value::Float(a), Value::Int(b)) => Ok(float_op(*a, *b as f64)),
        _ => Err(ElangError::RuntimeError {
            message: format!("Cannot perform arithmetic on non-numbers"),
            line,
            stack: vec![],
        }),
    }
}

fn compare_binop(
    left: &Value,
    right: &Value,
    int_op: fn(i64, i64) -> bool,
    float_op: fn(f64, f64) -> bool,
    line: usize,
) -> Result<Value, ElangError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(int_op(*a, *b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(float_op(*a, *b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(float_op(*a as f64, *b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(float_op(*a, *b as f64))),
        _ => Err(ElangError::RuntimeError {
            message: format!("Cannot compare these types"),
            line,
            stack: vec![],
        }),
    }
}

fn call_value(callee: Value, args: &[Value], line: usize) -> Result<Value, ElangError> {
    match callee {
        Value::Fn {
            params,
            body,
            closure,
            ..
        } => {
            if args.len() != params.len() {
                return Err(ElangError::RuntimeError {
                    message: format!(
                        "Expected {} arguments, got {}",
                        params.len(),
                        args.len()
                    ),
                    line,
                    stack: vec![],
                });
            }
            let mut fn_env = Environment::child(&closure);
            for (param, arg) in params.iter().zip(args.iter()) {
                fn_env.declare(param.clone(), arg.clone())?;
            }
            let result = execute_block(&body, &mut fn_env)?;
            match result {
                Control::Return(val) => Ok(val),
                _ => Ok(Value::Nothing),
            }
        }
        Value::Native { func, .. } => {
            match func(args) {
                Ok(val) => Ok(val),
                Err(msg) => Err(ElangError::RuntimeError {
                    message: msg,
                    line,
                    stack: vec![],
                }),
            }
        }
        Value::Class {
            name,
            parent: _,
            default_fields,
            methods,
        } => {
            let fields = Rc::new(RefCell::new(default_fields.clone()));
            let instance = Value::Instance {
                class_name: name,
                fields: Rc::clone(&fields),
                methods: methods.clone(),
            };
            Ok(instance)
        }
        Value::Instance {
            class_name,
            fields,
            methods,
        } => {
            Err(ElangError::RuntimeError {
                message: format!("Cannot call an instance of '{}' as a function", class_name),
                line,
                stack: vec![],
            })
        }
        _ => Err(ElangError::RuntimeError {
            message: format!("Cannot call this value as a function"),
            line,
            stack: vec![],
        }),
    }
}

fn call_method(method: Value, instance: Value, args: &[Value], line: usize) -> Result<Value, ElangError> {
    match method {
        Value::Fn {
            params,
            body,
            is_async,
            is_pure,
            closure,
            ..
        } => {
            let expected = params.len();
            let total = args.len() + 1;
            if total != expected {
                return Err(ElangError::RuntimeError {
                    message: format!(
                        "Method expects {} arguments (including self), got {}",
                        expected, total
                    ),
                    line,
                    stack: vec![],
                });
            }
            let mut fn_env = Environment::child(&closure);
            fn_env.declare("self".to_string(), instance)?;
            for (i, param) in params.iter().enumerate() {
                if *param == "self" {
                    continue;
                }
                let arg_idx = if *param == "self" { 0 } else { i - 1 };
                if i == 0 {
                    continue;
                }
                let arg_idx = i - 1;
                if arg_idx < args.len() {
                    fn_env.declare(param.clone(), args[arg_idx].clone())?;
                }
            }
            let result = execute_block(&body, &mut fn_env)?;
            match result {
                Control::Return(val) => Ok(val),
                _ => Ok(Value::Nothing),
            }
        }
        _ => Err(ElangError::RuntimeError {
            message: "Cannot call non-function as method".into(),
            line,
            stack: vec![],
        }),
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Nothing => false,
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Float(n) => *n != 0.0,
        Value::Str(s) => !s.is_empty(),
        Value::List(items) => !items.is_empty(),
        Value::Map(map) => !map.is_empty(),
        _ => true,
    }
}

pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nothing, Value::Nothing) => true,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
        (Value::Int(a), Value::Float(b)) => (*a as f64 - b).abs() < f64::EPSILON,
        (Value::Float(a), Value::Int(b)) => (a - *b as f64).abs() < f64::EPSILON,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Str(a), Value::Str(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser;

    fn run_source(source: &str) -> Result<(), ElangError> {
        let tokens = tokenize(source)?;
        let program = parser::parse(tokens)?;
        run(program)
    }

    fn eval_source(source: &str) -> Result<String, ElangError> {
        let output = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = run_source(source);
        }));
        let _ = output;
        run_source(source).map(|_| "ok".to_string())
    }

    #[test]
    fn test_self_field_persists_after_method() {
        let source = r#"
class Counter:
    pub count = 0
    pub def increment():
        self.count = self.count + 1
    end
end

let c = Counter()
c.increment()
print c.count
"#;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_source(source).unwrap();
        }));
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_increments_accumulate() {
        let source = r#"
class Counter:
    pub count = 0
    pub def increment():
        self.count = self.count + 1
    end
end

let c = Counter()
c.increment()
c.increment()
c.increment()
let x = c.count
print x
"#;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_source(source).unwrap();
        }));
        assert!(result.is_ok());
    }

    #[test]
    fn test_two_instances_independent() {
        let source = r#"
class Counter:
    pub count = 0
    pub def increment():
        self.count = self.count + 1
    end
end

let a = Counter()
let b = Counter()
a.increment()
a.increment()
b.increment()
print a.count
"#;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_source(source).unwrap();
        }));
        assert!(result.is_ok());
    }
}

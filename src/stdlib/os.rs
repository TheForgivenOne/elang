use std::collections::HashMap;
use std::env as std_env;
use std::process;

use crate::interpreter::Value;

fn make_native(name: &str, func: fn(&[Value]) -> Result<Value, String>) -> Value {
    Value::Native { name: name.to_string(), func }
}

fn check_args(name: &str, args: &[Value], count: usize) -> Result<(), String> {
    if args.len() != count {
        return Err(format!("{} takes {} argument(s), got {}", name, count, args.len()));
    }
    Ok(())
}

fn os_env(args: &[Value]) -> Result<Value, String> {
    check_args("os.env", args, 1)?;
    let key = match &args[0] {
        Value::Str(s) => s.clone(),
        _ => return Err("os.env requires a string key".into()),
    };
    match std_env::var(&key) {
        Ok(val) => Ok(Value::Str(val)),
        Err(_) => Ok(Value::Nothing),
    }
}

fn os_exit(args: &[Value]) -> Result<Value, String> {
    check_args("os.exit", args, 1)?;
    let code = match &args[0] {
        Value::Int(n) => *n as i32,
        _ => return Err("os.exit requires an integer code".into()),
    };
    process::exit(code);
}

fn os_cwd(_args: &[Value]) -> Result<Value, String> {
    check_args("os.cwd", _args, 0)?;
    match std_env::current_dir() {
        Ok(path) => Ok(Value::Str(path.to_string_lossy().to_string())),
        Err(e) => Err(format!("os.cwd error: {}", e)),
    }
}

fn os_args(_args: &[Value]) -> Result<Value, String> {
    check_args("os.args", _args, 0)?;
    let args: Vec<Value> = std_env::args().map(|a| Value::Str(a)).collect();
    Ok(Value::List(args))
}

pub fn make_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("env".into(), make_native("os.env", os_env));
    m.insert("exit".into(), make_native("os.exit", os_exit));
    m.insert("cwd".into(), make_native("os.cwd", os_cwd));
    m.insert("args".into(), make_native("os.args", os_args));
    m
}

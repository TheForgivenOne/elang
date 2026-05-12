use std::collections::HashMap;
use std::fs;
use std::io::{self as std_io, Write, BufRead};

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

fn io_read(args: &[Value]) -> Result<Value, String> {
    check_args("io.read", args, 1)?;
    let path = match &args[0] {
        Value::Str(s) => s.clone(),
        _ => return Err("io.read requires a string path".into()),
    };
    fs::read_to_string(&path)
        .map(Value::Str)
        .map_err(|e| format!("io.read error: {}", e))
}

fn io_write(args: &[Value]) -> Result<Value, String> {
    check_args("io.write", args, 2)?;
    let path = match &args[0] {
        Value::Str(s) => s.clone(),
        _ => return Err("io.write requires a string path".into()),
    };
    let content = match &args[1] {
        Value::Str(s) => s.clone(),
        _ => return Err("io.write requires string content".into()),
    };
    fs::write(&path, &content)
        .map(|_| Value::Nothing)
        .map_err(|e| format!("io.write error: {}", e))
}

fn io_input(args: &[Value]) -> Result<Value, String> {
    check_args("io.input", args, 1)?;
    let prompt = match &args[0] {
        Value::Str(s) => s.clone(),
        _ => return Err("io.input requires a string prompt".into()),
    };
    print!("{}", prompt);
    std_io::stdout().flush().map_err(|e| format!("io.input error: {}", e))?;
    let mut line = String::new();
    std_io::stdin().lock().read_line(&mut line).map_err(|e| format!("io.input error: {}", e))?;
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(Value::Str(line))
}

pub fn make_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("read".into(), make_native("io.read", io_read));
    m.insert("write".into(), make_native("io.write", io_write));
    m.insert("input".into(), make_native("io.input", io_input));
    m
}

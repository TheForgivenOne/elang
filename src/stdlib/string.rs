use std::collections::HashMap;

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

fn to_string(val: &Value) -> Result<String, String> {
    match val {
        Value::Str(s) => Ok(s.clone()),
        _ => Err("expected a string argument".into()),
    }
}

fn str_find(args: &[Value]) -> Result<Value, String> {
    check_args("string.find", args, 2)?;
    let s = to_string(&args[0])?;
    let sub = to_string(&args[1])?;
    match s.find(&sub) {
        Some(i) => Ok(Value::Int(i as i64)),
        None => Ok(Value::Int(-1)),
    }
}

fn str_replace(args: &[Value]) -> Result<Value, String> {
    check_args("string.replace", args, 3)?;
    let s = to_string(&args[0])?;
    let old = to_string(&args[1])?;
    let new = to_string(&args[2])?;
    Ok(Value::Str(s.replace(&old, &new)))
}

fn str_starts_with(args: &[Value]) -> Result<Value, String> {
    check_args("string.starts_with", args, 2)?;
    let s = to_string(&args[0])?;
    let prefix = to_string(&args[1])?;
    Ok(Value::Bool(s.starts_with(&prefix)))
}

fn str_ends_with(args: &[Value]) -> Result<Value, String> {
    check_args("string.ends_with", args, 2)?;
    let s = to_string(&args[0])?;
    let suffix = to_string(&args[1])?;
    Ok(Value::Bool(s.ends_with(&suffix)))
}

fn str_repeat(args: &[Value]) -> Result<Value, String> {
    check_args("string.repeat", args, 2)?;
    let s = to_string(&args[0])?;
    let n = match &args[1] {
        Value::Int(n) if *n >= 0 => *n as usize,
        Value::Int(_) => return Err("string.repeat requires a non-negative count".into()),
        _ => return Err("string.repeat requires an integer count".into()),
    };
    Ok(Value::Str(s.repeat(n)))
}

fn str_pad_left(args: &[Value]) -> Result<Value, String> {
    check_args("string.pad_left", args, 3)?;
    let s = to_string(&args[0])?;
    let n = match &args[1] {
        Value::Int(n) if *n >= 0 => *n as usize,
        Value::Int(_) => return Err("string.pad_left requires a non-negative width".into()),
        _ => return Err("string.pad_left requires an integer width".into()),
    };
    let ch = match &args[2] {
        Value::Str(c) if c.chars().count() == 1 => c.chars().next().unwrap(),
        _ => return Err("string.pad_left requires a single character string".into()),
    };
    if s.len() >= n {
        Ok(Value::Str(s))
    } else {
        Ok(Value::Str(format!("{}{}", ch.to_string().repeat(n - s.len()), s)))
    }
}

fn str_pad_right(args: &[Value]) -> Result<Value, String> {
    check_args("string.pad_right", args, 3)?;
    let s = to_string(&args[0])?;
    let n = match &args[1] {
        Value::Int(n) if *n >= 0 => *n as usize,
        Value::Int(_) => return Err("string.pad_right requires a non-negative width".into()),
        _ => return Err("string.pad_right requires an integer width".into()),
    };
    let ch = match &args[2] {
        Value::Str(c) if c.chars().count() == 1 => c.chars().next().unwrap(),
        _ => return Err("string.pad_right requires a single character string".into()),
    };
    if s.len() >= n {
        Ok(Value::Str(s))
    } else {
        Ok(Value::Str(format!("{}{}", s, ch.to_string().repeat(n - s.len()))))
    }
}

pub fn make_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("find".into(), make_native("string.find", str_find));
    m.insert("replace".into(), make_native("string.replace", str_replace));
    m.insert("starts_with".into(), make_native("string.starts_with", str_starts_with));
    m.insert("ends_with".into(), make_native("string.ends_with", str_ends_with));
    m.insert("repeat_str".into(), make_native("string.repeat_str", str_repeat));
    m.insert("pad_left".into(), make_native("string.pad_left", str_pad_left));
    m.insert("pad_right".into(), make_native("string.pad_right", str_pad_right));
    m
}

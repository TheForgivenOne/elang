use std::collections::HashMap;
use std::thread;
use std::time::Duration;

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

fn to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Float(n) => Some(*n),
        _ => None,
    }
}

fn time_now(_args: &[Value]) -> Result<Value, String> {
    check_args("time.now", _args, 0)?;
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    Ok(Value::Str(format!("{}.{:09}", d.as_secs(), d.subsec_nanos())))
}

fn time_sleep(args: &[Value]) -> Result<Value, String> {
    check_args("time.sleep", args, 1)?;
    let secs = to_f64(&args[0]).ok_or_else::<String, _>(|| "time.sleep requires a number".into())?;
    let millis = (secs * 1000.0) as u64;
    thread::sleep(Duration::from_millis(millis));
    Ok(Value::Nothing)
}

pub fn make_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("now".into(), make_native("time.now", time_now));
    m.insert("sleep".into(), make_native("time.sleep", time_sleep));
    m
}

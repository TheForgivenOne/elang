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

fn to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Float(n) => Some(*n),
        _ => None,
    }
}

fn rand_val() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as u64;
    let mut state = nanos.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (state >> 33) as f64 / (1u64 << 31) as f64
}

fn math_sqrt(args: &[Value]) -> Result<Value, String> {
    check_args("math.sqrt", args, 1)?;
    to_f64(&args[0]).map(|x| Value::Float(x.sqrt())).ok_or_else(|| "math.sqrt requires a number".into())
}

fn math_floor(args: &[Value]) -> Result<Value, String> {
    check_args("math.floor", args, 1)?;
    to_f64(&args[0]).map(|x| Value::Int(x.floor() as i64)).ok_or_else(|| "math.floor requires a number".into())
}

fn math_ceil(args: &[Value]) -> Result<Value, String> {
    check_args("math.ceil", args, 1)?;
    to_f64(&args[0]).map(|x| Value::Int(x.ceil() as i64)).ok_or_else(|| "math.ceil requires a number".into())
}

fn math_abs(args: &[Value]) -> Result<Value, String> {
    check_args("math.abs", args, 1)?;
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(n.abs())),
        Value::Float(n) => Ok(Value::Float(n.abs())),
        _ => Err("math.abs requires a number".into()),
    }
}

fn math_random(_args: &[Value]) -> Result<Value, String> {
    check_args("math.random", _args, 0)?;
    Ok(Value::Float(rand_val()))
}

fn math_pow(args: &[Value]) -> Result<Value, String> {
    check_args("math.pow", args, 2)?;
    let a = to_f64(&args[0]).ok_or_else::<String, _>(|| "math.pow requires numbers".into())?;
    let b = to_f64(&args[1]).ok_or_else::<String, _>(|| "math.pow requires numbers".into())?;
    Ok(Value::Float(a.powf(b)))
}

const PI: f64 = 3.14159265358979;

pub fn make_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("sqrt".into(), make_native("math.sqrt", math_sqrt));
    m.insert("floor".into(), make_native("math.floor", math_floor));
    m.insert("ceil".into(), make_native("math.ceil", math_ceil));
    m.insert("abs".into(), make_native("math.abs", math_abs));
    m.insert("random".into(), make_native("math.random", math_random));
    m.insert("pow".into(), make_native("math.pow", math_pow));
    m.insert("PI".into(), Value::Float(PI));
    m
}

// Standard library: built-in modules for elang

use std::collections::HashMap;
use std::fs;
use std::io::{self as std_io, Write, BufRead};
use std::thread;
use std::time::Duration;

use crate::interpreter::{Environment, Value};

type NativeFunc = fn(&[Value]) -> Result<Value, String>;

fn make_native(name: &str, func: NativeFunc) -> Value {
    Value::Native { name: name.to_string(), func }
}

fn make_const(name: &str, value: Value) -> (String, Value) {
    (name.to_string(), value)
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

// --- Math module ---

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

fn rand_val() -> f64 {
    // Simple linear congruential generator
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as u64;
    let mut state = nanos.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (state >> 33) as f64 / (1u64 << 31) as f64
}

const PI: f64 = 3.14159265358979;

fn make_math_module() -> HashMap<String, Value> {
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

// --- Time module ---

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

fn make_time_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("now".into(), make_native("time.now", time_now));
    m.insert("sleep".into(), make_native("time.sleep", time_sleep));
    m
}

// --- IO module ---

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

fn make_io_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("read".into(), make_native("io.read", io_read));
    m.insert("write".into(), make_native("io.write", io_write));
    m.insert("input".into(), make_native("io.input", io_input));
    m
}

// --- Registration ---

fn wrap_module(map: HashMap<String, Value>) -> Value {
    Value::Map(map)
}

pub fn register_all(env: &mut Environment) {
    env.declare("math", wrap_module(make_math_module())).ok();
    env.declare("time", wrap_module(make_time_module())).ok();
    env.declare("io", wrap_module(make_io_module())).ok();
}

#[cfg(test)]
mod tests {
    use crate::errors::ElangError;
    use crate::lexer::tokenize;
    use crate::parser;
    use crate::interpreter;

    fn run_source(source: &str) -> Result<(), ElangError> {
        let tokens = tokenize(source)?;
        let program = parser::parse(tokens)?;
        interpreter::run(program)
    }

    #[test]
    fn test_math_sqrt() {
        let output = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_source("import math\nprint math.sqrt(144)").unwrap();
        }));
        assert!(output.is_ok());
    }

    #[test]
    fn test_list_sort() {
        let output = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_source("let nums = [3,1,2]\nnums.sort()\nprint nums").unwrap();
        }));
        assert!(output.is_ok());
    }
}

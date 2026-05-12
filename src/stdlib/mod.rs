use std::collections::HashMap;

use crate::interpreter::{Environment, Value};

pub mod io;
pub mod json;
pub mod math;
pub mod os;
pub mod string;
pub mod time;

fn wrap_module(map: HashMap<String, Value>) -> Value {
    Value::Map(map)
}

pub fn register_all(env: &mut Environment) {
    env.declare("math".to_string(), wrap_module(math::make_module())).ok();
    env.declare("time".to_string(), wrap_module(time::make_module())).ok();
    env.declare("io".to_string(), wrap_module(io::make_module())).ok();
    env.declare("json".to_string(), wrap_module(json::make_module())).ok();
    env.declare("os".to_string(), wrap_module(os::make_module())).ok();
    env.declare("string".to_string(), wrap_module(string::make_module())).ok();
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
        run_source("import math\nprint math.sqrt(144)").unwrap();
    }

    #[test]
    fn test_json_stringify_map_with_list() {
        let source = r#"import json
let data = json.stringify({nums: [1, 2, 3], ok: true})
print data
"#;
        run_source(source).unwrap();
    }

    #[test]
    fn test_string_find() {
        let source = r#"import string
let idx = string.find("hello world", "world")
print idx
"#;
        run_source(source).unwrap();
    }

    #[test]
    fn test_string_repeat() {
        let source = r#"import string
let s = string.repeat_str("hi", 3)
print s
"#;
        run_source(source).unwrap();
    }

    #[test]
    fn test_string_pad_left() {
        let source = r#"import string
let s = string.pad_left("hello", 8, "-")
print s
"#;
        run_source(source).unwrap();
    }

    #[test]
    fn test_os_cwd() {
        let source = r#"import os
let dir = os.cwd()
print dir
"#;
        run_source(source).unwrap();
    }

    #[test]
    fn test_os_env_path() {
        let source = r#"import os
let p = os.env("PATH")
print p
"#;
        run_source(source).unwrap();
    }
}

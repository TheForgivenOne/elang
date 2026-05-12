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

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Parser { chars: input.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('n') => self.parse_null(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(format!("unexpected character '{}' in JSON", c)),
            None => Err("unexpected end of JSON input".into()),
        }
    }

    fn parse_object(&mut self) -> Result<Value, String> {
        self.advance();
        let mut map = HashMap::new();
        self.skip_whitespace();
        if self.peek() == Some('}') {
            self.advance();
            return Ok(Value::Map(map));
        }
        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                Value::Str(s) => s,
                _ => return Err("JSON object key must be a string".into()),
            };
            self.skip_whitespace();
            if self.advance() != Some(':') {
                return Err("expected ':' in JSON object".into());
            }
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => { self.advance(); }
                Some('}') => { self.advance(); return Ok(Value::Map(map)); }
                _ => return Err("expected ',' or '}' in JSON object".into()),
            }
        }
    }

    fn parse_array(&mut self) -> Result<Value, String> {
        self.advance();
        let mut list = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.advance();
            return Ok(Value::List(list));
        }
        loop {
            list.push(self.parse_value()?);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => { self.advance(); }
                Some(']') => { self.advance(); return Ok(Value::List(list)); }
                _ => return Err("expected ',' or ']' in JSON array".into()),
            }
        }
    }

    fn parse_string(&mut self) -> Result<Value, String> {
        self.advance();
        let mut s = String::new();
        loop {
            match self.advance() {
                Some('"') => return Ok(Value::Str(s)),
                Some('\\') => {
                    match self.advance() {
                        Some('"') => s.push('"'),
                        Some('\\') => s.push('\\'),
                        Some('/') => s.push('/'),
                        Some('n') => s.push('\n'),
                        Some('r') => s.push('\r'),
                        Some('t') => s.push('\t'),
                        Some('u') => {
                            let hex: String = (0..4).filter_map(|_| self.advance()).collect();
                            if hex.len() != 4 {
                                return Err("invalid unicode escape in JSON string".into());
                            }
                            let code = u32::from_str_radix(&hex, 16).map_err(|_| <String as From<&str>>::from("invalid unicode escape in JSON string"))?;
                            if let Some(ch) = char::from_u32(code) {
                                s.push(ch);
                            } else {
                                return Err("invalid unicode code point".into());
                            }
                        }
                        Some(c) => s.push(c),
                        None => return Err("unexpected end of JSON string escape".into()),
                    }
                }
                Some(c) => s.push(c),
                None => return Err("unterminated JSON string".into()),
            }
        }
    }

    fn parse_number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.advance();
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        let mut is_float = false;
        if self.peek() == Some('.') {
            is_float = true;
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        if self.peek() == Some('e') || self.peek() == Some('E') {
            is_float = true;
            self.advance();
            if self.peek() == Some('-') || self.peek() == Some('+') {
                self.advance();
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        if is_float {
            let n: f64 = s.parse().map_err(|_| format!("invalid JSON number: {}", s))?;
            Ok(Value::Float(n))
        } else {
            let n: i64 = s.parse().map_err(|_| format!("invalid JSON number: {}", s))?;
            Ok(Value::Int(n))
        }
    }

    fn parse_bool(&mut self) -> Result<Value, String> {
        if self.peek() == Some('t') {
            for c in "true".chars() {
                if self.advance() != Some(c) {
                    return Err("invalid JSON value".into());
                }
            }
            Ok(Value::Bool(true))
        } else {
            for c in "false".chars() {
                if self.advance() != Some(c) {
                    return Err("invalid JSON value".into());
                }
            }
            Ok(Value::Bool(false))
        }
    }

    fn parse_null(&mut self) -> Result<Value, String> {
        for c in "null".chars() {
            if self.advance() != Some(c) {
                return Err("invalid JSON value".into());
            }
        }
        Ok(Value::Nothing)
    }
}

fn json_parse(args: &[Value]) -> Result<Value, String> {
    check_args("json.parse", args, 1)?;
    let text = match &args[0] {
        Value::Str(s) => s.clone(),
        _ => return Err("json.parse requires a string".into()),
    };
    let mut parser = Parser::new(&text);
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.peek().is_some() {
        return Err("trailing characters after JSON value".into());
    }
    Ok(value)
}

fn stringify_value(val: &Value) -> String {
    match val {
        Value::Nothing => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(n) => {
            if n.fract() == 0.0 && n.is_finite() {
                format!("{}.0", n)
            } else {
                n.to_string()
            }
        }
        Value::Str(s) => {
            let escaped: String = s.chars().flat_map(|c| match c {
                '"' => "\\\"".chars().collect(),
                '\\' => "\\\\".chars().collect(),
                '\n' => "\\n".chars().collect(),
                '\r' => "\\r".chars().collect(),
                '\t' => "\\t".chars().collect(),
                other => vec![other],
            }).collect();
            format!("\"{}\"", escaped)
        }
        Value::List(items) => {
            let items: Vec<String> = items.iter().map(stringify_value).collect();
            format!("[{}]", items.join(","))
        }
        Value::Map(map) => {
            let pairs: Vec<String> = map.iter()
                .map(|(k, v)| format!("\"{}\":{}", k, stringify_value(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
        Value::Native { name, .. } => format!("\"<native {}>\"", name),
        Value::Fn { name, .. } => format!("\"<fn {}>\"", name),
        Value::Class { name, .. } => format!("\"<class {}>\"", name),
        Value::Instance { class_name, .. } => format!("\"<instance of {}>\"", class_name),
    }
}

fn json_stringify(args: &[Value]) -> Result<Value, String> {
    check_args("json.stringify", args, 1)?;
    Ok(Value::Str(stringify_value(&args[0])))
}

pub fn make_module() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("parse".into(), make_native("json.parse", json_parse));
    m.insert("stringify".into(), make_native("json.stringify", json_stringify));
    m
}

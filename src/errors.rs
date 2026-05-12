// Errors: unified error types for the elang compiler

const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

fn strip_suffix(msg: &str) -> (String, Option<usize>) {
    if let Some(pos) = msg.rfind(" at line ") {
        let prefix = &msg[..pos];
        let rest = &msg[pos + 9..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(n) = num_str.parse::<usize>() {
            return (prefix.to_string(), Some(n));
        }
    }
    (msg.to_string(), None)
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ElangError {
    LexError(String),
    ParseError(String),
    RuntimeError {
        message: String,
        line: usize,
        stack: Vec<String>,
    },
}

fn kind_name(e: &ElangError) -> &'static str {
    match e {
        ElangError::LexError(_) => "LexError",
        ElangError::ParseError(_) => "ParseError",
        ElangError::RuntimeError { .. } => "RuntimeError",
    }
}

fn msg_with_line(e: &ElangError) -> (String, Option<usize>) {
    match e {
        ElangError::LexError(msg) => strip_suffix(msg),
        ElangError::ParseError(msg) => strip_suffix(msg),
        ElangError::RuntimeError { message, line, .. } => (message.clone(), Some(*line)),
    }
}

impl std::fmt::Display for ElangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = kind_name(self);
        write!(f, "{}[elang error]{}", RED, RESET)?;
        write!(f, " {}{}{}", BOLD, kind, RESET)?;

        match self {
            ElangError::RuntimeError { message, line, stack } => {
                write!(f, " on line {}{}{}", YELLOW, line, RESET)?;
                write!(f, ": {}{}{}", BOLD, message, RESET)?;
                if !stack.is_empty() {
                    writeln!(f)?;
                    writeln!(f)?;
                    write!(f, "{}Stack trace:{}", BOLD, RESET)?;
                    for entry in stack {
                        writeln!(f)?;
                        write!(f, "  in {}", entry)?;
                    }
                }
                Ok(())
            }
            ElangError::LexError(msg) | ElangError::ParseError(msg) => {
                let (message, line_opt) = strip_suffix(msg);
                if let Some(n) = line_opt {
                    write!(f, " on line {}{}{}", YELLOW, n, RESET)?;
                }
                write!(f, ": {}{}{}", BOLD, message, RESET)
            }
        }
    }
}

impl std::error::Error for ElangError {}

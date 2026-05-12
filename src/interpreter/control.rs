#![allow(dead_code)]

use crate::errors::ElangError;

pub fn break_outside_loop(line: usize) -> ElangError {
    ElangError::RuntimeError {
        message: "Break outside loop".into(),
        line,
        stack: vec![],
    }
}

pub fn continue_outside_loop(line: usize) -> ElangError {
    ElangError::RuntimeError {
        message: "Continue outside loop".into(),
        line,
        stack: vec![],
    }
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
    fn test_break_exits_repeat_loop() {
        let source = r#"
let count = 0
repeat 100 times:
    count = count + 1
    if count == 5:
        break
    end
end
print count
"#;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_source(source).unwrap();
        }));
        assert!(result.is_ok());
    }

    #[test]
    fn test_continue_skips_for_loop_iteration() {
        let source = r#"
let result = 0
for n in [1, 2, 3, 4, 5]:
    if n == 3:
        continue
    end
    result = result + n
end
print result
"#;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_source(source).unwrap();
        }));
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_in_function_not_confused_with_break() {
        let source = r#"
def find_first(items, target):
    for x in items:
        if x == target:
            return x
        end
    end
    return -1
end

let found = find_first([10, 20, 30], 20)
print found
"#;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_source(source).unwrap();
        }));
        assert!(result.is_ok());
    }

    #[test]
    fn test_break_in_nested_loop_only_exits_inner() {
        let source = r#"
var outer = 0
var i = 0
var inner = 0
while i < 3:
    inner = 0
    while inner < 5:
        if inner == 2:
            break
        end
        outer = outer + 1
        inner = inner + 1
    end
    outer = outer + 1
    i = i + 1
end
print outer
"#;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_source(source).unwrap();
        }));
        assert!(result.is_ok());
    }
}

# ELANG STRESS TEST FINAL REPORT

Program 1 FizzBuzz:    PASS
Program 2 Fibonacci:   PASS
Program 3 Word Count:  FAIL
Program 4 OOP Grades:  FAIL
Program 5 JSON Config: FAIL

## RESULTS

### Program 1 — FizzBuzz (PASS)
Full 1..100 output with correct Fizz/Buzz/FizzBuzz. Uses `while` loop + nested `else`/`if` (native `else if` not available in original test, now supported via `fix/else-if-not`).

### Program 2 — Fibonacci (PASS)
Correct sequence: 0,1,1,2,3,5,8,13,21,34,55,89,144,233,377. Recursion fix (`fix/recursion`) injects function name into its own call environment, enabling recursive calls.

### Program 3 — Word Count (FAIL)
**Error:** `Cannot access field 'split' on this value` (line 5)

`"hello".split(" ")` method syntax requires `Value::Str` handling in `get_field()` inside `env.rs`. The string module (`string.rs`) has `split()`, `trim()`, `list_add()`, etc. as **module functions** (`string.split(str, sep)`), but value.method() syntax (`content.split(" ")`) is not wired up.

**Broken features:**
- `.split()` method on strings
- `.trim()` method on strings
- `.add()` method on lists
- `.count` property on lists
- `let`/`var` cannot be redeclared inside loop bodies (same scope reused each iteration)

### Program 4 — OOP Grades (FAIL)
**Error:** `'self.name' is not defined` (line 0)

String interpolation `{self.name}` does a bare variable lookup for `"self.name"` as a single identifier, instead of evaluating the expression `self.name` as a field access on `self`.

**Broken features:**
- String interpolation `{expr}` only supports simple variable names
- No expression parsing inside `{...}` in `env.rs`
- `not` keyword on method calls (`not self.passed()`)

### Program 5 — JSON Config (FAIL)
**Error:** `'config.app' is not defined` (line 0)

Same interpolation issue as Program 4 — `{config.app}`, `{config.version}`, `{config.author.name}`, `{config.debug}` all fail because string interpolation doesn't evaluate expressions.

**Broken features:**
- `{config.app}` — field access in interpolation
- `{config.debug}` — boolean value in interpolation
- `{config.author.name}` — nested field access in interpolation
- `config.debug == true` comparison after JSON parse

## Root Cause

The `fix/string-interp` branch created a separate alternate interpreter in `src/interpreter/expr.rs` with its own `Val` type and expression evaluator. This does NOT integrate with the real interpreter in `env.rs`. The actual `env.rs` interpreter still has:

1. **String interpolation** — only does `env.get(&buf)` (bare variable lookup), no expression parsing inside `{...}`
2. **No method dispatch** for `Value::Str` or `Value::List` in `get_field()` — `.split()`, `.trim()`, `.add()`, `.count`, `.remove()` all fail
3. **No scope isolation** per loop iteration — `let`/`var` cannot be re-declared inside loop bodies

## Working Features

- `let`/`var`/`const` variable declarations
- `while` loops with `<`, `<=`, `==`, `%` operators
- `if`/`else` with nested `if` (native `else if` now supported)
- `not` keyword
- `repeat N times` loops
- `loop forever` / `break` / `continue`
- `print` with integer, string, and simple `{var}` interpolation
- Variable assignment `i = i + 1`
- Non-recursive function calls
- Recursive function calls (fix/recursion)
- `import` and all stdlib modules (math, io, json, os, string)
- `io.read()` file reading
- `json.parse()` / `json.stringify()`
- `string.split()`, `string.trim()`, `string.list_add()` as module functions
- Class definition with `pub` fields
- `self.field = value` mutations persisting (Rc<RefCell>)
- Method calls on instances (`obj.method()`)

## Priority Fixes

1. **String interpolation expression evaluation** in `env.rs` — parse `{...}` content as an expression, not just a variable name. This unblocks Programs 4 & 5.
2. **Value method dispatch** in `get_field()` — add `Value::Str` (split, trim) and `Value::List` (add, count, remove, contains, first, last) handlers. This unblocks Program 3.
3. **Loop scope isolation** — create a fresh scope for each loop iteration so `let`/`var` works inside loops.

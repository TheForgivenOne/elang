# ELANG STRESS TEST FINAL REPORT

Program 1 FizzBuzz:    PASS
Program 2 Fibonacci:   FAIL
Program 3 Word Count:  FAIL
Program 4 OOP Grades:  FAIL
Program 5 JSON Config: FAIL

## BROKEN FEATURES

1. **else-if chaining** (Program 1) — `else if` is not supported as a single construct. Requires nested `else: if ... end end`. Worked around with nesting.

2. **Recursive function calls** (Program 2) — `'fibonacci' is not defined` when a function tries to call itself recursively. The function name is not visible in its own closure environment.

3. **String .split() method** (Program 3) — `Cannot access field 'split' on this value`. No built-in `split()` on strings. Requires `string.split()` from stdlib.

4. **String .trim() method** (Program 3) — No `.trim()` on strings. Not implemented.

5. **List .add() method** (Program 3) — No `.add()` on lists. Not implemented.

6. **List .count property** (Program 3) — No `.count` property on lists. Not implemented.

7. **List index expression** (Program 3) — `words[words.count - 1]` would fail since `.count` doesn't exist.

8. **String interpolation field access** (Programs 4, 5) — `{self.name}` and `{config.app}` look up the literal string `"self.name"` / `"config.app"` as a variable name instead of evaluating the expression. String interpolation only supports simple variable names.

9. **Match inside method returning** (Program 4) — Match `is int:` type checking inside a method may have issues.

10. **`not` keyword** (Program 4) — Unknown if supported; `not self.passed()` might fail.

11. **JSON boolean comparison** (Program 5) — `config.debug == true` might fail or work.

## WORKING FEATURES

- `let` variable declarations
- `while` loops with `<`, `<=` operators
- `if/else if/else` (via nested else)
- `<`, `<=`, `==`, `%` operators
- `print` with integer and string values
- `print` with simple string interpolation `{var}`
- `repeat N times` loops
- Variable assignment `i = i + 1`
- Function calls (non-recursive)
- `io.read()` file reading
- `json.parse()` on JSON strings
- `json.stringify()` 
- `import` statements
- Nested `if` inside `else` blocks

## PRIORITY FIXES

1. **String interpolation expression support** — `{expr}` should evaluate any expression, not just variable lookup. Blocks Programs 4 & 5.
2. **Recursive function calls** — Make function name visible in its own closure for recursion.
3. **String `.split()` method** — Required for most real-world programs.
4. **List `.count` property** — Required for accessing list length.
5. **List `.add()` method** — Required for building lists dynamically.
6. **`else if` chaining** — Nested `else if` is a common pattern that should work natively.
7. **String `.trim()` method** — Needed for string processing.

## UNEXPECTED BEHAVIOR

- String interpolation `{var}` works but only for simple variable names — dots cause failures without clear error message about interpolation limitations.
- `else if` on the same line fails; the fix requires understanding elang's parser structure (two `end` keywords needed).
- `repeat` is a keyword so `string.repeat()` becomes `string.repeat_str()`. This is a naming collision in the language design.

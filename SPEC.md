# SPEC: elang

elang is an AI-first systems language. Its source is a dense symbol stream,
compiled to native binaries through LLVM. It is designed for one reader and
one writer — the AI — and every design decision is scored by a single metric:
**characters per program**.

## 1. Objective

Give the AI a language where the correct program is also the shortest
program, and where ambiguity is structurally impossible rather than merely
discouraged.

- **Writer/reader:** the AI. Humans supervise and verify; they do not author.
- **Success for v0.1:** elang source tokenizes and parses into an AST, and a
  minimal program compiles through LLVM to a native x86-64 binary that runs.
- **North-star metric:** the `bench/` suite (FizzBuzz, string transform, list
  ops, JSON-ish parsing, file I/O). Every syntax decision must reduce
  characters-per-program versus the Rust equivalent for the same task.

## 2. Design constraints

1. **Minimize characters written.** Formatting is canonical *and reductive*:
   `elang fmt` only ever removes characters, never adds them.
2. **One spelling per construct.** No aliases, no optional syntax, no
   operator overloading, no user-defined operators, no macros, no traits in
   v0.1. Spelling multiplicity is the failure mode; it is designed against.
3. **Sigils may never appear inside identifiers.** Token boundaries are
   self-evident, which is what makes zero-whitespace packing safe. Any future
   feature that reintroduces word-keywords or context-sensitive lexing is
   rejected — it breaks packing.
4. **Human readability is a non-goal.** Source is allowed to look like a
   binary blob with symbols in it.

## 3. Lexical grammar

- **Identifiers (bindings, functions):** `[a-z_][a-z0-9_]*`
- **Type identifiers:** `[A-Z_][A-Za-z0-9_]*`
- **Integers:** `[0-9][0-9_]*`; width digests `i8 i16 i32 i64 i128 u8 u16
  u32 u64 u128`, default aliases `i` = `i64`, `u` = `u64`, `f` = `f64`,
  `f32`.
- **Strings:** `"..."` with escapes `\n \t \\ \" \0`; UTF-8.
- **Whitespace:** insignificant except that a newline terminates a
  statement. `;` is an accepted equivalent terminator.
- **A numeric literal immediately followed by an identifier character is an
  error** (it would be ambiguous under packing).
- **Comments:** `%` to end of line.
- **Reserved literals:** `t` and `f` are the boolean constants. They are not
  identifiers; naming a binding `t` is illegal.

### 3.1 Sigil table

| Sigil | Role |
|---|---|
| `=` | assignment; declares when the target name is new |
| `==` `!=` `<` `<=` `>` `>=` | comparison |
| `!` | leading: mutable declaration (statement position) / boolean not (expression); trailing: propagate `Result` error |
| `?` | `if` |
| `~` | `else` |
| `@` | `while`; `@name < iterable` is a for-binding loop |
| `<` | loop binding (after `@name`); also less-than |
| `#` | `match` |
| `=>` | match arm |
| `^` | return; bare `^` returns the unit value |
| `%` | comment |
| `\|>` | pipeline: `a \|> f \|> g` is `g(f(a))` |
| `\|` | closure argument list: `\|a,b\| expr` |
| `:` `,` `.` `;` | annotation, separator, member, terminator |
| `( ) { } [ ]` | grouping, block, list/array |
| `&` | immutable reference (`&T`); mutable reference is `&!T` |
| `+ - * /` | arithmetic (fixed precedence, documented once) |
| `\` | explicit line continuation |

Bitwise operators are deliberately omitted; they cost characters and free `|`
for closures and pipelines. Bitwise ops are stdlib functions.

## 4. Core grammar rules

1. Every block is braced; no bare or one-line bodies.
2. A function is `name(params) [rettype] { ... }` — no `fn` keyword. Top
   level `name(...)` followed by a block is a function declaration.
3. Declaration is `name = expr` (`!name = expr` for mutable). Reassignment
   `name = expr` is only legal on a mutable binding.
4. A type declaration is `T Name { fields }`; an enum is `E Name { variants }`;
   an impl block is `# Name { methods }` with `&self` as the receiver.
5. The final expression of a block is its value; `^` returns early.
6. `if`/`match`/blocks are expressions and may appear wherever a value is
   expected.
7. Statements end at a newline or `;`.
8. Type annotations are required on function parameters, struct fields, and
   enum payloads; everything else is inferred.

### 4.1 Minimal example

```elang
T P { x:i, y:i }
# P {
  new(x:i, y:i) -> P { P{x:x, y:y} }
  add(&self, o:&P) -> P { P{x:self.x+o.x, y:self.y+o.y} }
}
E Shape { C(i), R(P, P) }
area(s:&Shape) -> i {
  #s { Shape.C(r)=>r*r*3, Shape.R(a,b)=>(b.x-a.x)*(b.y-a.y) }
}
main() {
  o=P.new(0,0)
  !v=V.new()
  v.push(Shape.C(3))
  v.iter()|s|{ print(area(s)) }
  ?o.x>5 {^1}
  0
}
```

## 5. Memory model (staged)

- **v0.1:** move-by-default ownership, deterministic destruction, no
  garbage collector. Passing a value moves it; `&T` borrows without moving.
  Aliasing through a mutable borrow is *not yet rejected at compile time*.
- **v1.1 gate:** the borrow checker — full Rust-style borrow rules enforced
  at compile time. It is staged because it is the single most complex
  component and it needs real elang code to be checked against v0.1 output.
- **v2:** self-hosting — elang's compiler written in elang, compiled by the
  Go host.

## 6. Builtins (v0.1)

`V<T>` vector, `S` string (owned) and `&S` string slice, `B<T>` box,
`M<K,V>` map, `R<T,E>` result with the trailing-`!` propagate operator,
`print`, `len`, `push`, `map`, `filter`, `fold`, `open`, `read`, `write`.

## 7. Backend and CLI

- Pipeline: elang source → LLVM IR (`.ll`) → `llc` → `as`/`cc` → native
  x86-64 Linux binary.
- CLI: `elang build x.el`, `elang fmt x.el`, `elang ir x.el`, `elang obj
  x.el`, `elang bench`, `elang version`.
- Compiled artifacts and debug dumps are emitted in compact binary form for
  the AI to inspect. Source itself stays dense text: an LLM pays hex/base64
  markup to write bytes into a stream, so binary source would cost more
  characters, not fewer.

## 8. Tech stack and host

- Host compiler: **Go** (fast builds, tiny RAM — this box is a Celeron with
  3.7 GiB).
- Backend: LLVM 19 (`llc`), already installed. No new heavy dependencies.

## 9. Project structure

```
elang/
├── SPEC.md              this file
├── README.md            public overview
├── AGENTS.md            project rules for agent sessions
├── go.mod
├── cmd/elang/           CLI entry point
├── internal/
│   ├── lexer/           sigil-first lexer + tests
│   ├── ast/             AST node types
│   ├── parser/          parser + tests
│   ├── sema/            type/ownership checking (later slices)
│   └── ir/              LLVM IR emission + tests
└── bench/               characters-per-program benchmark suite
```

## 10. Commands

```
Build:  go build ./...
Test:   go test ./...
Fmt:    gofmt -l . ; go vet ./...
Bench:  ./bench/run.sh
```

## 11. Boundaries

- **Always:** tests before commit; `gofmt` clean; one spelling per construct;
  a syntax decision must be scored against `bench/` before it ships.
- **Ask first:** adding a dependency; changing the sigil table; adding a
  builtin; changing the ownership model.
- **Never:** reintroduce word-keywords; add aliases or optional syntax;
  commit secrets; weaken a test to get green.

## 12. Milestones

- **M1 (foundations):** git repo, spec, module layout, lexer with full test
  coverage, CLI skeleton.
- **M2 (front end):** parser producing an AST for the minimal example;
  formatter that reduces characters; error reporting with spans.
- **M3 (back end):** LLVM IR emission; `elang build hello.el` produces a
  native binary that runs. v0.1 ships here.
- **M4 (language depth):** `Vec`/`String`/`Result`, ownership/move
  semantics enforced, structs/enums/methods.
- **M5 (v1.1):** borrow checker.
- **M6 (v2):** self-hosting.

## 13. Open questions

- Exact stdlib spelling for map/filter/fold receivers (`.map(f)` vs
  `map(v,f)`) — decide by character count in `bench/`.
- Whether `elang fmt` is a separate pass or a lexer re-printer.

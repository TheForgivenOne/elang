# elang — Roadmap

## Goal
A self-hosted systems language that unifies concepts from C, Python, Java, JavaScript, Rust, Go, and Haskell — capable of everything existing languages can do.

---

## Stage 1: Bootstrap Compiler (Rust + LLVM)

Replace the tree-walking interpreter with an LLVM codegen backend using `inkwell`.

- [ ] Install LLVM dev libraries and add `inkwell` crate
- [ ] Create `src/codegen/` module — walks AST, emits LLVM IR
- [ ] Implement IR for core types: `int`, `float`, `bool`, `string`, `nothing`
- [ ] Implement IR for expressions: literals, `BinOp`, `UnaryOp`, `Ident`, `Call`
- [ ] Implement IR for statements: `LetDecl`, `Assign`, `FnDef`, `Return`, `If`, `Loop`, `Print`
- [ ] Implement IR for compound types: `list`, `map`, `class`, `instance`
- [ ] Implement IR for features: lambda, string interp, pipe, match, try/catch, `for..in`
- [ ] Add `elang build <file>` CLI command that emits a native binary
- [ ] Link with a minimal runtime (heap alloc, print, stdlib)
- [ ] Enable cross-compilation target flags
- [ ] Remove interpreter code
- [ ] All existing tests pass via compiled binaries

## Stage 2: Self-Hosting

Rewrite the elang compiler in elang itself.

- [ ] Port lexer to elang
- [ ] Port parser to elang
- [ ] Port codegen/IR emission to elang
- [ ] Compile the elang compiler with the Stage 1 bootstrap
- [ ] The elang compiler now compiles itself
- [ ] Drop the Rust bootstrap — only elang remains

## Stage 3: Own the Stack

- [ ] Replace LLVM with our own codegen (optional — own optimizations, smaller binaries)
- [ ] WASM backend for browser/server target
- [ ] Package manager (`elang install`)
- [ ] LSP / editor support
- [ ] FFI to C libraries
- [ ] Full standard library in elang

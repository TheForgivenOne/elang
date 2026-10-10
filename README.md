# elang

**An AI-first systems language.** Dense symbol source, native binaries, one
spelling per construct.

elang is designed for one writer and one reader: the AI. It is not designed
to be pleasant for humans to read — it is designed so that the correct
program is also the shortest program, and so that ambiguous source is
structurally impossible rather than merely discouraged.

## What it looks like

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

No `fn`, `let`, `struct`, `impl`, `return`, or `match` keywords — sigils
carry those roles, so source packs with zero whitespace and stays
unambiguous. `|>` pipelines and minimal closures make one-line
transformations the natural spelling:

```elang
print(v.iter().map(area).filter(|x|x>5).fold(0,|a,b|a+b))
```

## The metric

Every syntax decision is scored by one number: **characters per program**,
against a fixed benchmark suite in [`bench/`](bench/). If a construct does
not reduce characters, it does not ship.

## Status

Early foundations. The lexer is written and tested; the parser and LLVM
backend are next. See [SPEC.md](SPEC.md) for the full design and the
milestone list.

## Building

```
go build ./...
go test ./...
```

## Roadmap

| Milestone | Scope |
|---|---|
| M1 | Repo, spec, module layout, lexer, CLI skeleton |
| M2 | Parser, formatter, span diagnostics |
| M3 | LLVM IR backend — `elang build hello.el` produces a native binary (v0.1) |
| M4 | Vec/String/Result, move semantics enforced |
| M5 | Borrow checker (v1.1) |
| M6 | Self-hosting (v2) |

## License

MIT

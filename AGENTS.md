# AGENTS.md — elang project rules

These apply to every agent session in this repository, in addition to the
global rules in ~/.config/opencode/AGENTS.md.

## The one metric

**Characters per program** on the `bench/` suite. Every syntax decision,
builtin name, and grammar rule must be scored against it before shipping.
If a construct does not reduce characters, it does not ship.

## Non-negotiables

- **One spelling per construct.** No aliases, no optional syntax, no
  operator overloading, no user-defined operators.
- **Sigils never appear inside identifiers.** Token boundaries must be
  self-evident, which is what makes zero-whitespace packing safe. Never add
  a word-keyword or a context-sensitive lexing rule — it breaks packing.
- **Human readability is a non-goal.** Do not propose syntax changes whose
  justification is human comfort.
- **Tests before commit.** Every lexer/parser/sema change ships with tests
  in the same commit; `go test ./...` must be green before pushing.

## Toolchain

- Host compiler: Go 1.27 (single static binary, tiny RAM).
- Backend: LLVM 19 via `llc` (already installed). Do not add build
  dependencies to go.mod without asking.
- Platform target: x86-64 Linux.

## Workflow

- Work in small vertical slices; each commit leaves `go build ./...` and
  `go test ./...` green.
- Reference milestone numbers (M1–M6 from SPEC.md) in commit messages.
- Run `gofmt -l .` and `go vet ./...` before committing.

## Boundaries

- Always: tests green, gofmt clean, score syntax changes against bench/.
- Ask first: adding a dependency, changing the sigil table, adding a
  builtin, changing the ownership model.
- Never: commit secrets, weaken a test to get green, delete the spec to
  "simplify".

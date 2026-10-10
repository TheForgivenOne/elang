# elang benchmark suite

The north-star metric is characters per program. Each case is the same task
written twice: `*.rs` (idiomatic Rust) and `*.el` (target elang). `run.sh`
reports the ratio and the absolute character counts.

The ratio is the score. A change to the language that makes the ratio worse
does not ship.

Rust counts include idiomatic formatting (rustfmt); elang counts include no
whitespace beyond newline terminators.

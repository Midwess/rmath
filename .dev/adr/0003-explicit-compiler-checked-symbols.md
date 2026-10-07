# ADR-0003: Symbols are declared explicitly and resolved by the Rust compiler

**Date:** 2026-10-07 · **Status:** accepted

## Context

Inside `expr!(x^2 + f(y))`, what is `x`? Symbolica's `parse!` auto-creates a symbol for every
unknown name. That is convenient but a typo (`xx`) silently becomes a new variable, and Rust
values cannot be mixed in without a marker.

## Decision

Every identifier in a macro body is re-emitted as the same Rust identifier at the call site.
Therefore:

- Symbols must exist as Rust bindings, created by `symbols!(x, y: Real, f: Symmetric)`.
- Built-in functions and constants (`sin`, `cos`, `exp`, `log`, `sqrt`, `abs`, `I`) are
  ordinary items in `rmath::prelude`, not macro keywords.
- Any other Rust value in scope (`k`, a loop counter, an existing `Atom`) is spliced through
  the `IntoExpr` trait with no marker.
- Wildcards in `rule!`/`find!` are the one exception: a trailing underscore declares them
  implicitly, mirroring how `match` arms introduce bindings.

## Consequences

- Typos are compile errors (`E0425`), with rustc's "similar name exists" suggestions for free.
- IDE features (go-to-definition, rename, autocomplete) work on symbols and built-ins.
- Symbol attributes live at the declaration, visible to readers.
- Cost: a `symbols!` line before use. Judged acceptable for a library whose point is
  compiler-checked math.
- A local variable named `sin` shadows the built-in. Documented; same rule as Rust.
- Visible API choice, hard to change later without breaking every user: accepted.

## Alternatives rejected

- *Implicit auto-creation (parse!-style):* zero setup but silent typos; Rust splices need a
  sigil (`#k`).
- *Explicit with `@name` escape for ad-hoc symbols:* adds a second way to do the same thing;
  can be added later without breaking anything if demand appears.

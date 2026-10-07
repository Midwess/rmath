# Proposal: rmath v1 — native math-syntax macros over Symbolica

**Status**: approved

## Summary

Turn the `rmath` stub into a public library that lets Rust users write computer-algebra
expressions as math (`expr!(x^2 + sin(y)/2)`), checked by the compiler and expanded at compile
time into calls on Symbolica's public API. v1 ships `symbols!`, `expr!`, `rule!`, `find!`,
`solve!` and `func!` on top of a shared parser, in four phases within this single proposal.

## Motivation

Symbolica is fast and complete, but writing expressions from Rust today means either:

- `parse!("x^2 + sin(y)")`: a string parsed at runtime. Typos surface only when the code runs,
  and splicing Rust values (`k`, a loop counter, an existing `Atom`) means formatting strings.
- Builder calls / operator overloading: correct but verbose, and `^` cannot be used because
  Rust's `^` is XOR with the wrong precedence.

rmath removes both costs: math syntax, compile-time name checking (an undeclared symbol is an
ordinary `E0425`), free splicing of Rust values, zero runtime parsing, and the result is still a
plain `symbolica::atom::Atom`, so the full Symbolica API remains available.

## Scope

### In Scope

- Cargo workspace: `rmath` (runtime crate) + `rmath-macros` (proc-macro crate)
- Shared Pratt parser over `proc_macro2` tokens with span-accurate diagnostics
- Grammar: `+ - * / ^` (right-assoc, tighter than unary minus), unary minus, postfix `!`,
  calls, parentheses, integer and decimal literals, identifiers, `{ rust }` splice blocks
- `symbols!(x, y: Real + Positive, f: Symmetric, tau0 = "τ_0")`
- `expr!(...)` → `Atom`; `IntoExpr` trait; prelude built-ins `sin cos exp log sqrt abs I`
- `rule!` (single rule, rule set, `if` condition), `.apply()` extension trait, `find!` with
  per-call-site typed match struct
- `solve!([lhs == rhs, ...] for vars [over Domain])` → `Result<SolutionSet, SolveError>`
- `func!(|params| body)` → `Result<impl FnMut(f64, ...) -> f64, EvaluationError>`
- Tests: parser unit tests, equivalence tests against `symbolica::parse!`, `trybuild`
  compile-fail tests, doc examples
- Distribution compliance: `LICENSE-SYMBOLICA.md`, README notice that Symbolica runtime
  rights are not included, rmath's own license (assumed MIT OR Apache-2.0; confirm)
- ADRs 0001–0003 (see `.dev/adr/`)

### Out of Scope

- Re-implementing any CAS algorithm (see ADR-0001); rmath only calls Symbolica
- An `rmath::Error` wrapper; Symbolica's error types are returned directly
- Implicit multiplication, Unicode operators (`×`, `²`), LaTeX input
- `func!` outputs other than `f64` (complex, arbitrary precision, SIMD), C++/JIT export
- Named constants other than `I` (e.g. `pi`, `e`): follow-up once the Symbolica API is checked
- Python bindings; the crates.io release itself (separate step after v1 is green)

## Affected Areas

| Area | Impact |
|------|--------|
| `Cargo.toml` (root) | Becomes workspace root + `rmath` package; adds `symbolica`, `rmath-macros` deps and feature passthrough |
| `src/lib.rs` | Replace stub with re-exports (`symbolica`, macros, prelude), `IntoExpr`, `Callable`, `__private` |
| `src/into_expr.rs`, `src/call.rs`, `src/prelude.rs`, `src/apply.rs`, `src/__private.rs` | New runtime modules |
| `rmath-macros/` | New proc-macro crate: `lib.rs`, `parse/`, `expand/` |
| `tests/` | New: equivalence tests, `trybuild` UI tests under `tests/ui/` |
| `README.md`, `LICENSE-SYMBOLICA.md`, `LICENSE-*` | New: usage docs and license compliance |
| `.gitignore` | Narrow `.dev` to `.dev/symbolica/` so specs can be committed (Symbolica clone must never be committed) |

## Dependencies

- Rust ≥ 1.96 (Symbolica's MSRV), edition 2024
- `symbolica = "3.0"` (public API only; see ADR-0001)
- `proc-macro2 = "1"`, `quote = "1"`, `syn = "2"` (only for closure/attribute parsing in
  `func!`/`symbols!`; expression grammar is hand-written)
- Dev: `trybuild = "1"`
- A Symbolica runtime license for running tests locally (free tier runs on one core)

## Risks

| Risk | Mitigation |
|------|------------|
| Symbolica semver churn breaks generated code | Depend on `3.0`; generated code only touches `$crate::__private` helpers, so API shims live in one place; CI builds against the latest 3.x |
| User renames the crate (`rm = { package = "rmath" }`) | Public macros are `macro_rules!` shims forwarding `$crate` to the proc macros; CI fixture crate depends on rmath under another name |
| Users expect `-x^2` or `2^3^2` to mean something else | Document precedence table in README; `trybuild` tests pin behaviour; equivalence tests against Symbolica's parser |
| Symbolica default features set a global allocator (mimalloc) and need GMP | Pass features through (`default`, `pure-rust`); verify exact feature names on docs.rs at implementation |
| Unverified Symbolica APIs (MatchStack access, factorial, `Solution` accessor, multi-wildcard → Atom) | Explicit "verify on docs.rs" tasks at the start of each phase; fallback designs listed in `design.md` |
| License breach by accident (AI analysis of `.dev/symbolica`) | ADR-0001 working rule; memory note; `.gitignore` keeps the clone out of git |
| Proc-macro compile-time cost | Hand-written parser, `syn` only where needed, no `syn/full` feature |
| `find!` struct per call site leaks into user scope | Define struct inside the macro's block; expose only via `impl Iterator` |

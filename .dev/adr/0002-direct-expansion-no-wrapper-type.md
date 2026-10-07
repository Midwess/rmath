# ADR-0002: Macros expand directly to Symbolica calls; no `rmath::Expr` wrapper

**Date:** 2026-10-07 · **Status:** accepted

## Context

`expr!(...)` must produce a value. Three options were weighed:

1. **Direct expansion:** emit Symbolica public-API calls; the result is Symbolica's `Atom`.
2. **String + runtime parse:** emit a string for `symbolica::parse!` plus a binding table.
3. **Own AST, lowered at runtime:** emit an rmath tree type, convert to `Atom` when needed.

Independently: should the result be a new `rmath::Expr` newtype around `Atom`?

## Decision

Option 1, with no wrapper type. `expr!` and friends return `symbolica::atom::Atom` (or
Symbolica's `Replacement`/`SolutionSet`/evaluator types, lightly wrapped only where an
rmath-specific trait needs a local type, e.g. `Rule`). Generated code references only
`$crate::__private::*` helpers (the crate root is forwarded by a `macro_rules!` shim).

## Consequences

- Zero runtime parsing; splicing Rust values is a trait call (`IntoExpr`), not string
  formatting.
- The entire Symbolica API stays available on results (`.expand()`, `.derivative(x)`, …);
  rmath does not have to mirror it.
- Symbolica API changes surface at rmath's compile time inside `__private`, one place to fix.
- Error types are Symbolica's; rmath adds no `Error` enum in v1.
- Hard to reverse: once published, users' code holds `Atom`s everywhere; introducing a
  wrapper later would be a breaking change. Accepted.
- The macro must know Symbolica's call surface (`pow`, `sin`, `FunctionBuilder`, …). That is
  public API knowledge, compatible with ADR-0001.

## Alternatives rejected

- *String + runtime parse:* simplest macro, but pays parse cost per evaluation site, makes
  splicing awkward, and loses compile-time arity checks for built-ins.
- *Own AST:* decouples from Symbolica and enables richer diagnostics, but duplicates a tree
  type, costs a conversion per use, and invites reimplementing algebra on the tree, which
  ADR-0001 forbids.
- *`rmath::Expr` newtype:* would let rmath add inherent methods, but hides Symbolica's API
  behind delegation boilerplate and creates two expression types in user code.

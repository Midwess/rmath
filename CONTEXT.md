# rmath Vocabulary

Shared terms for specs, proposals and code. Use these words exactly; when a new term
appears in discussion, define it here first.

Architecture decisions that explain *why* these terms are shaped this way live in `.dev/adr/`
(ADR-0001 license boundary, ADR-0002 no wrapper type, ADR-0003 explicit symbols).

| Term | Definition |
|------|------------|
| **Symbolica** | The upstream computer algebra crate (`symbolica` on crates.io, v3.x) that rmath depends on. Source-available, not open source. rmath only ever calls its *public API*. |
| **Expression** | A value of type `symbolica::atom::Atom`. rmath defines no wrapper type; every macro that produces math returns a plain `Atom`. |
| **Symbol** | A named variable or function head, of type `symbolica::atom::Symbol`. Created with `symbols!`. Globally interned by name: two `symbols!(x)` in different functions yield the same symbol. |
| **Attribute** | A property attached to a symbol at declaration (`Real`, `Positive`, `Symmetric`, `Linear`, …), written `name: A + B` inside `symbols!`. Maps one-to-one to Symbolica's `SymbolAttribute`. |
| **Built-in** | A math function or constant provided by rmath's prelude as an ordinary Rust item (`sin`, `cos`, `exp`, `log`, `sqrt`, `abs`, `I`). Needs no declaration. Not a hard-coded macro keyword: the compiler resolves it by name. |
| **Splice** | Using an existing Rust value (`k`, `a`, a loop variable) inside a macro body. Any identifier that is not a built-in is a splice; it must implement `IntoExpr`. |
| **`IntoExpr`** | rmath trait converting a Rust value to an `Atom`: implemented for `Symbol`, `Atom`, `&Atom`, integer types, `f64`, and Symbolica's `Rational`. |
| **Wildcard** | A pattern variable in `rule!` / `find!`, declared implicitly by a trailing underscore. `a_` matches one argument, `a__` one or more, `a___` zero or more. Not a symbol the user must declare. |
| **Rule** | The `rmath::Rule` type: one or more rewrites `lhs => rhs` (optionally `, if guard`) produced by `rule!`; wraps Symbolica `Replacement`s. Applied with `.apply(&rule)` from the `ApplyRule` trait. |
| **Rule set** | A `Rule` holding several rewrites, written `rule! { … }` like `match` arms and applied in one simultaneous pass. Not a separate type. |
| **Guard** | The `if cond` clause of a rule: a Rust `bool` expression over the rule's wildcards (bound as `&Atom`). Guards with a top-level comma must be parenthesised. |
| **Equation** | `lhs == rhs` inside `solve!`. Lowered to `lhs - rhs` for Symbolica's solver. |
| **Compiled function** | The closure returned by `func!(\|params\| body)`: a Symbolica evaluator wrapped so it is called like `f(1.0, 2.0)`. Captured splices are frozen at creation time. |
| **Expansion** | What a macro emits: Rust code calling `$crate::__private::*` helpers (full paths, crate root forwarded by a `macro_rules!` shim) which in turn call Symbolica's public API. Never a string parsed at runtime, never a `::symbolica::` path. |
| **Grammar** | The math-syntax accepted inside `expr!`-family macros: `+ - * / ^`, unary minus, postfix `!`, parentheses, calls `f(a, b)`, integer / decimal literals, identifiers. `^` is right-associative and binds tighter than unary minus. No implicit multiplication. |
| **Runtime license** | Symbolica's per-user license required to *execute* Symbolica. rmath's distribution does not include or transfer it; rmath's README must say so. |

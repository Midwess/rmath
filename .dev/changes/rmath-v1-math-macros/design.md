# Design: rmath-v1-math-macros

## Overview

rmath is a thin, compile-time layer: proc macros parse math syntax into a short-lived AST, then
emit Rust code that calls Symbolica's public API through `$crate::__private` helpers. Nothing
mathematical happens in rmath; every operation is delegated to Symbolica.

## Architecture

```
user crate ──expr!(k*x^2 + sin(y))──▶ rmath-macros (compile time)
                                        │  parse/   tokens → AST (Pratt)
                                        │  expand/  AST → TokenStream
                                        ▼
            // via a `macro_rules!` shim that forwards `$crate` (R = $crate::__private)
            R::add(R::mul(R::into_expr(&k), R::pow(R::into_expr(&x), R::into_expr(&2i64))),
                   R::call(&sin, [R::into_expr(&y)]))
                                        │
                                        ▼  (run time)
                       rmath runtime: IntoExpr, Callable, prelude, apply, solve/func helpers
                                        │
                                        ▼
                       symbolica public API  →  Atom / Replacement / SolutionSet / Evaluator
```

Workspace layout:

```
rmath/                      [workspace] + [package] rmath
├── src/lib.rs              re-exports: symbolica, macros, prelude; pub mod __private
├── src/macros.rs           #[macro_export] shims: expr! → $crate::__private::__expr!{ $crate; ... }
├── src/into_expr.rs        trait IntoExpr { fn to_expr(&self) -> Atom }
├── src/call.rs             trait Callable<const N>, impl for Symbol + built-ins
├── src/prelude.rs          sin cos exp log sqrt abs I, macros, ApplyRule, Rule, Symbol, Atom, AtomCore
├── src/rule.rs             Rule(Vec<Replacement>), trait ApplyRule, guard(), find_all()   (Phase 2)
├── src/solve.rs            solve() helper                                                (Phase 3)
├── src/func.rs             compile_f64() helper                                          (Phase 4)
├── src/__private.rs        #[doc(hidden)] fns called by expansions
├── tests/                  equivalence.rs, ui.rs (+ tests/ui/*.rs, *.stderr)
└── rmath-macros/
    ├── src/lib.rs          #[proc_macro] expr, symbols, rule, find, solve, func
    ├── src/parse/{mod,ast,pratt,error}.rs
    └── src/expand/{expr,symbols,rule,find,solve,func}.rs
```

## Key Decisions

### Decision 1: Grammar and precedence

**Context:** Rust's own precedence is wrong for math (`^` is XOR, binds looser than `+`), so
a `macro_rules!`/`syn::Expr`-based approach cannot work. A custom parser is needed.

**Decision:** Pratt parser over `proc_macro2::TokenTree`s. Precedence, loosest to tightest:

| Level | Operators | Assoc |
|------:|-----------|-------|
| 1 | `+` `-` (binary) | left |
| 2 | `*` `/` | left |
| 3 | unary `-` | prefix |
| 4 | `^` | right |
| 5 | postfix `!` | postfix |
| 6 | call `f(...)`, parentheses, literals, identifiers, `{ rust }` | — |

Consequences: `-x^2` = `-(x^2)`; `2^3^2` = `2^9`; `x^-1` allowed (unary minus in exponent
position parses as a prefix at level 3 inside the exponent operand); `n!^2` = `(n!)^2`;
`2x` is an error with help "write `2*x`"; `f!(x)` is an error ("macro-call syntax is not an
expression; use `{ f!(x) }` to splice Rust code").

Literals: integer → `into_expr(&<int>i64)`, or `i128` when it does not fit `i64`; beyond
`i128` is a compile error. User-written suffixes (`2u8`) are preserved. Decimal `0.5` →
`float(0.5)` (float coefficient); exact fractions are written `1/2` and normalised by
Symbolica.

Factorial: `n!` lowers to `factorial(atom)` in `__private`. Verified in 1.3: Symbolica has
`Integer::factorial(u32)` and the `gamma` function but no symbolic factorial. So `factorial`
returns the exact `Integer::factorial(n)` when the argument is a non-negative integer that
fits `u32`, and `gamma(a + 1)` otherwise (standard identity; evaluable inside `func!`).
Negative integers and integers above `u32::MAX` also take the `gamma` path.

### Decision 2: Name resolution — explicit, compiler-checked (ADR-0003)

Identifiers are re-emitted verbatim with their original spans, so they resolve to the user's
locals and diagnostics underline the exact token. Internal temporaries use
`Span::mixed_site()` so they can never collide with user names. `symbols!` introduces `let` bindings of type `Symbol`. Built-ins are prelude items.
Unknown names are plain `E0425` errors. Shadowing a built-in with a local named `sin` is allowed
and behaves as Rust shadowing would.

### Decision 3: Expansion target — direct Symbolica calls, no wrapper (ADR-0002)

`expr!` returns `symbolica::atom::Atom`. Each public macro is a `macro_rules!` shim
(`($($t:tt)*) => { $crate::__private::__expr!{ $crate; $($t)* } }`) so the proc macro receives
the crate root and emits full paths `#root::__private::add(..)`, never `::symbolica::*` and
never a glob import (which could shadow user locals named `add`, `pow`, …). Users need only
`rmath` in `Cargo.toml`, renaming the dependency works, and API shims are centralised.

### Decision 4: Function-call dispatch

```rust
#[diagnostic::on_unimplemented(message = "`{Self}` cannot be called with {N} argument(s)")]
pub trait Callable<const N: usize> { fn call(&self, args: [Atom; N]) -> Atom; }
impl<const N: usize> Callable<N> for Symbol { /* FunctionBuilder::new(*self).add_args(args).finish() */ }
pub struct Sin;  impl Callable<1> for Sin { fn call(&self, [x]: [Atom; 1]) -> Atom { x.sin() } }
#[allow(non_upper_case_globals)] pub const sin: Sin = Sin;
```
`f(a, b)` expands to `call(&f, [into_expr(&a), into_expr(&b)])`. Symbols accept any arity;
built-ins are arity-checked at compile time with a readable message.

### Decision 5: Rust splices

Any identifier that is not a call head is `into_expr(&ident)`; the trait method is
`IntoExpr::to_expr(&self) -> Atom` with a blanket impl for `&T`, so splicing never moves the
value (`a` and `b` remain usable after `expr!(a + b)`; one clone per `Atom` splice). Arbitrary
Rust expressions are spliced with braces: `expr!(x + {k + 1})`. `IntoExpr` is implemented for
`Symbol`, `Atom`, `i8`–`i128`, `u8`–`u64`, `f64`, `symbolica::domains::rational::Rational`
and the `I` marker (`i128`/`Rational` subject to `Into<Coefficient>` verification in 1.3).

### Decision 6: `rule!` and `find!`

- Wildcards: identifiers ending in `_`, `__`, `___` expand to `$crate::__private::symbol!("a_")`
  (Symbolica's wildcard naming convention) at the call site, so namespaces match the user's.
- One type for both forms: `pub struct Rule(Vec<Replacement>)`. `rule!(lhs => rhs)` holds one
  replacement, `rule! { r1, r2 }` holds several; both are applied in a single pass through
  `replace_multiple`.
- `, if <cond>` → `.when(guard([a_, b_], move |[a_, b_]| { cond }))`, where `guard` builds a
  `PatternRestriction::MatchStack` that stays inconclusive until all named wildcards are bound.
  The condition is an arbitrary Rust `bool` expression in which each wildcard name is bound to
  `&Atom`. Guard tokens are taken verbatim up to the next top-level `,`, so a guard containing
  a top-level comma (closure, turbofish) must be parenthesised. Comparison sugar (`a_ > 0`) is
  **not** special-cased in v1.
- `find!(e, pat)` expands to a block that defines `#[derive(Debug, Clone)] struct __Match {
  pub a_: Atom, ... }`, collects matches eagerly via `find_all` (the `Pattern` is local to the
  block, so a borrowing iterator cannot escape), and returns `impl Iterator<Item = __Match>`.
  Fields keep the wildcard name (including the underscore) so the code reads as the pattern
  does.
- `ApplyRule` extension trait on all `AtomCore` types: `fn apply(&self, rule: &Rule) -> Atom`.
  (Named `ApplyRule` to avoid a clash should `AtomCore` ever gain `apply`; verified in 1.3.)

Fallbacks if verification fails: if `MatchStack` has no public accessor for bindings,
conditions use `WildcardRestriction::filter` per wildcard and only single-wildcard
conditions are supported in v1 (documented limitation).

### Decision 7: `solve!`

`solve!([a == b, c == d] for x, y over Reals)` → `$crate::__private::solve(&[expr!(a - (b)),
expr!(c - (d))], &[x, y], SolveDomain::Reals)` → `Atom::solve(eqs).over(domain).wrt(vars)`.
Default domain: Symbolica's default (`Complexes`). Returns `Result<SolutionSet, SolveError>`
unchanged. A convenience accessor for "value of variable in branch i" is added only if
`Solution`'s public API supports it cleanly (verify); otherwise users call `as_point_dict()`.

### Decision 8: `func!`

```rust
// func!(|x, y| x^2 + sin(y))
{
    let __body: Atom = expr!(x^2 + sin(y));          // x, y are Symbols in scope
    let __params = [into_expr(x), into_expr(y)];
    $crate::__private::compile_f64(&__body, &__params)   // Result<ExpressionEvaluator<f64>, EvaluationError>
        .map(|mut ev| move |x: f64, y: f64| -> f64 { ev.evaluate_single(&[x, y]) })
}
```
`compile_f64` = `body.evaluator(params).build()?.map_coeff(&|c| c.re.to_f64())`. The closure
is `FnMut` because the evaluator takes `&mut self`. Closure parameter list is generated with
exactly N `f64` parameters. Symbolica's optimisation settings (Horner, CPE iterations) use
defaults in v1.

### Decision 9: Features and allocator passthrough

Symbolica's default features enable a mimalloc global allocator and the GMP backend. rmath
exposes `default = ["gmp"]`, `gmp`, `pure-rust`, `faster-alloc`, each forwarding to the
corresponding Symbolica features, with `default-features = false` on the dependency. Exact
feature names are verified on docs.rs/crates.io during Phase 1.

### Decision 10: Error reporting

Parser errors are emitted as `compile_error!` at the offending token's span with a one-line
message and, where useful, a `help:` sentence. Each error class has a `trybuild` test pinning
the exact message.

## Data Model

Compile-time only AST (lives in `rmath-macros`, never in user code):

```rust
pub enum Expr {
    Int { value: i64, span: Span },
    Float { value: f64, span: Span },           // kept as literal text for exact re-emission
    Ident(Ident),                               // symbol, splice or built-in — resolved by rustc
    Splice(Group),                              // `{ rust }`
    Unary { op: UnOp, expr: Box<Expr>, span: Span },     // Neg, Factorial
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>, span: Span }, // Add Sub Mul Div Pow
    Call { head: Ident, args: Vec<Expr>, span: Span },
}
```

## API Changes

Public surface of `rmath` as shipped in v1 (owner review, task 5.4):

- Macros: `expr!`, `symbols!`, `rule!`, `find!`, `solve!`, `func!` (`macro_rules!` shims)
- `pub use symbolica;`
- `prelude::*`: the six macros, `Atom`, `AtomCore`, `Symbol`, built-ins (`sin cos exp log sqrt
  abs`, `I`), `IntoExpr`, `Callable`, `ApplyRule`, `Rule`, `SolutionExt`
- Traits: `IntoExpr` (`to_expr(&self)`), `Callable<const N>` (method `invoke`, not `call`:
  Symbolica's `Symbol` has an inherent `call`), `ApplyRule` (`apply(&self, &Rule)`),
  `SolutionExt` (`value(&self, Symbol) -> Option<&Atom>`; added during implementation)
- Types: `Rule`, built-in markers (`Sin`, `Cos`, `Exp`, `Log`, `Sqrt`, `Abs`), `I`
- `#[doc(hidden)] pub mod __private` (unstable, macro use only)

Behavioural notes a reviewer should know:
- `rule!` guards bind wildcards as owned `Atom`s (not `&Atom`), so `a_ != expr!(1)` works.
- `func!` parameters must be `Symbol`s (checked at compile time via `__private::param`); the
  closure is `FnMut` and returns `f64`; `func!` returns `Result<_, EvaluationError>`.
- `solve!` passes Symbolica's `Result` through; inexact (`f64`) coefficients yield
  `Err(SolveError::UnsupportedProblem(..))`.
- `find!` collects matches eagerly and returns `impl Iterator` over a hygienic per-site struct.
- Tests must run single-threaded without a Symbolica license (`.cargo/config.toml`).

## Security Considerations

None beyond supply chain: rmath executes no untrusted input at compile time (the parser
only reads the user's own tokens), and performs no I/O. License compliance is a legal, not
security, concern and is covered by ADR-0001 and the distribution spec.

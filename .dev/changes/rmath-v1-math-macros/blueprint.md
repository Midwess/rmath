# Architecture Blueprint: rmath-v1-math-macros

Source: `code-architect` agent (2026-10-07), reviewed and merged with the verified Symbolica API
notes below. The agent read only `Cargo.toml`, `src/lib.rs`, `.dev/project.md`, `CONTEXT.md`;
`.dev/symbolica/` was never opened (ADR-0001).

## Design Summary

Two crates. **`rmath-macros`** parses math tokens with a Pratt parser into a span-carrying AST
and lowers it to calls on one-line helpers in `rmath::__private`. **`rmath`** owns those
helpers, the `IntoExpr` / `Callable` traits, the built-ins, `Rule`, and thin `macro_rules!`
shims that pass `$crate` to the proc macros. Every Symbolica call lives in `rmath/src/`, so an
upstream change is fixed in one place.

## Design Refinements (on top of design.md decisions 1–10)

| Topic | Chosen | Rationale |
|---|---|---|
| Crate path in generated code | `macro_rules!` shim: `($($t:tt)*) => { $crate::__private::__expr!{ $crate; $($t)* } }`; proc macro receives the root tokens before the first `;` | Survives renamed dependencies and re-exports; no `proc-macro-crate` dependency |
| Helper references | Full paths `#root::__private::add(..)` | A glob import inside the block would shadow user locals named `add`, `pow`, `call`… |
| Spans | User `Ident`s re-emitted unchanged; internal temporaries use `Span::mixed_site()` | Errors land on the exact token; shim hygiene cannot hide `symbols!` bindings |
| `IntoExpr` receiver | `fn to_expr(&self) -> Atom`; blanket impl for `&T`; splices lowered as `into_expr(&k)` | `expr!(k * x)` must not move `k`; covers `&&Atom` |
| Integer literals | `i64` if it fits, else `i128`, else compile error; user suffixes preserved | No runtime parsing, no overflow lint surprises |
| Literal parsing | `syn` with `default-features = false`, features `parsing`, `proc-macro`, `printing` (no `full`) | Correct handling of `1_000`, `0x1F`, `2i8` at small compile cost |
| `find!` result | Eager `Vec<[Atom; N]>` mapped into the per-site struct, returned as `impl Iterator` | The `Pattern` is local to the macro block; a borrowing iterator could not be returned |
| Rule type | `pub struct Rule(Vec<Replacement>)` for single rule and rule set alike | Both apply through `replace_multiple`; one `ApplyRule::apply` method |
| `func!` parameters | Must be `Symbol`s already in scope | Consistent with ADR-0003; keeps attributes from the declaration |
| Rule guard tokens | Taken verbatim up to the next top-level `,` | Avoids `syn/full`; guards containing top-level commas must be parenthesised (documented) |
| Symbolica version | `symbolica = "3.0"` (caret), CI also builds against latest 3.x | A tilde pin would risk two Symbolica copies (type mismatch) when a user also depends on it directly |

## Component Design

### `rmath-macros/src/lib.rs`

```rust
#[doc(hidden)] #[proc_macro] pub fn __expr(ts: TokenStream) -> TokenStream { run(ts, expand::expr::expand) }
// __symbols, __rule, __find, __solve, __func: same shape
fn run(ts: TokenStream, f: fn(&Root, TokenStream2) -> Result<TokenStream2, Errors>) -> TokenStream;
pub(crate) struct Root(TokenStream2); // tokens before the first top-level `;` (the `$crate` path)
```
Macros never panic; every failure becomes `Errors::to_compile_error()`.

### `rmath-macros/src/parse/`

```rust
// cursor.rs
pub struct Cursor { toks: Vec<TokenTree>, pos: usize, last: Span }
impl Cursor {
    fn peek(&self) -> Option<&TokenTree>;
    fn bump(&mut self) -> Option<TokenTree>;
    fn peek_op(&self, op: &str) -> bool;                       // joint-aware: "==", "=>", "||"
    fn split_top_level(ts: TokenStream2, sep: char) -> Vec<TokenStream2>;
}
// lit.rs
pub enum Num { I64(i64), I128(i128), Suffixed(Literal), Float(Literal) }
pub fn classify(lit: &Literal) -> Result<Num, Error>;       // `2x` arrives as a suffixed literal → implicit-mult error
// expr.rs
pub enum Mode { Expr, Pattern }
pub fn parse_expr(c: &mut Cursor, mode: Mode) -> Result<Expr, Error>;
fn expr_bp(c: &mut Cursor, min_bp: u8, mode: Mode) -> Result<Expr, Error>;
// symbols.rs / rule.rs / find.rs / solve.rs / func.rs
pub struct SymbolDecl { ident: Ident, attrs: Vec<Ident>, name: Option<Literal> }  // x | y: Real + Positive | tau0 = "τ_0"
pub struct RuleArm   { lhs: Expr, rhs: Expr, guard: Option<TokenStream2>, wildcards: Vec<Ident> }
pub struct FindInput { target: TokenStream2, pattern: Expr }
pub struct SolveInput { eqs: Vec<(Expr, Expr)>, vars: Vec<Ident>, domain: Option<Ident> }
pub struct FuncInput { params: Vec<Ident>, body: Expr }
```

**Binding powers** (matklad style, left/right): `+ -` (1, 2) · `* /` (3, 4) · prefix `-` right 5 ·
`^` (8, 7) → right-associative · postfix `!` left 9. Prefix `-` is allowed at the start of any
operand, so `-x^2` = `-(x^2)`, `2^3^2` = `2^9`, `2^-x*3` = `(2^(-x))*3`.

**Expression terminators:** `,` `==` `=>` `;`, the identifiers `for` `if` `over`, end of input.

**Primaries:** literal · identifier · `ident(args)` · `(expr)` · `{ … }` splice · an invisible-
delimiter group (from a caller's `$e:expr`), treated as a splice.

**Wildcards (Pattern mode):** identifier with a non-underscore prefix and 1–3 trailing
underscores.

### `rmath-macros/src/expand/`

```rust
pub fn expand(root: &Root, input: TokenStream2) -> Result<TokenStream2, Errors>;  // one per module
pub(crate) fn lower(root: &Root, e: &Expr) -> TokenStream2;                      // expr.rs, shared
```
Lowering (all paths prefixed `#root::__private::`): `Num` → `into_expr(&2i64)` / `float(0.5)`;
`Ident` → `into_expr(&x)` via `quote_spanned!` at the identifier; `Splice` → `into_expr(&{..})`;
`Binary` → `add` `sub` `mul` `div` `pow`; `Neg` → `neg`; `Factorial` → `factorial`;
`Call` → `call(&f, [args..])`.

### `rmath` runtime

```rust
// into_expr.rs
#[diagnostic::on_unimplemented(message = "`{Self}` cannot be used in a math expression",
    note = "implement `rmath::IntoExpr`, or splice a block returning `Atom`: `{ ... }`")]
pub trait IntoExpr { fn to_expr(&self) -> Atom; }
impl<T: IntoExpr + ?Sized> IntoExpr for &T { .. }  // + Symbol, Atom, i8..i128, u8..u64, f64, Rational, I

// call.rs
#[diagnostic::on_unimplemented(message = "`{Self}` cannot be called with {N} argument(s)")]
pub trait Callable<const N: usize> { fn call(&self, args: [Atom; N]) -> Atom; }
impl<const N: usize> Callable<N> for Symbol { /* FunctionBuilder::new(*self).add_args(args).finish() */ }

// prelude.rs
pub struct Sin; impl Callable<1> for Sin { /* let [a] = args; a.sin() */ }
#[allow(non_upper_case_globals)] pub const sin: Sin = Sin;   // cos exp log sqrt abs: same pattern
pub struct I;                                                // IntoExpr → Atom::i()
pub use crate::{expr, symbols, rule, find, solve, func, IntoExpr, ApplyRule, Rule};

// __private.rs  (#[doc(hidden)])
pub use rmath_macros::{__expr, __symbols, __rule, __find, __solve, __func};
pub use symbolica::symbol;
pub fn into_expr<T: IntoExpr + ?Sized>(v: &T) -> Atom;
pub fn float(v: f64) -> Atom;
pub fn add(a: Atom, b: Atom) -> Atom;                        // sub mul div pow; neg(a)
pub fn call<F: Callable<N> + ?Sized, const N: usize>(f: &F, args: [Atom; N]) -> Atom;
pub fn factorial(a: Atom) -> Atom;                           // backing decided by task 1.3

// rule.rs (Phase 2)
pub struct Rule(Vec<Replacement>);
impl Rule { pub fn replacements(&self) -> &[Replacement]; }
pub trait ApplyRule { fn apply(&self, rule: &Rule) -> Atom; }
impl<T: AtomCore> ApplyRule for T { /* replace_multiple(rule.replacements()) */ }
pub fn guard<const N: usize>(wcs: [Symbol; N],
    f: impl Fn([&Atom; N]) -> bool + Send + Sync + 'static) -> Condition<PatternRestriction>;
pub fn find_all<T: AtomCore, const N: usize>(target: &T, pat: &Pattern, wcs: [Symbol; N]) -> Vec<[Atom; N]>;

// solve.rs (Phase 3), func.rs (Phase 4)
pub fn solve(eqs: &[Atom], vars: &[Symbol], over: Option<SolveDomain>) -> Result<SolutionSet, SolveError>;
pub fn compile_f64(body: &Atom, params: &[Symbol]) -> Result<ExpressionEvaluator<f64>, EvaluationError>;
```

### Expansions

- **`symbols!`** → `let #ident = #root::__private::symbol!("name"; Attrs..);` per declaration,
  no enclosing block (bindings stay visible). `r#` prefix stripped from the name.
- **`rule!`** → `{ let a_ = symbol!("a_"); Rule(vec![Replacement::new(lhs.to_pattern(),
  rhs.to_pattern()).when(guard([a_..], move |[a_..]| { let _ = (&a_,..); #guard }))]) }`.
- **`find!`** → per-site `#[derive(Debug, Clone)] struct __Match { pub a_: Atom, .. }` filled
  from `find_all(..)`, returned as `impl Iterator<Item = __Match>`.
- **`func!`** → `compile_f64(&body, &[x, y]).map(|mut ev| move |p0: f64, p1: f64| -> f64 {
  ev.evaluate_single(&[p0, p1]) })`; `ev`, `p0`, `p1` use mixed-site spans.

## AST and Error Reporting

```rust
pub struct Expr { pub kind: ExprKind, pub span: SpanRange }
#[derive(Clone, Copy)] pub struct SpanRange { pub start: Span, pub end: Span }  // Span::join is nightly-only
pub enum ExprKind {
    Num(Num),
    Ident(Ident),                                   // re-emitted verbatim
    Wildcard { ident: Ident, arity: Wild },         // Pattern mode only
    Splice(Group),                                  // `{..}` or None-delimited
    Neg { op: Span, expr: Box<Expr> },
    Binary { op: BinOp, op_span: Span, lhs: Box<Expr>, rhs: Box<Expr> },
    Factorial { bang: Span, expr: Box<Expr> },
    Call { func: Ident, args: Vec<Expr>, parens: Span },
}
pub enum BinOp { Add, Sub, Mul, Div, Pow }
pub enum Wild { One, OneOrMore, ZeroOrMore }
pub struct Error { span: SpanRange, msg: String, help: Option<String> }
pub struct Errors(Vec<Error>);
```
Parentheses widen the span only. Syntax errors emit `::core::compile_error!{"msg\n\nhelp: …"}`
with `::core` at `start` and the brace group at `end` so rustc underlines the range (syn's
technique). Detected in the macro: implicit multiplication (primary followed by ident/literal/
group, or literal with non-numeric suffix) · `!(` · lone `=` (help: `==`) · `::` paths and `.`
calls (help: wrap in `{ }`) · string/char/bool literals · lone `_` · dangling operator / empty
expression or argument · RHS wildcard unbound on LHS · duplicate or typed `func!` params ·
`symbols!` names ending in `_`. Independent items (declarations, arms, equations) report all
errors at once; inside one expression parsing stops at the first error. Name/type errors are
left to rustc (E0425, `on_unimplemented`).

## File Blueprint

| File | Action | Purpose | Phase |
|---|---|---|---|
| `Cargo.toml` | MODIFY | `[workspace]`, `[workspace.package]` (edition 2024, rust-version 1.96), `include` whitelist, `symbolica = "3.0"` (default-features off + passthrough), `rmath-macros = "=0.1.0"` | 1 |
| `src/lib.rs` | MODIFY | Remove stub; modules, `pub use symbolica;`, crate docs (README included), license notice | 1 |
| `src/macros.rs` | CREATE | `#[macro_export]` `$crate` shims with doc examples | 1–4 |
| `src/into_expr.rs`, `src/call.rs`, `src/prelude.rs`, `src/__private.rs` | CREATE | Core runtime | 1 |
| `src/rule.rs`, `src/solve.rs`, `src/func.rs` | CREATE | Runtime per later macro | 2, 3, 4 |
| `rmath-macros/Cargo.toml` | CREATE | `proc-macro = true`; proc-macro2, quote, minimal syn | 1 |
| `rmath-macros/src/lib.rs` | CREATE | Hidden entry points, `run`, `Root` | 1 |
| `rmath-macros/src/parse/{mod,cursor,error,ast,lit,expr}.rs` | CREATE | Core parser + unit tests | 1 |
| `rmath-macros/src/parse/{symbols,rule,find,solve,func}.rs` | CREATE | Per-macro input grammar | 1–4 |
| `rmath-macros/src/expand/{mod,expr,symbols,rule,find,solve,func}.rs` | CREATE | Code generation | 1–4 |
| `tests/expr_equiv.rs` | CREATE | `expr!` vs `symbolica::parse!` | 1 |
| `tests/{symbols,rule,find,solve,func}.rs` | CREATE | Behaviour tests | 1–4 |
| `tests/ui.rs`, `tests/ui/*.rs`, `tests/ui/*.stderr` | CREATE | trybuild compile-fail tests | 1–4 |
| `README.md` | CREATE | Grammar/precedence table, pitfalls, licensing section | 1 |
| `LICENSE-SYMBOLICA.md` | CREATE | Verbatim Symbolica License (copied by a person from the official published source) | 1 |
| `LICENSE-MIT`, `LICENSE-APACHE` | CREATE | rmath's own license (pending owner confirmation) | 1 |
| `.gitignore` | MODIFY | `.dev/symbolica/`, `.dev/test`, `/target` | 1 |

## API Verification Log

Verified on docs.rs/symbolica 3.0.1 during planning (2026-10-07):

| Item | Result |
|---|---|
| `Atom` ops `+ - * /`, unary `-`, with ints and `f64`; `From<Symbol>`, `From<i64>` | ✅ |
| `AtomCore::{pow, sin, cos, exp, log, sqrt, abs, to_pattern, replace, replace_multiple, pattern_match, derivative, expand, factor, solve, evaluator}` | ✅ |
| `symbol!("f"; Symmetric, Linear)`; `SymbolAttribute` = Symmetric, Antisymmetric, Cyclesymmetric, Linear, Flat, Scalar, Real, Integer, Positive | ✅ |
| `FunctionBuilder::new(Symbol).add_arg/add_args(Into<AtomOrView>).finish()` | ✅ |
| `Replacement::new(Into<Pattern>, Into<ReplaceWith>)`, `.when(Condition<PatternRestriction>)`, level/greedy options | ✅ |
| `ReplaceWith` from `Pattern`, `&Pattern`, `Atom`, `Into<Coefficient>` | ✅ |
| `WildcardRestriction::{Length, IsAtomType, HasTag, IsLiteralWildcard, Filter, Cmp, NotGreedy}`; `PatternRestriction::{Wildcard, MatchStack(Box<dyn MatchStackFn>)}` | ✅ |
| `SolveBuilder::over(SolveDomain).wrt(&[V]) -> Result<SolutionSet, SolveError>`; equations implicitly `= 0`; domains Complexes/Reals/Rationals/Integers | ✅ |
| `SolutionSet::{iter, get, Index<usize>, len, variables, as_point_dict, is_empty}` | ✅ |
| `EvaluatorBuilder::build() -> Result<ExpressionEvaluator<Complex<Rational>>, _>`; `map_coeff(&\|c\| c.re.to_f64())`; `evaluate_single(&mut self, &[T]) -> T` | ✅ |

| Symbolica cargo features (task 1.2, via docs.rs features page + `cargo info`): `default` = `tracing_max_level_info`, `faster_alloc` (→ `mimalloc`), `integer-gmp` (→ `rug`), `float-mpfr`, `native_code_generation`; pure-Rust: `integer-malachite`, `float-astro`; also `wasm`, `serde`, `bincode`, `binary_size` | ✅ |

| `From<T> for Coefficient` (task 1.3): all of `i8`–`i128`, `isize`, `u8`–`u128`, `usize` (and `&T`), `f64`, `Integer`, `Rational`, `Float`, `Complex<Rational>`, `Complex<Float>`, `(iN, iN)` tuples | ✅ |
| `AtomCore` has no method named `apply` (only `map_*`), so `ApplyRule::apply` cannot clash; `is_positive/is_nonnegative/is_integer -> ConditionResult` exist | ✅ |
| `AtomCore::evaluator<A: AtomCore>(&self, params: &[A])`; `Symbol` is **not** listed as an `AtomCore` implementor → `compile_f64` takes `&[Atom]` (symbols converted with `Atom::var`) | ✅ (design adjusted) |
| `AtomCore::pattern_match(&self, &Pattern, C: Into<Option<&Condition<PatternRestriction>>>, S: Into<Option<&MatchSettings>>) -> PatternAtomTreeIterator`; pass `None, None` for defaults | ✅ |
| `Symbol`: `get_wildcard_level() -> u8` (`x_`=1, `x__`=2, `x___`=3); `Symbol::parse`, `get_symbol`; implements `Add/Sub/Mul/Div`, `Ord`; namespace syntax `"ns::x"`; no documented character restrictions | ✅ |
| `transcendental` module: `gamma`, `erf`, `zeta`, `polylog`, hyperbolic/inverse trig, Bessel, `euler_gamma`; **no factorial** | ✅ (fallback design applies) |

| `Integer::factorial(n: u32) -> Integer`, `Integer::binom(n: i64, k: i64)`, `Integer::multinom(&[u32])`; `Integer: Into<Coefficient>`, `to_i64() -> Option<i64>` | ✅ |
| `TranscendentalFunctions::gamma(&self) -> Self::Output`, implemented for every `AtomCore`; no factorial/binomial on the trait → symbolic `n!` lowers to `(n + 1).gamma()` | ✅ |
| 1.11: `#[diagnostic::on_unimplemented]` renders `{N}` concretely ("`Sin` cannot be called with 2 argument(s)") only when the arity is fixed by the call site; calls are therefore lowered as `call::<_, N>(..)`. Without the turbofish rustc infers `N` from the single impl and reports an array-size mismatch instead. | ✅ |
| Found while compiling 1.7: `Symbol` has an inherent `call<A: FunctionArguments>(…)` method. Inherent methods shadow trait methods, so rmath's dispatch trait method is named `Callable::invoke`, not `call`. | ✅ (design adjusted) |

| Empirical (1.9): `symbol!` invoked through `$crate::__private::symbol!` inside the `symbols!` shim lands in the **user's** namespace: `symbols!(x); expr!(x) == parse!("x")` holds in an integration test | ✅ |

| 2.1: `MatchStack::{get(Symbol) -> Option<&Match>, get_atom(Symbol) -> Option<AtomView>, get_matches() -> &[(Symbol, Match)]}`; `Match::{Single(AtomView), Multiple(SliceType, Vec<AtomView>), FunctionName(Symbol)}` with `to_atom(&self) -> Atom` (argument lists wrapped in `arg(...)`) | ✅ |
| 2.1: `MatchStackFn` = blanket impl for `Clone + Send + Sync + Fn(&MatchStack<'_>) -> ConditionResult`; return `Inconclusive` until all needed wildcards are bound. `ConditionResult::{True, False, Inconclusive}`, `From<bool>` | ✅ (primary guard design confirmed; no fallback needed) |
| 2.1: `Condition<T>: From<T>` (→ `Yield`), `From<(Symbol, WildcardRestriction)>`, `&`/`\|`/`!` operators; `PatternAtomTreeIterator: Iterator<Item = HashMap<Symbol, Atom>>`; `BorrowReplacement` for `Replacement` and `&Replacement` (so `replace_multiple(&[Replacement])` works) | ✅ |

| 3.1: `SolveBuilder::wrt<V: AtomCore>(&self, &[V])` (pass `&[Atom]`), `.over(SolveDomain)` default `Complexes`; `Solution::{get(&PolyVariable) -> Option<&Atom>, coordinates() -> &[(PolyVariable, Atom)], free_variables(), conditions(), is_point(), as_point_dict()}`; `PolyVariable::{Symbol, Function, Power}`, `From<Symbol>`, `to_atom()` | ✅ |

Still to verify: conflicting re-declaration behaviour (undocumented → empirical test, deferred) ·
`MatchStackFn` signature and binding access · `Match` → `Atom` for `a__`/`a___` ·
`PatternAtomTreeIterator::Item` · `wrt` element type · `Solution` accessors · user-defined
functions inside `func!`.

## Risks and Mitigations

| Risk | L / I | Mitigation |
|---|---|---|
| Symbolica 3.x API change | M / H | All calls in `__private`; equivalence tests; CI on latest 3.x; `pub use symbolica` |
| Crate renamed / re-exported | M / M | `$crate` shim; CI fixture crate depending on rmath under another name |
| Shim hygiene hides `symbols!` bindings | L / H | User idents re-emitted verbatim; task 1.9 proves it |
| Unicode identifiers | L / L | rustc NFC-normalises; strip `r#`; verify Symbolica accepts names (1.3) |
| `^` surprises (`x^1/2` = `x/2`, `-2^2` = `-4`) | M / M | README precedence table + doctests |
| mimalloc / GMP forced on users | M / M | Passthrough features, `default-features = false` (1.2) |
| `func!` arity | L / L | Fixed-arity closure generated per site |
| Compile-time cost | L / M | No `syn/full`; short expansions; measured in 1.13 |
| trybuild drift across rustc versions | H / L | UI tests on one pinned toolchain |
| Publishing Symbolica source by accident | L / H | `include` whitelist, `.gitignore`, `cargo package --list` check |
| `f64` literal mismatch vs `parse!` | M / L | Compare numerically |
| `MatchStack` API unusable for guards | M / M | Fallback: `WildcardRestriction::filter`, single-wildcard guards only (documented) |
| No factorial in Symbolica | M / L | Fallback: `rmath::factorial` symbol; integer literals folded at compile time; symbolic `n!` stays unevaluated and `func!` returns `Err` for it |

## Open Questions (for the owner)

1. rmath's own license: MIT OR Apache-2.0 assumed.
2. Should `solve!` also accept bare expressions meaning `= 0`? (v1 spec: no, compile error.)
3. Should v1 include `apply_repeat` (apply until fixpoint, via Symbolica's `repeat`)? (v1: no.)

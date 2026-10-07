# Tasks: rmath-v1-math-macros

## Progress: [14/35]

Every "verify" task records its result in the API Verification Log in `blueprint.md` before
dependent tasks start. All Symbolica facts come from docs.rs only (ADR-0001).

## 1. Foundation: workspace, parser, `expr!`, `symbols!`

- [x] 1.1 Convert to a workspace: root `rmath` package + empty `rmath-macros` (proc-macro);
      remove the stub; `[workspace.package]` edition 2024, `rust-version = "1.96"`. Done when
      `cargo build` passes on 1.96.
- [x] 1.2 **Verify on docs.rs:** Symbolica cargo feature names (integer/float backends,
      allocator). Add `symbolica = "3.0"` with `default-features = false` and passthrough
      features `default = ["gmp", "faster-alloc"]`, `gmp`, `pure-rust`, `faster-alloc`. Add
      `include` whitelist. Narrow `.gitignore` to `.dev/symbolica/`, `.dev/test`, `/target`.
- [x] 1.3 **Verify on docs.rs:** `symbol!` namespace when called through a re-export;
      attribute syntax and conflicting re-declaration behaviour; Unicode names;
      `Into<Coefficient>` for `f64`, `i128`, `Rational`; `add_args` item type; factorial support
      (choose primary or fallback design); whether `AtomCore` already defines `apply`.
      *(Namespace-via-re-export and re-declaration conflicts are undocumented → covered by
      empirical tests in 1.9. Results logged in blueprint.md.)*
- [x] 1.4 `parse/cursor.rs` + `parse/error.rs` (SpanRange, Error, Errors, compile_error
      emission) with unit tests, including joint-punct detection for `==`, `=>`, `||`.
- [x] 1.5 `parse/lit.rs`: classify integer/float/suffixed literals. Tests: `1_000`, `0x1F`,
      `2i8`, `0.5`, `2x` (implicit-mult error), value beyond `i128` (error).
- [x] 1.6 `parse/ast.rs` + `parse/expr.rs` Pratt parser. Precedence tests: `-x^2`, `2^3^2`,
      `2^-x*3`, `x^-1`, `n!^2`, `-n!`, `(a+b)*c`. Error tests: `2x`, `x y`, `2(x)`, `f!(x)`,
      `x = 1`, `a::b`, `x.y`, `"s"`, `x +`, `f(,)`, empty input.
- [x] 1.7 Runtime: `into_expr.rs` (`IntoExpr` + impls), `call.rs` (`Callable<N>`, impl for
      `Symbol`), `prelude.rs` (built-in markers + consts, `I`), `__private.rs` helpers. Unit
      tests; confirm `#[diagnostic::on_unimplemented]` renders `{N}`.
- [x] 1.8 `expand/expr.rs` lowering + `__expr` entry + `expr!` shim in `src/macros.rs`.
      Smoke test: `expr!(x^2 + 1) == parse!("x^2+1")`.
- [x] 1.9 `symbols!`: `parse/symbols.rs`, `expand/symbols.rs`, shim. Tests: bindings visible
      after the macro (proves shim hygiene), attributes applied, `τ`, `r#type`, display name
      `tau0 = "τ_0"`, `x_` rejected, unknown attribute rejected with the attribute list.
- [x] 1.10 `tests/expr_equiv.rs`: ~30 cases vs `symbolica::parse!` covering every grammar
      rule, splices (`k`, `&Atom`, `{k+1}`), `I`, function symbols, floats compared numerically.
- [x] 1.11 `tests/ui.rs` (trybuild) with `.stderr` snapshots: undeclared name, implicit
      multiplication, `f!(`, wrong built-in arity, non-`IntoExpr` splice, unknown attribute,
      string literal, oversized integer literal.
- [ ] 1.12 `README.md` (install, flagship example, grammar + precedence table, pitfalls,
      Licensing section with runtime-rights notice), include README as crate docs so examples
      doc-test; `LICENSE-SYMBOLICA.md` (copied by a person from the official source);
      `LICENSE-MIT`/`LICENSE-APACHE` once the owner confirms. Check `cargo package --list`
      contains nothing under `.dev/`.
- [x] 1.13 `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test` on
      1.96; record `cargo build --timings` for a 200-term `expr!` in `tasks.md` Notes.

## 2. `rule!` and `find!`

- [x] 2.1 **Verify on docs.rs:** `Replacement::new` bounds; `replace_multiple` item type
      (owned vs borrowed) and whether `Replacement: Clone`; `MatchStackFn` signature and how a
      wildcard's value is read; `Match` → `Atom` for `a__`/`a___`; `pattern_match` default
      args; wildcard suffix semantics. Decide primary (MatchStack guard) vs fallback
      (`WildcardRestriction::filter`, single-wildcard guards).
- [x] 2.2 Pattern mode in the parser: wildcard classification, `parse/rule.rs` (arms, `=>`,
      optional `, if` guard tokens, rule-set braces). Unit tests incl. unbound RHS wildcard.
- [ ] 2.3 `src/rule.rs`: `Rule`, `ApplyRule`, `guard`, `find_all`. Unit tests.
- [ ] 2.4 `expand/rule.rs` + `rule!` shim. Tests: argument swap, `a__` match, guard
      (`if a_ != expr!(1)`), rule set applied in one pass, chained `.apply()`.
- [ ] 2.5 `find!`: `parse/find.rs`, `expand/find.rs` (per-site struct), shim. Tests: field
      access, `a__`, pattern without wildcards, unknown field is a compile error.
- [ ] 2.6 trybuild: unbound RHS wildcard, guard naming an unknown wildcard, lone `_`,
      missing `=>`, guard with top-level comma without parentheses.
- [ ] 2.7 Docs for `rule!`/`find!` (README section + macro docs), including the
      parenthesised-guard rule and the Phase-2 fallback limitation if taken.

## 3. `solve!`

- [ ] 3.1 **Verify on docs.rs:** `wrt` element type (`Symbol` vs `Atom`); default domain when
      `.over` is omitted; `Solution` per-variable accessor; `as_point_dict` key type
      (`PolyVariable` from `Symbol`).
- [ ] 3.2 `parse/solve.rs`: `[eq, …]` list, `lhs == rhs` required, `for vars`, optional
      `over Domain` (Complexes/Reals/Rationals/Integers). Unit tests incl. `=` vs `==`.
- [ ] 3.3 `src/solve.rs` + `expand/solve.rs` + shim: each equation lowered to `sub(lhs, rhs)`;
      domain emitted as `SolveDomain::#ident` at the user's span.
- [ ] 3.4 Tests: 2×2 linear system, `over Reals` on `x^2 + 1 == 0` is empty, inconsistent
      system → `Err`, spliced `f64` coefficient. trybuild: missing `for`, `=` instead of `==`,
      bare expression without `==`, unknown domain.
- [ ] 3.5 Docs; optional convenience accessor only if 3.1 found a clean `Solution` API.

## 4. `func!`

- [ ] 4.1 **Verify on docs.rs:** `evaluator` parameter type; `map_coeff` closure type;
      `evaluate_single` availability for `f64`; behaviour with user-declared (non-built-in)
      function symbols in the body.
- [ ] 4.2 `parse/func.rs`: `|a, b|` and `||` (joint punct), identifiers only; reject typed or
      duplicate params. Unit tests.
- [ ] 4.3 `src/func.rs` (`compile_f64`) + `expand/func.rs` (fixed-arity closure, mixed-site
      temporaries) + shim.
- [ ] 4.4 Tests: arity 0, 1, 2, 5; values vs hand-computed `f64`; captured value frozen after
      mutation; free symbol → `Err`; existing `Atom` as body. trybuild: typed param, duplicate
      param, wrong call arity.
- [ ] 4.5 Docs: `func!` section, README feature table.

## 5. Release readiness

- [ ] 5.1 CI workflow: fmt, clippy `-D warnings`, test on 1.96 and stable, trybuild on the
      pinned toolchain, build with `--no-default-features --features pure-rust`, fixture crate
      depending on rmath under a renamed package.
- [ ] 5.2 `cargo doc --no-deps` clean (no broken intra-doc links); `cargo publish --dry-run`
      for both crates.
- [ ] 5.3 Update `CONTEXT.md`, `.dev/project.md`, ADR links in README "Contributing".
- [ ] 5.4 Owner review of the public API surface against `design.md` "API Changes".
- [ ] 5.5 Archive the change (`/dev-workflow:archive`), merging delta specs into `.dev/specs/`.

---

## Notes

Implementation notes, timings and verification results are appended here during development.

- **1.2** rmath features: `default = ["gmp", "faster-alloc", "native-codegen",
  "symbolica/tracing_max_level_info"]` (mirrors Symbolica's default set). Verified with
  `cargo tree`: defaults pull `rug` and `rustfs-mimalloc` (Symbolica's allocator dep is the
  `rustfs-mimalloc` package renamed to `mimalloc`); `--no-default-features --features pure-rust`
  has no `rug` and resolves `malachite-nz`. Full default build passed locally (Rust 1.97,
  Homebrew GMP/MPFR). A full *compile* with `pure-rust` is deferred to CI (task 5.1).
- **1.2** `rtk` filters cargo output; use `rtk proxy cargo …` when exact output matters.
- **1.7/1.8** Symbolica's unlicensed mode is single-core and **aborts the process (SIGABRT)**
  when used from two threads at once (reproduced with `--test-threads=2`; one thread passes).
  Fix: `.cargo/config.toml` sets `RUST_TEST_THREADS = "1"` for the workspace. README (1.12)
  must tell users the same for their own test suites, and CI (5.1) inherits the config.
- **1.7** `Symbol` has an inherent `call(...)` in Symbolica, so rmath's dispatch method is
  `Callable::invoke`. Symbolic `n!` lowers to `(n + 1).gamma()`; integer `n!` is exact.
- **1.9** `symbols!` namespace verified empirically: `symbols!(x); expr!(x) == parse!("x")`.
  rustc's `mixed_script_confusables` lint fires on a crate whose only Greek identifier is e.g.
  `θ`; README should mention `#![allow(mixed_script_confusables)]` for single-letter symbols.
- **1.9** `#[diagnostic::on_unimplemented]` renders `{N}` as `_` when the arity cannot be
  inferred (e.g. calling a plain `fn` item inside `expr!`); 1.11 must check a concrete arity.
- **1.12 (partial)** README written and included as crate docs (`#![doc = include_str!]`,
  3 doctested blocks). `cargo package --list` ships only `src/`, manifests, README: nothing
  under `.dev/`. Still needed from the owner: confirm MIT OR Apache-2.0 (then add
  `LICENSE-MIT`/`LICENSE-APACHE`), copy `LICENSE-SYMBOLICA.md` from the official source, and a
  `repository` URL for `Cargo.toml`.
- **1.13** Compile-time cost: `examples/large_expr.rs` (200 terms, 5.4 KB of `expr!`) rebuilds
  incrementally in 0.44 s on stable 1.97 (a 1-term example measured 0.84 s), so macro expansion
  is negligible. The example is kept as a compile-time smoke test; it is not packaged.

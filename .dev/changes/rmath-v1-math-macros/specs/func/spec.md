# Delta for Func

The `func!` macro.

## ADDED Requirements

### Requirement: Compile an expression to a numeric closure

`func!(|p1, p2, …| body)` SHALL build `body` with the shared grammar, compile it with
Symbolica's evaluator for the parameters `p1…pn` (which must be `Symbol`s in scope), and
return `Result<impl FnMut(f64, …) -> f64, EvaluationError>` with exactly n `f64` parameters.

#### Scenario: Two-parameter function

- WHEN the user writes `let mut f = func!(|x, y| x^2 + sin(y))?;`
- THEN `f(1.0, 0.0)` returns `1.0` (within floating-point tolerance)

#### Scenario: Existing Atom as body

- WHEN `e` is an `Atom` and the user writes `func!(|x, y| e)`
- THEN the closure evaluates `e` with `x`, `y` substituted positionally

#### Scenario: Body references a symbol that is not a parameter

- WHEN the user writes `func!(|x| x + y)` with `y` a symbol not listed as a parameter
- THEN the macro returns `Err(EvaluationError)` (Symbolica reports the free symbol); it does
  not panic

#### Scenario: Parameter is not a Symbol

- WHEN the user writes `func!(|n| n^2)` with `n: i32` in scope
- THEN compilation fails with a trait-bound error (parameters must be `Symbol`)

### Requirement: Captured values are frozen

Rust values spliced into the body SHALL be evaluated once, when `func!` runs, and baked into
the compiled function.

#### Scenario: Later mutation does not affect the closure

- WHEN the user writes `let mut k = 2; let mut f = func!(|x| k * x)?; k = 5;`
- THEN `f(1.0)` returns `2.0`

### Requirement: Arity is fixed at compile time

The closure SHALL accept exactly as many `f64` arguments as parameters were listed.

#### Scenario: Wrong call arity

- WHEN the user writes `let mut f = func!(|x, y| x + y)?; f(1.0);`
- THEN compilation fails with rustc's argument-count error

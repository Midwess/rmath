# Delta for Solve

The `solve!` macro.

## ADDED Requirements

### Requirement: Solve a system written with `==`

`solve!([eq, …] for v1, v2, …)` SHALL accept equations of the form `lhs == rhs` in the shared
grammar, lower each to `lhs - rhs`, and return Symbolica's `Result<SolutionSet, SolveError>`
for the listed unknowns, in the listed order.

#### Scenario: Linear system

- WHEN the user writes `solve!([2*x + y == 3, x - y == 0] for x, y)`
- THEN the result is `Ok` with one branch assigning `x = 1`, `y = 1`

#### Scenario: Equation without `==`

- WHEN the user writes `solve!([x^2 - 1] for x)`
- THEN compilation fails at the equation with "expected `lhs == rhs`"

#### Scenario: Unknown not declared

- WHEN the user writes `solve!([x == 1] for z)` and `z` is not in scope
- THEN compilation fails with `E0425` for `z`

### Requirement: Optional domain clause

`solve!([…] for vars over Domain)` SHALL select the solution domain, where `Domain` is one of
`Complexes`, `Reals`, `Rationals`, `Integers`. Without the clause, Symbolica's default applies.

#### Scenario: Real solutions only

- WHEN the user writes `solve!([x^2 + 1 == 0] for x over Reals)`
- THEN the returned `SolutionSet` is empty (per Symbolica's `is_empty`)

#### Scenario: Unknown domain name

- WHEN the user writes `… over Quaternions`
- THEN compilation fails at `Quaternions` listing the four supported domains

### Requirement: Rust values in equations

Equations SHALL accept splices exactly as `expr!` does.

#### Scenario: Exact values spliced in

- WHEN the user writes `solve!([k * x == m] for x)` with `k: i32` and `m: Rational`
- THEN the result assigns `x = m / k` with both values substituted

#### Scenario: Inexact values are rejected by the solver

- WHEN the user writes `solve!([2 * x == m] for x)` with `m: f64`
- THEN the macro returns `Err(SolveError::UnsupportedProblem(..))`, because Symbolica's exact
  solver requires exact coefficients (this is Symbolica's behaviour, passed through unchanged)

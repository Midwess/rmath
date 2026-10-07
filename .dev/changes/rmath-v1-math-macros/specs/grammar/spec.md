# Delta for Grammar

The math-syntax grammar shared by `expr!`, `rule!`, `find!`, `solve!` and `func!`.

## ADDED Requirements

### Requirement: Operator precedence and associativity

The parser SHALL apply, from loosest to tightest: binary `+`/`-` (left), `*`/`/` (left),
unary `-` (prefix), `^` (right-associative), postfix `!`, then atoms (calls, parentheses,
literals, identifiers, splices).

#### Scenario: Power binds tighter than unary minus

- WHEN the user writes `expr!(-x^2)`
- THEN the result equals `-(x^2)` (i.e. `symbolica::parse!("-x^2")`)

#### Scenario: Power is right-associative

- WHEN the user writes `expr!(2^3^2)`
- THEN the result equals `2^9`

#### Scenario: Negative exponent

- WHEN the user writes `expr!(x^-1)`
- THEN the result equals `x^(-1)`

#### Scenario: Factorial binds tightest

- WHEN the user writes `expr!(n!^2)` with `n` a symbol
- THEN the result equals `(n!)^2`

### Requirement: Literals

The parser SHALL accept integer literals as exact integers and decimal literals as
floating-point coefficients.

#### Scenario: Exact rational via division

- WHEN the user writes `expr!(1/3 * x)`
- THEN the coefficient is the exact rational `1/3`

#### Scenario: Decimal literal is a float

- WHEN the user writes `expr!(0.5 * x)`
- THEN the coefficient is a floating-point `0.5`

#### Scenario: Integer literal too large

- WHEN an integer literal does not fit in `i128`
- THEN compilation fails with an error at the literal's span saying the literal exceeds the
  supported range in v1

### Requirement: Rust splices

The parser SHALL treat a brace-delimited group `{ ... }` as a Rust expression spliced through
`IntoExpr`.

#### Scenario: Splice an arithmetic Rust expression

- WHEN the user writes `expr!(x + {k + 1})` with `k: i32`
- THEN the result equals `x + (k + 1)` evaluated in Rust first

### Requirement: Rejected syntax has actionable diagnostics

The parser SHALL reject implicit multiplication, macro-call syntax and empty input with
compile errors located at the offending token.

#### Scenario: Implicit multiplication

- WHEN the user writes `expr!(2x)`
- THEN compilation fails at `x` with "implicit multiplication is not supported; write `2*x`"

#### Scenario: Macro-call syntax inside the body

- WHEN the user writes `expr!(f!(x))`
- THEN compilation fails at `!` with a message suggesting `{ f!(x) }` to splice Rust code

#### Scenario: Empty body

- WHEN the user writes `expr!()`
- THEN compilation fails with "expected an expression"

#### Scenario: Unbalanced or trailing tokens

- WHEN the user writes `expr!(x + )` or `expr!(x y)`
- THEN compilation fails at the unexpected or missing token with a message naming what was
  expected

### Requirement: Identifiers

The parser SHALL accept any valid Rust identifier, including non-ASCII ones, and re-emit it
unchanged for the compiler to resolve.

#### Scenario: Unicode symbol names

- WHEN the user declares `symbols!(θ, κ)` and writes `expr!(κ * cos(θ))`
- THEN it compiles and produces `κ*cos(θ)`

#### Scenario: Undeclared identifier

- WHEN the user writes `expr!(xx + 1)` and no `xx` is in scope
- THEN compilation fails with rustc's `E0425` for `xx`

# Delta for Symbols and Expressions

`symbols!`, `expr!`, `IntoExpr`, `Callable` and the prelude built-ins.

## ADDED Requirements

### Requirement: Declare symbols with `symbols!`

The system SHALL provide `symbols!` which binds each listed name to a `symbolica::atom::Symbol`
of the same name, with optional attributes written `name: A + B` and an optional display name
written `name = "text"`.

#### Scenario: Plain symbols

- WHEN the user writes `symbols!(x, y, z);`
- THEN `x`, `y`, `z` are `Symbol` bindings whose names are `"x"`, `"y"`, `"z"`

#### Scenario: Attributes

- WHEN the user writes `symbols!(t: Real + Positive, f: Symmetric);`
- THEN `t` carries the `Real` and `Positive` attributes and `f` carries `Symmetric`

#### Scenario: Display name differs from Rust name

- WHEN the user writes `symbols!(tau0 = "τ_0");`
- THEN `tau0` is a `Symbol` whose name is `"τ_0"`

#### Scenario: Unknown attribute

- WHEN the user writes `symbols!(x: Hermitian);`
- THEN compilation fails at `Hermitian` listing the supported attributes (Symmetric,
  Antisymmetric, Cyclesymmetric, Linear, Flat, Scalar, Real, Integer, Positive)

#### Scenario: Same name declared twice in different scopes

- WHEN `symbols!(x)` is executed in two different functions
- THEN both bindings refer to the same interned Symbolica symbol

### Requirement: Build expressions with `expr!`

The system SHALL provide `expr!` which parses the grammar and evaluates to a
`symbolica::atom::Atom` built entirely from Symbolica public-API calls, without runtime parsing.

#### Scenario: Equivalence with Symbolica's parser

- WHEN the user writes `expr!(x^2 + sin(y)/2 - 3*f(x, y))` with `x`, `y`, `f` declared
- THEN the result equals `symbolica::parse!("x^2 + sin(y)/2 - 3*f(x, y)")` for the same symbols

#### Scenario: Splice a Rust integer without a marker

- WHEN the user writes `let k = 5; expr!(k * x^2)`
- THEN the result equals `5*x^2`

#### Scenario: Splice an existing Atom

- WHEN `a` and `b` are `Atom`s and the user writes `expr!(a + b)`
- THEN the result equals `a + b` and `a`, `b` are still usable afterwards (borrowed, not moved)

#### Scenario: Imaginary unit

- WHEN the user writes `expr!(exp(I * x))`
- THEN the result equals `exp(𝑖*x)` using Symbolica's imaginary unit

#### Scenario: Symbol used as a function

- WHEN `f` is declared with `symbols!(f: Symmetric)` and the user writes `expr!(f(y, x))`
- THEN the result is the function `f` applied to `(y, x)`, normalised by Symbolica as
  symmetric (equal to `f(x, y)`)

### Requirement: Built-ins are prelude items

The system SHALL expose `sin`, `cos`, `exp`, `log`, `sqrt`, `abs` and `I` as ordinary Rust
items in `rmath::prelude`, resolvable by the compiler, and arity-checked at compile time.

#### Scenario: Wrong arity on a built-in

- WHEN the user writes `expr!(sin(x, y))`
- THEN compilation fails with a message stating `sin` cannot be called with 2 arguments

#### Scenario: Built-in without the prelude

- WHEN the user writes `expr!(sin(x))` without importing `rmath::prelude::*` or `rmath::sin`
- THEN compilation fails with `E0425` for `sin`

### Requirement: `IntoExpr` conversions

The system SHALL implement `IntoExpr` for `Symbol`, `Atom`, `&Atom`, `i8`–`i64`, `u8`–`u64`,
`f64` and `symbolica::domains::rational::Rational`.

#### Scenario: Unsupported splice type

- WHEN the user writes `expr!(s + 1)` where `s: String`
- THEN compilation fails with a trait-bound error naming `IntoExpr`

### Requirement: Generated code is self-contained

Expansions SHALL reference only paths rooted at the rmath crate (via `$crate`), never
`::symbolica::...`, and SHALL not introduce glob imports into the user's scope.

#### Scenario: User depends only on rmath

- WHEN a crate lists `rmath` but not `symbolica` in `[dependencies]` and uses `expr!`
- THEN it compiles

#### Scenario: Renamed dependency

- WHEN a crate depends on `rm = { package = "rmath", version = "0.1" }` and uses `rm::expr!`
- THEN it compiles

#### Scenario: User local named like a helper

- WHEN the user has a local variable `add` in scope and writes `expr!(x + 1)`
- THEN it compiles and `add` is unaffected

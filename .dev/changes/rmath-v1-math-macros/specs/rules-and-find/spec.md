# Delta for Rules and Find

`rule!`, `Rule`, the `ApplyRule` extension trait and `find!`.

## ADDED Requirements

### Requirement: Wildcards declared by trailing underscore

Inside `rule!` and `find!`, an identifier ending in `_`, `__` or `___` SHALL denote a wildcard
matching exactly one argument, one or more, or zero or more respectively, created with
Symbolica's wildcard naming convention. Wildcards need no `symbols!` declaration.

#### Scenario: Single-argument wildcard

- WHEN the user writes `rule!(f(a_) => a_^2)` and applies it to `f(1) + f(x)`
- THEN the result equals `1 + x^2`

#### Scenario: Multi-argument wildcard

- WHEN the user writes `rule!(g(a__) => 0)` and applies it to `g(1, 2, 3) + 7`
- THEN the result equals `7`

### Requirement: Single rewrite rule

`rule!(lhs => rhs)` SHALL produce a `Rule` wrapping a Symbolica `Replacement` whose pattern is
`lhs` and whose replacement is `rhs`, with wildcards shared between both sides.

#### Scenario: Reorder arguments

- WHEN the user writes `rule!(f(a_, b_) => g(b_, a_))` and applies it to `f(1, x) + f(y, 2)`
- THEN the result equals `g(x, 1) + g(2, y)`

#### Scenario: Wildcard only on the right-hand side

- WHEN the user writes `rule!(f(a_) => b_)`
- THEN compilation fails at `b_` with "wildcard `b_` does not appear on the left-hand side"

### Requirement: Conditional rule

`rule!(lhs => rhs, if cond)` SHALL attach `cond`, an arbitrary Rust `bool` expression in which
every left-hand-side wildcard is bound to its matched `Atom`, as a restriction on the match.

#### Scenario: Condition filters matches

- WHEN the user writes `rule!(f(a_) => 0, if a_ != expr!(1))` and applies it to `f(1) + f(2)`
- THEN the result equals `f(1)`

#### Scenario: Condition containing a top-level comma

- WHEN the user writes a guard with a top-level comma, e.g. `if (check(&a_, &b_))`
- THEN it is accepted when parenthesised, and without parentheses compilation fails with a
  message asking to wrap the guard in parentheses

### Requirement: Rule sets

`rule! { lhs1 => rhs1, lhs2 => rhs2, }` SHALL produce a `Rule` holding several replacements,
applied as a single simultaneous replacement pass.

#### Scenario: Two rules in one pass

- WHEN the user applies `rule! { f(a_) => a_, g(a_) => 2*a_ }` to `f(x) + g(y)`
- THEN the result equals `x + 2*y`

### Requirement: `ApplyRule` extension trait

The system SHALL provide `ApplyRule` with `fn apply(&self, rule: &Rule) -> Atom`, available via
the prelude on `Atom` and every other `AtomCore` implementor.

#### Scenario: Chained application

- WHEN the user writes `e.apply(&r1).apply(&r2)`
- THEN the second rule is applied to the output of the first

### Requirement: Typed matches with `find!`

`find!(expr, pattern)` SHALL return an iterator whose items expose one `Atom` field per wildcard
in `pattern`, named exactly as the wildcard.

#### Scenario: Field access on matches

- WHEN the user writes `for m in find!(e, f(a_, b_)) { … }` with `e = f(1, 2) + f(x, y)`
- THEN the loop sees two items with `(m.a_, m.b_)` equal to `(1, 2)` and `(x, y)` in some order

#### Scenario: Unknown field

- WHEN the user writes `m.c_` for a match from `find!(e, f(a_, b_))`
- THEN compilation fails with rustc's "no field `c_`" error

#### Scenario: Pattern without wildcards

- WHEN the user writes `find!(e, f(1))`
- THEN the iterator yields one empty item per occurrence of `f(1)` in `e`

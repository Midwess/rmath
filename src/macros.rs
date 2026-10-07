//! The public macros. Each is a thin `macro_rules!` shim that forwards `$crate` to the
//! procedural macro, so generated code can refer back to this crate whatever the user named
//! the dependency.

/// Build a Symbolica expression from math syntax.
///
/// ```
/// use rmath::{expr, symbolica::{parse, symbol}};
///
/// let x = symbol!("x");
/// assert_eq!(expr!(x^2 + 1), parse!("x^2 + 1"));
/// ```
///
/// Grammar: `+ - * /`, `^` (right-associative, binds tighter than unary minus so `-x^2` is
/// `-(x^2)`), postfix `!`, parentheses, calls `f(a, b)`, integer and decimal literals,
/// identifiers (symbols, built-ins or any `IntoExpr` value in scope) and `{ rust }` splices.
/// There is no implicit multiplication: write `2 * x`.
#[macro_export]
macro_rules! expr {
    ($($t:tt)*) => {
        $crate::__private::__expr! { $crate ; $($t)* }
    };
}

/// Declare Symbolica symbols as compiler-checked local bindings.
///
/// ```
/// use rmath::{expr, symbols, symbolica::parse};
///
/// symbols!(x, y);
/// assert_eq!(expr!(x + y), parse!("x + y"));
/// ```
///
/// Each name becomes a `let` binding of type `Symbol`, so a typo inside `expr!` is an
/// ordinary "cannot find value" compile error. Symbols are interned globally by name: two
/// `symbols!(x)` in different functions refer to the same symbol.
#[macro_export]
macro_rules! symbols {
    ($($t:tt)*) => {
        $crate::__private::__symbols! { $crate ; $($t)* }
    };
}

/// Build a rewrite [`Rule`](crate::Rule) in `match`-like syntax.
///
/// ```
/// use rmath::{prelude::*, symbolica::parse};
///
/// symbols!(f, g, x, y);
/// let swap = rule!(f(a_, b_) => g(b_, a_));
/// assert_eq!(expr!(f(1, x) + f(y, 2)).apply(&swap), parse!("g(x, 1) + g(2, y)"));
/// ```
///
/// Wildcards are declared implicitly by a trailing underscore: `a_` matches one argument,
/// `a__` one or more, `a___` zero or more. A guard `, if cond` is any Rust `bool` expression
/// in which each wildcard of the pattern is bound to its matched `Atom`; a guard containing a
/// top-level comma must be parenthesised. Several rules in braces, `rule! { a => b, c => d }`,
/// are applied together in one pass.
#[macro_export]
macro_rules! rule {
    ($($t:tt)*) => {
        $crate::__private::__rule! { $crate ; $($t)* }
    };
}

/// Find every match of a pattern and get the wildcards back as named fields.
///
/// ```
/// use rmath::prelude::*;
///
/// symbols!(f, x, y);
/// let e = expr!(f(1, 2) + f(x, y));
/// let mut pairs: Vec<String> = find!(e, f(a_, b_))
///     .map(|m| format!("{} {}", m.a_, m.b_))
///     .collect();
/// pairs.sort();
/// assert_eq!(pairs, ["1 2", "x y"]);
/// ```
///
/// `find!(expression, pattern)` returns an iterator; each item has one `Atom` field per
/// wildcard of the pattern, named exactly like the wildcard, so a typo in a field name is a
/// compile error. Matches are collected eagerly.
#[macro_export]
macro_rules! find {
    ($($t:tt)*) => {
        $crate::__private::__find! { $crate ; $($t)* }
    };
}

/// Solve a system of equations written with `==`.
///
/// ```
/// use rmath::prelude::*;
///
/// symbols!(x, y);
/// let sols = solve!([2 * x + y == 3, x - y == 0] for x, y).unwrap();
/// assert_eq!(sols[0].value(x), Some(&expr!(1)));
/// assert_eq!(sols[0].value(y), Some(&expr!(1)));
/// ```
///
/// Each `lhs == rhs` is passed to Symbolica as `lhs - rhs`; the result is Symbolica's own
/// `Result<SolutionSet, SolveError>`. An optional domain clause restricts the unknowns:
/// `solve!([x^2 + 1 == 0] for x over Reals)` (`Complexes` is the default; also `Rationals`,
/// `Integers`). `value(symbol)` on a solution branch comes from [`SolutionExt`](crate::SolutionExt).
#[macro_export]
macro_rules! solve {
    ($($t:tt)*) => {
        $crate::__private::__solve! { $crate ; $($t)* }
    };
}

/// Compile an expression into a fast numeric closure.
///
/// ```
/// use rmath::prelude::*;
///
/// symbols!(x, y);
/// let mut f = func!(|x, y| x ^ 2 + sin(y)).unwrap();
/// assert_eq!(f(3.0, 0.0), 9.0);
/// ```
///
/// `func!(|params| body)` builds `body` with the usual grammar (the parameters are symbols in
/// scope), compiles it with Symbolica's evaluator and returns
/// `Result<impl FnMut(f64, ..) -> f64, EvaluationError>` with exactly one `f64` argument per
/// parameter. Rust values spliced into the body are frozen when the function is created. The
/// closure is `FnMut` because the evaluator reuses internal buffers, so bind it with `let mut`.
#[macro_export]
macro_rules! func {
    ($($t:tt)*) => {
        $crate::__private::__func! { $crate ; $($t)* }
    };
}

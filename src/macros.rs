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

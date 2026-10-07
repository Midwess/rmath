//! Function-call dispatch for `f(a, b)` inside macro bodies.

use symbolica::atom::{Atom, FunctionBuilder, Symbol};

/// Something that can be called with exactly `N` expression arguments.
///
/// Implemented for every `Symbol` (any arity, building `f(a, b)`) and for the built-in
/// function markers in the prelude (fixed arity, checked at compile time). Expansions call it
/// as `Callable::invoke(&f, [..])`; the name avoids Symbolica's own `Symbol::call`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be called with {N} argument(s)",
    note = "built-in functions have a fixed number of arguments; symbols accept any number"
)]
pub trait Callable<const N: usize> {
    fn invoke(&self, args: [Atom; N]) -> Atom;
}

impl<const N: usize> Callable<N> for Symbol {
    fn invoke(&self, args: [Atom; N]) -> Atom {
        FunctionBuilder::new(*self).add_args(args).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IntoExpr;
    use symbolica::{function, symbol};

    #[test]
    fn a_symbol_is_callable_with_any_number_of_arguments() {
        let (f, x, y) = symbol!("rmath_call_f", "rmath_call_x", "rmath_call_y");
        assert_eq!(f.invoke([x.to_expr(), y.to_expr()]), function!(f, x, y));
        assert_eq!(f.invoke([x.to_expr()]), function!(f, x));
        assert_eq!(f.invoke([]), function!(f));
    }
}

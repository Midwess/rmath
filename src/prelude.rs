//! Everything needed to write math with rmath: the macros, Symbolica's core types, the
//! built-in functions and constants, and rmath's traits.

pub use crate::{
    ApplyRule, Callable, IntoExpr, Rule, SolutionExt, expr, find, func, rule, solve, symbols,
};
pub use symbolica::atom::{Atom, AtomCore, Symbol};

/// The imaginary unit. Spliced into expressions as Symbolica's `𝑖`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct I;

impl IntoExpr for I {
    fn to_expr(&self) -> Atom {
        Atom::i()
    }
}

/// Built-in one-argument functions. Each is a marker type plus a lowercase constant, so
/// `sin(x)` inside a macro resolves to an ordinary Rust item that the compiler checks.
macro_rules! builtin_unary {
    ($($(#[$doc:meta])* $marker:ident => $name:ident / $method:ident;)*) => {$(
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $marker;

        impl Callable<1> for $marker {
            fn invoke(&self, [x]: [Atom; 1]) -> Atom {
                x.$method()
            }
        }

        $(#[$doc])*
        #[allow(non_upper_case_globals)]
        pub const $name: $marker = $marker;
    )*};
}

builtin_unary! {
    /// Sine.
    Sin => sin / sin;
    /// Cosine.
    Cos => cos / cos;
    /// Exponential function.
    Exp => exp / exp;
    /// Natural logarithm.
    Log => log / log;
    /// Square root.
    Sqrt => sqrt / sqrt;
    /// Absolute value.
    Abs => abs / abs;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imaginary_unit_marker_converts_to_symbolicas_i() {
        assert_eq!(I.to_expr(), Atom::i());
    }

    #[test]
    fn built_in_markers_apply_the_matching_symbolica_function() {
        use symbolica::symbol;
        let x = Atom::var(symbol!("rmath_prelude_x"));
        assert_eq!(sin.invoke([x.clone()]), x.sin());
        assert_eq!(cos.invoke([x.clone()]), x.cos());
        assert_eq!(exp.invoke([x.clone()]), x.exp());
        assert_eq!(log.invoke([x.clone()]), x.log());
        assert_eq!(sqrt.invoke([x.clone()]), x.sqrt());
        assert_eq!(abs.invoke([x.clone()]), x.abs());
    }
}

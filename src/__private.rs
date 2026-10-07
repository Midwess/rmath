//! Helpers called by macro expansions. Not part of the public API: anything here may change
//! in a patch release.

use symbolica::{atom::AtomCore, domains::integer::Integer, id::Pattern};

use crate::{Callable, IntoExpr};

pub use crate::func::compile_f64;
pub use crate::rule::{Rule, find_all, guard};
pub use crate::solve::solve;
pub use rmath_macros::{__expr, __find, __func, __rule, __solve, __symbols};
pub use symbolica::{atom::Atom, id::Replacement, solve::SolveDomain, symbol};

pub fn into_expr<T: IntoExpr + ?Sized>(value: &T) -> Atom {
    value.to_expr()
}

pub fn float(value: f64) -> Atom {
    Atom::num(value)
}

pub fn add(lhs: Atom, rhs: Atom) -> Atom {
    lhs + rhs
}

pub fn sub(lhs: Atom, rhs: Atom) -> Atom {
    lhs - rhs
}

pub fn mul(lhs: Atom, rhs: Atom) -> Atom {
    lhs * rhs
}

pub fn div(lhs: Atom, rhs: Atom) -> Atom {
    lhs / rhs
}

pub fn pow(base: Atom, exp: Atom) -> Atom {
    base.pow(exp)
}

pub fn neg(value: Atom) -> Atom {
    -value
}

pub fn call<F: Callable<N> + ?Sized, const N: usize>(f: &F, args: [Atom; N]) -> Atom {
    f.invoke(args)
}

/// `n!`: exact for non-negative integers that fit `u32`, otherwise `Γ(n + 1)`.
pub fn factorial(value: Atom) -> Atom {
    use symbolica::transcendental::TranscendentalFunctions;

    if let Ok(n) = u64::try_from(value.as_atom_view())
        && let Ok(n) = u32::try_from(n)
    {
        return Atom::num(Integer::factorial(n));
    }
    (value + 1).gamma()
}

/// Convert an expression into a pattern without needing `AtomCore` in the caller's scope.
pub fn pattern(expr: Atom) -> Pattern {
    expr.to_pattern()
}

/// A `func!` parameter: must be a `Symbol`, checked by the compiler at the call site.
pub fn param(symbol: &symbolica::atom::Symbol) -> Atom {
    Atom::var(*symbol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::sin;
    use symbolica::{symbol, transcendental::TranscendentalFunctions};

    #[test]
    fn arithmetic_helpers_match_symbolica_operators() {
        let x = Atom::var(symbol!("rmath_private_x"));
        let two = Atom::num(2);
        assert_eq!(add(x.clone(), two.clone()), &x + &two);
        assert_eq!(sub(x.clone(), two.clone()), &x - &two);
        assert_eq!(mul(x.clone(), two.clone()), &x * &two);
        assert_eq!(div(x.clone(), two.clone()), &x / &two);
        assert_eq!(pow(x.clone(), two.clone()), x.pow(&two));
        assert_eq!(neg(x.clone()), -&x);
        assert_eq!(float(0.5), Atom::num(0.5));
        assert_eq!(into_expr(&3), Atom::num(3));
        assert_eq!(call(&sin, [x.clone()]), x.sin());
    }

    #[test]
    fn factorial_of_a_small_integer_is_exact() {
        assert_eq!(factorial(Atom::num(5)), Atom::num(120));
        assert_eq!(factorial(Atom::num(0)), Atom::num(1));
        assert_eq!(
            factorial(Atom::num(20)),
            Atom::num(2_432_902_008_176_640_000i64)
        );
    }

    #[test]
    fn factorial_of_anything_else_is_gamma_of_the_successor() {
        let x = Atom::var(symbol!("rmath_private_fact_x"));
        assert_eq!(factorial(x.clone()), (&x + 1).gamma());
        assert_eq!(factorial(Atom::num(-3)), Atom::num(-2).gamma());
    }
}

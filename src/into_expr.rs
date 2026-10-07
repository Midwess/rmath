//! Conversion of Rust values into Symbolica expressions for splicing into macros.

use symbolica::atom::{Atom, Symbol};

/// A value that can appear inside `expr!` and friends.
///
/// Every identifier in a macro body that is not a call head is spliced as
/// `into_expr(&value)`, so conversion borrows and never moves the original.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used in a math expression",
    note = "implement `rmath::IntoExpr`, or splice a block that evaluates to an `Atom`: `{{ ... }}`"
)]
pub trait IntoExpr {
    fn to_expr(&self) -> Atom;
}

impl<T: IntoExpr + ?Sized> IntoExpr for &T {
    fn to_expr(&self) -> Atom {
        (**self).to_expr()
    }
}

impl IntoExpr for Atom {
    fn to_expr(&self) -> Atom {
        self.clone()
    }
}

impl IntoExpr for Symbol {
    fn to_expr(&self) -> Atom {
        Atom::var(*self)
    }
}

impl IntoExpr for symbolica::domains::rational::Rational {
    fn to_expr(&self) -> Atom {
        Atom::num(self.clone())
    }
}

/// Numbers Symbolica accepts as coefficients: exact integers of every width, and `f64`.
macro_rules! impl_into_expr_for_numbers {
    ($($ty:ty),*) => {$(
        impl IntoExpr for $ty {
            fn to_expr(&self) -> Atom {
                Atom::num(*self)
            }
        }
    )*};
}

impl_into_expr_for_numbers!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f64
);

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::{domains::rational::Rational, symbol};

    #[test]
    fn symbols_numbers_and_atoms_convert_without_being_moved() {
        let x = symbol!("rmath_into_expr_x");
        assert_eq!(x.to_expr(), Atom::var(x));
        assert_eq!(3i64.to_expr(), Atom::num(3));
        assert_eq!(0.5f64.to_expr(), Atom::num(0.5));

        let a = Atom::var(x) + 1;
        let b = a.to_expr();
        assert_eq!(b, a, "`a` is still usable: conversion borrows");
    }

    #[test]
    fn every_primitive_integer_width_converts_exactly() {
        assert_eq!(7i8.to_expr(), Atom::num(7));
        assert_eq!(7i16.to_expr(), Atom::num(7));
        assert_eq!((-7i32).to_expr(), Atom::num(-7));
        assert_eq!(i128::MAX.to_expr(), Atom::num(i128::MAX));
        assert_eq!(7u8.to_expr(), Atom::num(7));
        assert_eq!(7u16.to_expr(), Atom::num(7));
        assert_eq!(7u32.to_expr(), Atom::num(7));
        assert_eq!(u64::MAX.to_expr(), Atom::num(u64::MAX));
        assert_eq!(u128::MAX.to_expr(), Atom::num(u128::MAX));
    }

    #[test]
    fn rationals_convert_to_exact_fractions() {
        let third = Rational::new(1, 3);
        assert_eq!(third.to_expr(), Atom::num(1) / Atom::num(3));
        assert_eq!(Rational::new(-4, 2).to_expr(), Atom::num(-2));
    }
}

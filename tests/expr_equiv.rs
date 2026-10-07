//! `expr!` must agree with Symbolica's own parser (or the equivalent API call) for every
//! grammar rule.

use rmath::{
    expr,
    prelude::*,
    symbolica::{parse, transcendental::TranscendentalFunctions},
    symbols,
};

#[test]
fn polynomials_and_precedence() {
    symbols!(x, y);
    assert_eq!(expr!(x ^ 2 + 2 * x + 1), parse!("x^2 + 2*x + 1"));
    assert_eq!(expr!(-x ^ 2), parse!("-(x^2)"));
    assert_eq!(expr!(2 ^ 3 ^ 2), parse!("2^(3^2)"));
    assert_eq!(expr!(x ^ -1), parse!("x^(-1)"));
    assert_eq!(expr!((x + 1) * (x - 1)), parse!("(x+1)*(x-1)"));
    assert_eq!(expr!(x / (y + 1)), parse!("x/(y+1)"));
    assert_eq!(expr!(--x), parse!("x"));
}

#[test]
fn exact_and_floating_coefficients() {
    symbols!(x);
    assert_eq!(expr!(1 / 3 * x), parse!("1/3*x"));
    assert_eq!(expr!(0.5 * x), Atom::num(0.5) * Atom::var(x));
    assert_eq!(expr!(1e3 * x), Atom::num(1000.0) * Atom::var(x));
    assert_eq!(
        expr!(170141183460469231731687303715884105727),
        Atom::num(i128::MAX)
    );
    assert_eq!(expr!(2u8 * x), parse!("2*x"));
}

#[test]
fn built_in_and_user_functions() {
    symbols!(x, y, f, g);
    assert_eq!(
        expr!(sin(x) ^ 2 + cos(x) ^ 2),
        parse!("sin(x)^2 + cos(x)^2")
    );
    assert_eq!(expr!(exp(log(x))), parse!("exp(log(x))"));
    assert_eq!(expr!(sqrt(x) * abs(y)), parse!("sqrt(x)*abs(y)"));
    assert_eq!(expr!(f(x, y)), parse!("f(x, y)"));
    assert_eq!(expr!(f(g(x), 2)), parse!("f(g(x), 2)"));
    assert_eq!(expr!(f()), parse!("f()"));
}

#[test]
fn splices_of_rust_values() {
    symbols!(x);
    let k = 3;
    let a = parse!("x + 1");
    let r = &a;
    assert_eq!(expr!(k * x), parse!("3*x"));
    assert_eq!(expr!({ k + 1 } * x), parse!("4*x"));
    assert_eq!(expr!(a ^ 2), parse!("(x+1)^2"));
    assert_eq!(expr!(r + 1), parse!("x + 2"));
    assert_eq!(a, parse!("x + 1"), "splicing borrows: `a` still usable");
}

#[test]
fn imaginary_unit_and_factorial() {
    symbols!(n);
    assert_eq!(expr!(I ^ 2), Atom::num(-1));
    assert_eq!(expr!(5!), Atom::num(120));
    assert_eq!(expr!(n!), (Atom::var(n) + 1).gamma());
    assert_eq!(expr!(3! ^ 2), Atom::num(36));
}

#[test]
fn normalisation_happens_in_symbolica_not_in_the_macro() {
    symbols!(x, f);
    assert_eq!(expr!(x ^ 2 * x ^ 3), parse!("x^5"));
    assert_eq!(expr!(2 * x + 3 * x), parse!("5*x"));
    assert_eq!(expr!(x - x), Atom::zero());
    assert_eq!(expr!(f(x) ^ 2 / f(x)), parse!("f(x)"));
    assert_eq!(expr!(-(-x)), parse!("x"));
    assert_eq!(expr!((x + 1)!), (Atom::var(x) + 2).gamma());
}

#[test]
fn floats_compare_numerically() {
    symbols!(x);
    let coefficient_of =
        |e: Atom| e.evaluate(&std::collections::HashMap::from([(Atom::var(x), 1.0_f64)]));
    assert_eq!(coefficient_of(expr!(0.1 * x + 0.2 * x)), Ok(0.1 + 0.2));
    assert_eq!(coefficient_of(expr!(2.5e-1 * x)), Ok(0.25));
    assert_eq!(coefficient_of(expr!(1 / 4 * x)), Ok(0.25));
}

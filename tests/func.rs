//! `func!` compiles an expression to a fast `f64` closure.

use rmath::{func, prelude::*};

#[test]
fn a_two_parameter_function_evaluates_positionally() {
    symbols!(x, y);
    let mut f = func!(|x, y| x ^ 2 + sin(y)).unwrap();
    assert_eq!(f(3.0, 0.0), 9.0);
    assert!((f(0.0, std::f64::consts::FRAC_PI_2) - 1.0).abs() < 1e-12);
}

#[test]
fn an_existing_atom_can_be_the_body() {
    symbols!(x, y);
    let e = expr!(x * y + 1);
    let mut g = func!(|x, y| e).unwrap();
    assert_eq!(g(2.0, 5.0), 11.0);
}

#[test]
fn captured_values_are_frozen_at_creation() {
    symbols!(x);
    let mut k = 2;
    let mut f = func!(|x| k * x).unwrap();
    k = 5;
    let _ = k;
    assert_eq!(f(1.0), 2.0);
}

#[test]
fn zero_and_many_parameters_work() {
    symbols!(a, b, c, d, e);
    let mut constant = func!(|| 7 / 2).unwrap();
    assert_eq!(constant(), 3.5);
    let mut sum = func!(|a, b, c, d, e| a + b + c + d + e).unwrap();
    assert_eq!(sum(1.0, 2.0, 3.0, 4.0, 5.0), 15.0);
}

#[test]
fn a_free_symbol_in_the_body_is_an_error_not_a_panic() {
    symbols!(x, y);
    assert!(func!(|x| x + y).is_err());
}

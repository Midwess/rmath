//! `solve!` writes equation systems with `==`.

use rmath::{
    prelude::*,
    solve,
    symbolica::{
        domains::rational::Rational,
        solve::{SolveDomain, SolveError},
    },
};

#[test]
fn a_linear_system_solves_to_a_point() {
    symbols!(x, y);
    let sols = solve!([2 * x + y == 3, x - y == 0] for x, y).unwrap();
    assert_eq!(sols.len(), 1);
    assert_eq!(sols[0].value(x), Some(&expr!(1)));
    assert_eq!(sols[0].value(y), Some(&expr!(1)));
}

#[test]
fn the_domain_clause_restricts_solutions() {
    symbols!(x);
    let over_reals = solve!([x ^ 2 + 1 == 0] for x over Reals).unwrap();
    assert!(over_reals.is_empty().unwrap());
    assert_eq!(over_reals.domain(), SolveDomain::Reals);

    let over_complexes = solve!([x ^ 2 + 1 == 0] for x).unwrap();
    assert!(!over_complexes.is_empty().unwrap());
}

#[test]
fn spliced_rust_values_appear_in_the_equations() {
    symbols!(x);
    let m = Rational::new(5, 2);
    let k = 5;
    let sols = solve!([k * x == m] for x).unwrap();
    assert_eq!(sols[0].value(x), Some(&expr!(1 / 2)));
}

#[test]
fn inexact_coefficients_are_rejected_by_symbolicas_exact_solver() {
    symbols!(x);
    let measured = 2.5_f64;
    let result = solve!([2 * x == measured] for x);
    assert!(
        matches!(result, Err(SolveError::UnsupportedProblem(_))),
        "{result:?}"
    );
}

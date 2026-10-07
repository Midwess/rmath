//! `rule!` builds rewrite rules in `match`-like syntax.

use rmath::{prelude::*, rule, symbolica::parse};

#[test]
fn a_single_rule_swaps_arguments() {
    symbols!(f, g, x, y);
    let swap = rule!(f(a_, b_) => g(b_, a_));
    let e = expr!(f(1, x) + f(y, 2));
    assert_eq!(e.apply(&swap), parse!("g(x, 1) + g(2, y)"));
}

#[test]
fn a_guard_filters_matches_with_the_wildcards_bound_as_atoms() {
    symbols!(f);
    let drop_non_ones = rule!(f(a_) => 0, if a_ != expr!(1));
    assert_eq!(
        expr!(f(1) + f(2) + f(3)).apply(&drop_non_ones),
        parse!("f(1)")
    );
}

#[test]
fn a_multi_argument_wildcard_matches_any_number_of_arguments() {
    symbols!(g);
    let kill = rule!(g(a__) => 0);
    assert_eq!(expr!(g(1, 2, 3) + g(1) + 7).apply(&kill), parse!("7"));
}

#[test]
fn a_rule_set_is_applied_in_one_pass_and_rules_can_be_chained() {
    symbols!(f, g, h, x, y);
    let set = rule! {
        f(a_) => a_,
        g(a_) => 2 * a_,
    };
    assert_eq!(expr!(f(x) + g(y)).apply(&set), parse!("x + 2*y"));

    let to_h = rule!(g(a_) => h(a_));
    let to_f = rule!(h(a_) => f(a_));
    assert_eq!(expr!(g(x)).apply(&to_h).apply(&to_f), parse!("f(x)"));
}

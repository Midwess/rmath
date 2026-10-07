//! `find!` yields typed matches: one field per wildcard, named after it.

use rmath::{find, prelude::*};

#[test]
fn fields_are_named_after_the_wildcards() {
    symbols!(f, x, y);
    let e = expr!(f(1, 2) + f(x, y));
    let mut seen: Vec<String> = find!(e, f(a_, b_))
        .map(|m| format!("{},{}", m.a_, m.b_))
        .collect();
    seen.sort();
    assert_eq!(seen, ["1,2", "x,y"]);
}

#[test]
fn a_pattern_without_wildcards_yields_one_empty_item_per_occurrence() {
    symbols!(f, x);
    let e = expr!(f(1) + x * f(1) + 3);
    assert_eq!(find!(e, f(1)).count(), 2);
    assert_eq!(find!(e, f(2)).count(), 0);
}

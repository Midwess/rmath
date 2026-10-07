//! `symbols!` declares compiler-visible bindings for Symbolica symbols.

// rustc warns when a crate's only Greek identifiers look like Latin ones; `θ` below is on
// purpose. Users writing single Greek symbols will want this too.
#![allow(mixed_script_confusables)]

use rmath::{expr, symbolica::parse, symbols};

#[test]
fn declared_symbols_are_visible_after_the_macro_and_match_the_parser() {
    symbols!(x, y);
    assert_eq!(expr!(x + y), parse!("x + y"));
}

#[test]
fn attributes_are_written_like_trait_bounds() {
    symbols!(x, y, f: Symmetric, t: Real + Positive);
    // A symmetric function normalises its arguments into canonical order.
    assert_eq!(expr!(f(y, x)), expr!(f(x, y)));
    assert_eq!(t, rmath::symbolica::symbol!("t"; Real, Positive));
}

#[test]
fn a_display_name_may_differ_from_the_rust_name() {
    symbols!(tau0 = "τ_0", kappa = "κ": Positive);
    assert_eq!(expr!(tau0 + kappa), parse!("τ_0 + κ"));
}

#[test]
fn unicode_and_raw_identifiers_are_valid_names() {
    symbols!(θ, r#type);
    assert_eq!(expr!(θ + r#type), parse!("θ + type"));
}

#[test]
fn the_same_name_declared_in_two_scopes_is_one_symbol() {
    fn first() -> rmath::prelude::Symbol {
        symbols!(shared);
        shared
    }
    fn second() -> rmath::prelude::Symbol {
        symbols!(shared);
        shared
    }
    assert_eq!(first(), second());
}

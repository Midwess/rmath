//! First end-to-end check of `expr!`: the expansion must agree with Symbolica's own parser.

use rmath::{
    expr,
    symbolica::{parse, symbol},
};

#[test]
fn expr_agrees_with_symbolicas_parser_on_a_polynomial() {
    let x = symbol!("x");
    let _ = &x;
    assert_eq!(expr!(x ^ 2 + 1), parse!("x^2 + 1"));
}

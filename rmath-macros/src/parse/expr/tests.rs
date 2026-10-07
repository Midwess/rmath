use super::*;
use proc_macro2::TokenStream;
use std::str::FromStr;

/// Parse `src` and render the AST as an S-expression.
fn parse(src: &str) -> String {
    let mut c = Cursor::new(TokenStream::from_str(src).unwrap());
    parse_all(&mut c, Mode::Expr).unwrap().to_string()
}

/// Parse `src` expecting failure; return the rendered `compile_error!` text.
fn error(src: &str) -> String {
    let mut c = Cursor::new(TokenStream::from_str(src).unwrap());
    parse_all(&mut c, Mode::Expr)
        .map(|e| e.to_string())
        .expect_err("expected a parse error")
        .to_compile_error()
        .to_string()
}

#[test]
fn addition_of_identifier_and_literal() {
    assert_eq!(parse("x + 1"), "(+ x 1)");
}

#[test]
fn additive_and_multiplicative_operators_are_left_associative_with_math_precedence() {
    assert_eq!(parse("a - b - c"), "(- (- a b) c)");
    assert_eq!(parse("a / b * c"), "(* (/ a b) c)");
    assert_eq!(parse("a + b * c"), "(+ a (* b c))");
    assert_eq!(parse("a * b - c / d"), "(- (* a b) (/ c d))");
}

#[test]
fn power_is_right_associative_and_binds_tighter_than_multiplication() {
    assert_eq!(parse("2 ^ 3 ^ 2"), "(^ 2 (^ 3 2))");
    assert_eq!(parse("a * b ^ c"), "(* a (^ b c))");
    assert_eq!(parse("a ^ b * c"), "(* (^ a b) c)");
}

#[test]
fn unary_minus_binds_looser_than_power_but_tighter_than_multiplication() {
    assert_eq!(parse("-x ^ 2"), "(neg (^ x 2))");
    assert_eq!(parse("-a * b"), "(* (neg a) b)");
    assert_eq!(parse("x ^ -1"), "(^ x (neg 1))");
    assert_eq!(parse("2 ^ -x * 3"), "(* (^ 2 (neg x)) 3)");
    assert_eq!(parse("a - -b"), "(- a (neg b))");
}

#[test]
fn postfix_factorial_binds_tightest() {
    assert_eq!(parse("n! ^ 2"), "(^ (! n) 2)");
    assert_eq!(parse("-n!"), "(neg (! n))");
    assert_eq!(parse("n! * 2"), "(* (! n) 2)");
    assert_eq!(parse("2 ^ n!"), "(^ 2 (! n))");
}

#[test]
fn parentheses_group_without_adding_a_node() {
    assert_eq!(parse("(a + b) * c"), "(* (+ a b) c)");
    assert_eq!(parse("((x))"), "x");
    assert_eq!(parse("-(a + b)"), "(neg (+ a b))");
    assert_eq!(parse("(a + b)!"), "(! (+ a b))");
}

#[test]
fn identifier_followed_by_parentheses_is_a_call() {
    assert_eq!(parse("f(x, y)"), "(call f x y)");
    assert_eq!(parse("f()"), "(call f)");
    assert_eq!(parse("f(a + b, g(c))"), "(call f (+ a b) (call g c))");
    assert_eq!(parse("sin(x) ^ 2"), "(^ (call sin x) 2)");
}

#[test]
fn brace_block_is_a_rust_splice() {
    assert_eq!(parse("x + {k + 1}"), "(+ x {...})");
    assert_eq!(parse("{a} * {self.k}"), "(* {...} {...})");
}

#[test]
fn adjacent_operands_are_implicit_multiplication() {
    for (src, help) in [
        ("x y", "insert `*` between `x` and `y`"),
        ("2 (x)", "insert `*` between `2` and `(x)`"),
        ("(a) (b)", "insert `*` between `(a)` and `(b)`"),
        ("x ^ 2 y", "insert `*` between `2` and `y`"),
        ("x {k}", "insert `*` between `x` and `{ k }`"),
    ] {
        let msg = error(src);
        assert!(
            msg.contains("implicit multiplication is not supported"),
            "{src}: {msg}"
        );
        assert!(msg.contains(help), "{src}: {msg}");
    }
}

#[test]
fn keywords_end_the_expression_instead_of_being_operands() {
    let mut c = Cursor::new(TokenStream::from_str("x + 1 for x").unwrap());
    assert_eq!(
        parse_expr(&mut c, Mode::Expr).unwrap().to_string(),
        "(+ x 1)"
    );
    assert_eq!(c.peek().unwrap().to_string(), "for");
}

#[test]
fn macro_call_syntax_is_rejected_with_a_splice_hint() {
    let msg = error("f!(x) + 1");
    assert!(
        msg.contains("macro-call syntax is not a math expression"),
        "{msg}"
    );
    assert!(msg.contains("wrap it in braces: `{ f!(x) }`"), "{msg}");
}

#[test]
fn single_equals_points_at_double_equals() {
    let msg = error("x = 1");
    assert!(msg.contains("`=` is not an operator"), "{msg}");
    assert!(msg.contains("help: use `==` to write an equation"), "{msg}");
}

#[test]
fn paths_and_member_access_point_at_splicing() {
    let msg = error("a::b + 1");
    assert!(msg.contains("paths are not supported"), "{msg}");
    assert!(msg.contains("wrap the Rust expression in braces"), "{msg}");

    let msg = error("x.y * 2");
    assert!(
        msg.contains("method calls and field access are not supported"),
        "{msg}"
    );
    assert!(msg.contains("wrap the Rust expression in braces"), "{msg}");
}

#[test]
fn missing_operands_are_reported_where_they_were_expected() {
    assert!(error("x +").contains("expected an expression after `+`"));
    assert!(error("x * -").contains("expected an expression after `-`"));
    assert!(error("").contains("expected an expression"));
    assert!(error("f(,)").contains("empty argument in function call"));
    assert!(error("f(x,,y)").contains("empty argument in function call"));
    assert!(error("()").contains("expected an expression inside the parentheses"));
}

/// Parse `src` in pattern mode and render the AST.
fn pattern(src: &str) -> String {
    let mut c = Cursor::new(TokenStream::from_str(src).unwrap());
    parse_all(&mut c, Mode::Pattern).unwrap().to_string()
}

#[test]
fn trailing_underscores_denote_wildcards_only_in_pattern_mode() {
    assert_eq!(
        pattern("f(a_, b__, c___)"),
        "(call f (wc a_) (wc b__) (wc c___))"
    );
    assert_eq!(pattern("x_ + 1"), "(+ (wc x_) 1)");
    assert_eq!(
        parse("a_ + 1"),
        "(+ a_ 1)",
        "in expr mode `a_` is an ordinary Rust name"
    );
}

#[test]
fn more_than_three_trailing_underscores_is_an_error_in_pattern_mode() {
    let mut c = Cursor::new(TokenStream::from_str("a____").unwrap());
    let msg = parse_all(&mut c, Mode::Pattern)
        .map(|e| e.to_string())
        .expect_err("expected a parse error")
        .to_compile_error()
        .to_string();
    assert!(msg.contains("at most three trailing underscores"), "{msg}");
}

#[test]
fn a_lone_underscore_is_not_a_name() {
    let msg = error("x + _");
    assert!(msg.contains("`_` is not a valid name"), "{msg}");
    assert!(msg.contains("wildcards need a prefix, like `a_`"), "{msg}");
}

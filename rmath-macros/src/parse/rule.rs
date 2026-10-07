//! Input grammar of `rule!`: `lhs => rhs [, if guard]`, or a brace block of such arms.

use proc_macro2::{Delimiter, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use super::{
    ast::{Expr, ExprKind},
    cursor::Cursor,
    error::{Error, Errors},
    expr::{Mode, parse_expr, wildcard_arity},
};

/// One rewrite arm.
pub struct RuleArm {
    pub lhs: Expr,
    pub rhs: Expr,
    /// The raw tokens of the `if` guard, if any.
    pub guard: Option<TokenStream>,
    /// Every wildcard bound by `lhs`, in first-appearance order, without duplicates.
    pub wildcards: Vec<Ident>,
}

/// Parse one or more arms, reporting every arm's error at once.
///
/// `rule! { a => b, c => d }` delivers `a => b, c => d` (the braces are the invocation's own
/// delimiters), while `rule!({ a => b })` delivers a brace group; both forms are accepted.
pub fn parse_rules(body: TokenStream) -> Result<Vec<RuleArm>, Errors> {
    let arms_tokens = split_arms(single_brace_block(&body).unwrap_or(body));
    let mut errors = Errors::default();
    let mut arms = Vec::new();
    for tokens in arms_tokens {
        match parse_arm(tokens) {
            Ok(arm) => arms.push(arm),
            Err(err) => errors.push(err),
        }
    }
    if errors.is_empty() {
        Ok(arms)
    } else {
        Err(errors)
    }
}

/// `{ ... }` as the whole input: the rule-set form.
fn single_brace_block(body: &TokenStream) -> Option<TokenStream> {
    let mut iter = body.clone().into_iter();
    match (iter.next(), iter.next()) {
        (Some(TokenTree::Group(g)), None) if g.delimiter() == Delimiter::Brace => Some(g.stream()),
        _ => None,
    }
}

/// Split a rule set on top-level commas, but keep `, if guard` with the arm it belongs to.
fn split_arms(inner: TokenStream) -> Vec<TokenStream> {
    let mut arms: Vec<TokenStream> = Vec::new();
    for piece in Cursor::split_top_level(inner, ',') {
        let starts_with_if = matches!(piece.clone().into_iter().next(),
            Some(TokenTree::Ident(id)) if id == "if");
        match arms.last_mut() {
            Some(prev) if starts_with_if => {
                prev.extend(std::iter::once(TokenTree::Punct(Punct::new(
                    ',',
                    Spacing::Alone,
                ))));
                prev.extend(piece);
            }
            _ => arms.push(piece),
        }
    }
    arms
}

fn parse_arm(tokens: TokenStream) -> Result<RuleArm, Error> {
    let mut c = Cursor::new(tokens);
    if c.peek().is_none() {
        return Err(Error::new(
            Span::call_site(),
            "expected a rule `lhs => rhs`",
        ));
    }
    let lhs = parse_expr(&mut c, Mode::Pattern)?;
    if !c.peek_op("=>") {
        return Err(match c.peek() {
            Some(tok) => Error::new(tok.span(), format!("expected `=>`, found `{tok}`")),
            None => Error::new(
                c.last_span().unwrap_or_else(Span::call_site),
                "expected `=>` after the pattern",
            ),
        }
        .with_help("a rule is written `pattern => replacement`"));
    }
    c.bump();
    c.bump();
    let rhs = parse_expr(&mut c, Mode::Pattern)?;

    let guard = if c.peek_op(",") {
        c.bump();
        match c.bump() {
            Some(TokenTree::Ident(kw)) if kw == "if" => {}
            Some(other) => {
                return Err(Error::new(
                    other.span(),
                    format!("expected `if` after the comma, found `{other}`"),
                ));
            }
            None => {
                return Err(Error::new(
                    c.last_span().unwrap_or_else(Span::call_site),
                    "expected `if <condition>` after the comma",
                ));
            }
        }
        let tokens: TokenStream = std::iter::from_fn(|| c.bump()).collect();
        if tokens.is_empty() {
            return Err(Error::new(
                c.last_span().unwrap_or_else(Span::call_site),
                "expected a condition after `if`",
            ));
        }
        Some(tokens)
    } else {
        None
    };
    if let Some(tok) = c.peek() {
        return Err(Error::new(
            tok.span(),
            format!("unexpected `{tok}` after the replacement"),
        ));
    }

    let mut wildcards = Vec::new();
    collect_wildcards(&lhs, &mut wildcards);

    let mut on_rhs = Vec::new();
    collect_wildcards(&rhs, &mut on_rhs);
    if let Some(unbound) = on_rhs.iter().find(|w| !wildcards.contains(w)) {
        return Err(Error::new(
            unbound.span(),
            format!("wildcard `{unbound}` does not appear on the left-hand side"),
        )
        .with_help("every wildcard in the replacement must be bound by the pattern"));
    }
    if let Some(tokens) = &guard
        && let Some(unbound) = unbound_wildcard_in_guard(tokens.clone(), &wildcards)
    {
        return Err(Error::new(
            unbound.span(),
            format!("wildcard `{unbound}` is not bound by the pattern"),
        )
        .with_help("a guard may only refer to wildcards that appear in the pattern"));
    }

    Ok(RuleArm {
        lhs,
        rhs,
        guard,
        wildcards,
    })
}

/// The first wildcard-shaped identifier in the guard that the pattern does not bind.
fn unbound_wildcard_in_guard(tokens: TokenStream, bound: &[Ident]) -> Option<Ident> {
    for tt in tokens {
        match tt {
            TokenTree::Ident(id)
                if matches!(wildcard_arity(&id), Ok(Some(_))) && !bound.contains(&id) =>
            {
                return Some(id);
            }
            TokenTree::Group(g) => {
                if let Some(found) = unbound_wildcard_in_guard(g.stream(), bound) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

/// Wildcards in first-appearance order; a function head like `f_(x)` is a wildcard too.
fn collect_wildcards(expr: &Expr, out: &mut Vec<Ident>) {
    let mut push = |id: &Ident| {
        if !out.iter().any(|w| w == id) {
            out.push(id.clone());
        }
    };
    match &expr.kind {
        ExprKind::Wildcard { ident, .. } => push(ident),
        ExprKind::Call { func, args } => {
            if matches!(wildcard_arity(func), Ok(Some(_))) {
                push(func);
            }
            for a in args {
                collect_wildcards(a, out);
            }
        }
        ExprKind::Neg(e) | ExprKind::Factorial(e) => collect_wildcards(e, out),
        ExprKind::Binary { lhs, rhs, .. } => {
            collect_wildcards(lhs, out);
            collect_wildcards(rhs, out);
        }
        ExprKind::Num(_) | ExprKind::Ident(_) | ExprKind::Splice(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn ts(src: &str) -> TokenStream {
        TokenStream::from_str(src).unwrap()
    }

    fn names(arm: &RuleArm) -> Vec<String> {
        arm.wildcards.iter().map(ToString::to_string).collect()
    }

    fn error(src: &str) -> String {
        parse_rules(ts(src))
            .map(|_| ())
            .expect_err("expected a parse error")
            .to_compile_error()
            .to_string()
    }

    #[test]
    fn single_arm_splits_lhs_and_rhs_and_collects_wildcards_in_order() {
        let arms = parse_rules(ts("f(a_, b_) => g(b_, a_)")).unwrap();
        assert_eq!(arms.len(), 1);
        assert_eq!(arms[0].lhs.to_string(), "(call f (wc a_) (wc b_))");
        assert_eq!(arms[0].rhs.to_string(), "(call g (wc b_) (wc a_))");
        assert!(arms[0].guard.is_none());
        assert_eq!(names(&arms[0]), ["a_", "b_"]);
    }

    #[test]
    fn a_brace_block_holds_several_arms_and_a_guard_is_kept_verbatim() {
        let arms = parse_rules(ts("{ f(a_) => a_ ^ 2, if a_ != 1, g(a__) => 0 }")).unwrap();
        assert_eq!(arms.len(), 2);
        assert_eq!(arms[0].guard.as_ref().unwrap().to_string(), "a_ != 1");
        assert_eq!(arms[1].rhs.to_string(), "0");
        assert_eq!(names(&arms[1]), ["a__"]);
    }

    #[test]
    fn arms_arrive_without_braces_when_the_macro_itself_uses_braces() {
        // `rule! { f(a_) => a_, g(a_) => 2 * a_, }` hands the expander the inner tokens only.
        let arms = parse_rules(ts("f(a_) => a_, g(a_) => 2 * a_,")).unwrap();
        assert_eq!(arms.len(), 2);
        assert_eq!(arms[1].rhs.to_string(), "(* 2 (wc a_))");
    }

    #[test]
    fn a_missing_arrow_is_reported_with_the_rule_shape() {
        let msg = error("f(a_) + 1");
        assert!(msg.contains("expected `=>`"), "{msg}");
        assert!(msg.contains("pattern => replacement"), "{msg}");
    }

    #[test]
    fn wildcards_used_on_the_right_must_be_bound_on_the_left() {
        let msg = error("f(a_) => b_");
        assert!(
            msg.contains("wildcard `b_` does not appear on the left-hand side"),
            "{msg}"
        );

        let msg = error("f(a_) => a_, if b_ != 0");
        assert!(
            msg.contains("wildcard `b_` is not bound by the pattern"),
            "{msg}"
        );
    }
}

//! Input grammar of `find!`: `target, pattern`.

use proc_macro2::{Ident, Span, TokenStream};

use super::{
    ast::Expr,
    cursor::Cursor,
    error::Error,
    expr::{Mode, parse_all},
    rule::collect_wildcards,
};

pub struct FindInput {
    /// The Rust expression to search, spliced through `IntoExpr`.
    pub target: TokenStream,
    pub pattern: Expr,
    /// The pattern's wildcards in first-appearance order: one struct field each.
    pub wildcards: Vec<Ident>,
}

pub fn parse_find(body: TokenStream) -> Result<FindInput, Error> {
    let pieces = Cursor::split_top_level(body, ',');
    let [target, pattern_tokens] = <[TokenStream; 2]>::try_from(pieces).map_err(|pieces| {
        let span = pieces
            .first()
            .and_then(|p| p.clone().into_iter().next())
            .map_or_else(Span::call_site, |t| t.span());
        Error::new(span, "expected `find!(expression, pattern)`")
            .with_help("the pattern may use wildcards: `find!(e, f(a_, b_))`")
    })?;
    if target.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "expected an expression to search before the comma",
        ));
    }
    let mut c = Cursor::new(pattern_tokens);
    let pattern = parse_all(&mut c, Mode::Pattern)?;
    let mut wildcards = Vec::new();
    collect_wildcards(&pattern, &mut wildcards);
    Ok(FindInput {
        target,
        pattern,
        wildcards,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn target_tokens_pattern_and_wildcards_are_separated() {
        let input = parse_find(TokenStream::from_str("e, f(a_, b_)").unwrap()).unwrap();
        assert_eq!(input.target.to_string(), "e");
        assert_eq!(input.pattern.to_string(), "(call f (wc a_) (wc b_))");
        let names: Vec<String> = input.wildcards.iter().map(ToString::to_string).collect();
        assert_eq!(names, ["a_", "b_"]);
    }

    #[test]
    fn the_wrong_number_of_pieces_is_an_error() {
        let msg = parse_find(TokenStream::from_str("e").unwrap())
            .map(|_| ())
            .expect_err("expected a parse error")
            .to_compile_error()
            .to_string();
        assert!(
            msg.contains("expected `find!(expression, pattern)`"),
            "{msg}"
        );
    }
}

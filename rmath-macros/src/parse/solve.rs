//! Input grammar of `solve!`: `[lhs == rhs, ..] for v1, v2 [over Domain]`.

use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};

use super::{
    ast::Expr,
    cursor::Cursor,
    error::{Error, Errors},
    expr::{Mode, parse_all, parse_expr},
};

pub struct SolveInput {
    /// Each equation as `(lhs, rhs)`; the expander lowers it to `lhs - rhs`.
    pub equations: Vec<(Expr, Expr)>,
    pub unknowns: Vec<Ident>,
    /// `Complexes`, `Reals`, `Rationals` or `Integers`, if an `over` clause is present.
    pub domain: Option<Ident>,
}

/// Symbolica's `SolveDomain` variants.
const DOMAINS: &[&str] = &["Complexes", "Reals", "Rationals", "Integers"];

pub fn parse_solve(body: TokenStream) -> Result<SolveInput, Errors> {
    let mut c = Cursor::new(body);

    // `[eq, eq]` or a single bare equation up to `for`.
    let equation_tokens = match c.peek() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
            let inner = g.stream();
            c.bump();
            Cursor::split_top_level(inner, ',')
        }
        Some(_) => {
            let mut tokens = TokenStream::new();
            while let Some(tok) = c.peek() {
                if matches!(tok, TokenTree::Ident(id) if id == "for") {
                    break;
                }
                tokens.extend(c.bump());
            }
            vec![tokens]
        }
        None => {
            return Err(Error::new(
                Span::call_site(),
                "expected `solve!([equations] for unknowns)`",
            )
            .into());
        }
    };

    let mut errors = Errors::default();
    let mut equations = Vec::new();
    for tokens in equation_tokens {
        match parse_equation(tokens) {
            Ok(eq) => equations.push(eq),
            Err(err) => errors.push(err),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    match c.bump() {
        Some(TokenTree::Ident(kw)) if kw == "for" => {}
        Some(other) => {
            return Err(Error::new(
                other.span(),
                format!("expected `for` followed by the unknowns, found `{other}`"),
            )
            .into());
        }
        None => {
            return Err(Error::new(
                c.last_span().unwrap_or_else(Span::call_site),
                "expected `for` followed by the unknowns",
            )
            .with_help("write `solve!([equations] for x, y)`")
            .into());
        }
    }

    let mut unknowns = vec![unknown(&mut c)?];
    while c.peek_op(",") {
        c.bump();
        unknowns.push(unknown(&mut c)?);
    }

    let domain = match c.bump() {
        None => None,
        Some(TokenTree::Ident(kw)) if kw == "over" => Some(domain(&mut c)?),
        Some(other) => {
            return Err(Error::new(
                other.span(),
                format!("unexpected `{other}` after the unknowns"),
            )
            .with_help("an optional domain follows: `over Reals`")
            .into());
        }
    };
    if let Some(tok) = c.peek() {
        return Err(Error::new(tok.span(), format!("unexpected `{tok}` after the domain")).into());
    }

    Ok(SolveInput {
        equations,
        unknowns,
        domain,
    })
}

/// `lhs == rhs`.
fn parse_equation(tokens: TokenStream) -> Result<(Expr, Expr), Error> {
    let mut c = Cursor::new(tokens);
    if c.peek().is_none() {
        return Err(Error::new(Span::call_site(), "empty equation"));
    }
    let lhs = parse_expr(&mut c, Mode::Expr)?;
    if c.peek_op("=") {
        let eq = c.peek().expect("peeked");
        return Err(Error::new(eq.span(), "`=` is not an operator")
            .with_help("write the equation as `lhs == rhs`"));
    }
    if !c.peek_op("==") {
        return Err(match c.peek() {
            Some(tok) => Error::new(tok.span(), format!("expected `==`, found `{tok}`")),
            None => Error::new(
                c.last_span().unwrap_or_else(Span::call_site),
                "expected `lhs == rhs`",
            ),
        }
        .with_help("every equation is written `lhs == rhs`; an expression alone is not accepted"));
    }
    c.bump();
    c.bump();
    let rhs = parse_all(&mut c, Mode::Expr)?;
    Ok((lhs, rhs))
}

fn unknown(c: &mut Cursor) -> Result<Ident, Error> {
    match c.bump() {
        Some(TokenTree::Ident(id)) if id != "over" => Ok(id),
        Some(other) => Err(Error::new(
            other.span(),
            format!("expected an unknown (a symbol name), found `{other}`"),
        )),
        None => Err(Error::new(
            c.last_span().unwrap_or_else(Span::call_site),
            "expected an unknown (a symbol name)",
        )),
    }
}

fn domain(c: &mut Cursor) -> Result<Ident, Error> {
    match c.bump() {
        Some(TokenTree::Ident(id)) if DOMAINS.contains(&id.to_string().as_str()) => Ok(id),
        Some(TokenTree::Ident(id)) => Err(Error::new(id.span(), format!("unknown domain `{id}`"))
            .with_help(format!("supported domains: {}", DOMAINS.join(", ")))),
        Some(other) => Err(Error::new(
            other.span(),
            format!("expected a domain name after `over`, found `{other}`"),
        )),
        None => Err(Error::new(
            c.last_span().unwrap_or_else(Span::call_site),
            "expected a domain name after `over`",
        )
        .with_help(format!("supported domains: {}", DOMAINS.join(", ")))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn parse(src: &str) -> SolveInput {
        parse_solve(TokenStream::from_str(src).unwrap()).unwrap()
    }

    fn error(src: &str) -> String {
        parse_solve(TokenStream::from_str(src).unwrap())
            .map(|_| ())
            .expect_err("expected a parse error")
            .to_compile_error()
            .to_string()
    }

    #[test]
    fn equations_unknowns_and_domain_are_separated() {
        let input = parse("[2 * x + y == 3, x - y == 0] for x, y");
        let eqs: Vec<String> = input
            .equations
            .iter()
            .map(|(l, r)| format!("{l} == {r}"))
            .collect();
        assert_eq!(eqs, ["(+ (* 2 x) y) == 3", "(- x y) == 0"]);
        let unknowns: Vec<String> = input.unknowns.iter().map(ToString::to_string).collect();
        assert_eq!(unknowns, ["x", "y"]);
        assert!(input.domain.is_none());

        let input = parse("[x ^ 2 == 2] for x over Reals");
        assert_eq!(input.domain.unwrap().to_string(), "Reals");
    }

    #[test]
    fn a_single_equation_needs_no_brackets() {
        let input = parse("x ^ 2 == 4 for x");
        assert_eq!(input.equations.len(), 1);
    }

    #[test]
    fn malformed_input_is_explained() {
        assert!(error("[x == 1]").contains("expected `for`"));
        assert!(error("[x = 1] for x").contains("`=` is not an operator"));
        assert!(error("[x ^ 2 - 1] for x").contains("expected `lhs == rhs`"));
        let msg = error("[x == 1] for x over Quaternions");
        assert!(msg.contains("unknown domain `Quaternions`"), "{msg}");
        assert!(
            msg.contains("Complexes, Reals, Rationals, Integers"),
            "{msg}"
        );
    }
}

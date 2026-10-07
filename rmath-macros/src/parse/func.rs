//! Input grammar of `func!`: `|p1, p2, ..| body` (or `|| body`).

use proc_macro2::{Ident, Span, TokenStream, TokenTree};

use super::{
    ast::Expr,
    cursor::Cursor,
    error::Error,
    expr::{Mode, parse_all},
};

pub struct FuncInput {
    /// Parameter names, in order; each must be a `Symbol` in scope at the call site.
    pub params: Vec<Ident>,
    pub body: Expr,
}

pub fn parse_func(body: TokenStream) -> Result<FuncInput, Error> {
    let mut c = Cursor::new(body);
    let params = if c.peek_op("||") {
        c.bump();
        c.bump();
        Vec::new()
    } else if c.peek_op("|") {
        c.bump();
        parameters(&mut c)?
    } else {
        let span = c.peek().map_or_else(Span::call_site, TokenTree::span);
        return Err(Error::new(span, "expected `|parameters| body`")
            .with_help("write `func!(|x, y| x^2 + sin(y))`; the parameters are symbols in scope"));
    };
    if c.peek().is_none() {
        return Err(Error::new(
            c.last_span().unwrap_or_else(Span::call_site),
            "expected the function body after the parameter list",
        ));
    }
    let body = parse_all(&mut c, Mode::Expr)?;
    Ok(FuncInput { params, body })
}

/// `p1, p2, ..|` after the opening `|`.
fn parameters(c: &mut Cursor) -> Result<Vec<Ident>, Error> {
    let mut params: Vec<Ident> = Vec::new();
    loop {
        let param = match c.bump() {
            Some(TokenTree::Ident(id)) => id,
            Some(other) => {
                return Err(Error::new(
                    other.span(),
                    format!("expected a parameter name, found `{other}`"),
                ));
            }
            None => {
                return Err(Error::new(
                    c.last_span().unwrap_or_else(Span::call_site),
                    "unterminated parameter list; expected `|`",
                ));
            }
        };
        if c.peek_op(":") {
            let colon = c.peek().expect("peeked");
            return Err(Error::new(
                colon.span(),
                "parameters are symbol names only; types are not accepted here",
            )
            .with_help(format!(
                "write `|{param}|`: the compiled function always takes `f64`s"
            )));
        }
        if params.contains(&param) {
            return Err(Error::new(
                param.span(),
                format!("duplicate parameter `{param}`"),
            ));
        }
        params.push(param);
        if c.peek_op(",") {
            c.bump();
            continue;
        }
        if c.peek_op("|") {
            c.bump();
            return Ok(params);
        }
        return Err(match c.peek() {
            Some(tok) => Error::new(
                tok.span(),
                format!("expected `,` or `|` after the parameter, found `{tok}`"),
            ),
            None => Error::new(
                c.last_span().unwrap_or_else(Span::call_site),
                "unterminated parameter list; expected `|`",
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn parse(src: &str) -> FuncInput {
        parse_func(TokenStream::from_str(src).unwrap()).unwrap()
    }

    fn error(src: &str) -> String {
        parse_func(TokenStream::from_str(src).unwrap())
            .map(|_| ())
            .expect_err("expected a parse error")
            .to_compile_error()
            .to_string()
    }

    #[test]
    fn parameters_and_body_are_separated() {
        let input = parse("|x, y| x ^ 2 + sin(y)");
        let params: Vec<String> = input.params.iter().map(ToString::to_string).collect();
        assert_eq!(params, ["x", "y"]);
        assert_eq!(input.body.to_string(), "(+ (^ x 2) (call sin y))");

        let input = parse("|| 42");
        assert!(input.params.is_empty());
        assert_eq!(input.body.to_string(), "42");
    }

    #[test]
    fn typed_and_duplicate_parameters_are_rejected() {
        let msg = error("|x: f64| x");
        assert!(msg.contains("parameters are symbol names only"), "{msg}");
        let msg = error("|x, x| x");
        assert!(msg.contains("duplicate parameter `x`"), "{msg}");
        let msg = error("x + 1");
        assert!(msg.contains("expected `|parameters| body`"), "{msg}");
    }
}

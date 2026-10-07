//! Input grammar of `symbols!`: a comma-separated list of declarations.

use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};

use super::{
    cursor::Cursor,
    error::{Error, Errors},
};

/// One declaration: `x`, `t: Real + Positive`, or `tau0 = "τ_0"`.
pub struct SymbolDecl {
    pub ident: Ident,
    pub attrs: Vec<Ident>,
    pub name: Option<Literal>,
}

/// Symbolica's `SymbolAttribute` variants, in the order the error message lists them.
const ATTRIBUTES: &[&str] = &[
    "Symmetric",
    "Antisymmetric",
    "Cyclesymmetric",
    "Linear",
    "Flat",
    "Scalar",
    "Real",
    "Integer",
    "Positive",
];

/// Parse every declaration, reporting all errors at once.
pub fn parse_decls(body: TokenStream) -> Result<Vec<SymbolDecl>, Errors> {
    let mut errors = Errors::default();
    let mut decls = Vec::new();
    for piece in Cursor::split_top_level(body, ',') {
        match parse_decl(piece) {
            Ok(decl) => decls.push(decl),
            Err(err) => errors.push(err),
        }
    }
    if errors.is_empty() {
        Ok(decls)
    } else {
        Err(errors)
    }
}

fn parse_decl(piece: TokenStream) -> Result<SymbolDecl, Error> {
    let mut c = Cursor::new(piece);
    let ident = match c.bump() {
        Some(TokenTree::Ident(id)) => id,
        Some(other) => {
            return Err(Error::new(
                other.span(),
                format!("expected a symbol name, found `{other}`"),
            ));
        }
        None => return Err(Error::new(Span::call_site(), "empty symbol declaration")),
    };
    if ident.to_string().ends_with('_') {
        return Err(Error::new(
            ident.span(),
            format!("symbol name `{ident}` ends in `_`, which Symbolica reserves for wildcards"),
        )
        .with_help(
            "rename the symbol; wildcards are declared implicitly inside `rule!` and `find!`",
        ));
    }
    let name = if c.peek_op("=") {
        c.bump();
        Some(display_name(&mut c)?)
    } else {
        None
    };
    let mut attrs = Vec::new();
    if c.peek_op(":") {
        c.bump();
        attrs.push(attribute(&mut c)?);
        while c.peek_op("+") {
            c.bump();
            attrs.push(attribute(&mut c)?);
        }
    }
    if let Some(tok) = c.peek() {
        return Err(Error::new(
            tok.span(),
            format!("unexpected `{tok}` in the symbol declaration"),
        ));
    }
    Ok(SymbolDecl { ident, attrs, name })
}

/// One attribute name after `:` or `+`.
fn attribute(c: &mut Cursor) -> Result<Ident, Error> {
    match c.bump() {
        Some(TokenTree::Ident(id)) if ATTRIBUTES.contains(&id.to_string().as_str()) => Ok(id),
        Some(TokenTree::Ident(id)) => {
            Err(Error::new(id.span(), format!("unknown attribute `{id}`"))
                .with_help(format!("supported attributes: {}", ATTRIBUTES.join(", "))))
        }
        Some(other) => Err(Error::new(
            other.span(),
            format!("expected an attribute name, found `{other}`"),
        )),
        None => Err(Error::new(
            c.last_span().unwrap_or_else(Span::call_site),
            "expected an attribute name",
        )),
    }
}

/// The string literal after `=`: the symbol's name as Symbolica sees it.
fn display_name(c: &mut Cursor) -> Result<Literal, Error> {
    match c.bump() {
        Some(TokenTree::Literal(lit)) if matches!(syn::Lit::new(lit.clone()), syn::Lit::Str(_)) => {
            Ok(lit)
        }
        Some(other) => Err(Error::new(
            other.span(),
            format!("expected a string literal for the display name, found `{other}`"),
        )
        .with_help("write `name = \"display name\"`")),
        None => Err(Error::new(
            c.last_span().unwrap_or_else(Span::call_site),
            "expected a string literal after `=`",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn error(src: &str) -> String {
        parse_decls(TokenStream::from_str(src).unwrap())
            .map(|_| ())
            .expect_err("expected a parse error")
            .to_compile_error()
            .to_string()
    }

    #[test]
    fn unknown_attributes_list_the_supported_ones() {
        let msg = error("x: Hermitian");
        assert!(msg.contains("unknown attribute `Hermitian`"), "{msg}");
        assert!(
            msg.contains(
                "Symmetric, Antisymmetric, Cyclesymmetric, Linear, Flat, Scalar, Real, Integer, Positive"
            ),
            "{msg}"
        );
    }

    #[test]
    fn names_ending_in_an_underscore_are_rejected() {
        let msg = error("a_");
        assert!(msg.contains("ends in `_`"), "{msg}");
        assert!(msg.contains("wildcard"), "{msg}");
    }
}

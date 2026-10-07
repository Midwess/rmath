//! `find!`: a per-call-site match struct with one `Atom` field per wildcard, filled from
//! `find_all`.

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, quote_spanned};

use crate::{
    Root,
    parse::{error::Errors, find::parse_find},
};

use super::expr::lower;

pub fn expand(root: &Root, body: TokenStream) -> Result<TokenStream, Errors> {
    let input = parse_find(body).map_err(Errors::from)?;
    let r = root.tokens();
    let wcs = &input.wildcards;

    let decls = wcs.iter().map(|w| {
        let name = Literal::string(&w.to_string());
        quote_spanned!(w.span()=> let #w = #r::__private::symbol!(#name);)
    });
    let pattern = lower(root, &input.pattern);
    let target = &input.target;
    // Hygienic name: user code cannot refer to the struct type, only to values of it.
    let strukt = Ident::new("RmathMatch", Span::mixed_site());

    Ok(quote!({
        #(#decls)*
        #[allow(dead_code)]
        #[derive(Debug, Clone)]
        struct #strukt {
            #(pub #wcs: #r::__private::Atom,)*
        }
        let target = #r::__private::into_expr(&(#target));
        let pattern = #r::__private::pattern(#pattern);
        #r::__private::find_all(&target, &pattern, [#(#wcs),*])
            .into_iter()
            .map(|[#(#wcs),*]| #strukt { #(#wcs),* })
    }))
}

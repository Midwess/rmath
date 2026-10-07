//! `symbols!`: one `let` binding per declaration, no enclosing block, so the bindings stay
//! visible after the macro.

use proc_macro2::{Literal, TokenStream};
use quote::quote_spanned;

use crate::{
    Root,
    parse::{error::Errors, symbols::parse_decls},
};

pub fn expand(root: &Root, body: TokenStream) -> Result<TokenStream, Errors> {
    let decls = parse_decls(body)?;
    let r = root.tokens();
    Ok(decls
        .iter()
        .map(|decl| {
            let ident = &decl.ident;
            // `r#type` declares a symbol named `type`; `tau0 = "τ_0"` overrides the name.
            let name = decl
                .name
                .clone()
                .unwrap_or_else(|| Literal::string(ident.to_string().trim_start_matches("r#")));
            if decl.attrs.is_empty() {
                quote_spanned!(ident.span()=> let #ident = #r::__private::symbol!(#name);)
            } else {
                let attrs = &decl.attrs;
                quote_spanned!(ident.span()=> let #ident = #r::__private::symbol!(#name; #(#attrs),*);)
            }
        })
        .collect())
}

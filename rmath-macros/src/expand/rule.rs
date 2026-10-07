//! `rule!`: declare the wildcards, build one `Replacement` per arm, wrap them in a `Rule`.

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, quote_spanned};

use crate::{
    Root,
    parse::{error::Errors, rule::parse_rules},
};

use super::expr::lower;

pub fn expand(root: &Root, body: TokenStream) -> Result<TokenStream, Errors> {
    let arms = parse_rules(body)?;
    let r = root.tokens();

    // Every wildcard of every arm, once, as a local `Symbol` the lowered sides refer to.
    let mut wildcards: Vec<Ident> = Vec::new();
    for arm in &arms {
        for w in &arm.wildcards {
            if !wildcards.contains(w) {
                wildcards.push(w.clone());
            }
        }
    }
    let decls = wildcards.iter().map(|w| {
        let name = Literal::string(&w.to_string());
        quote_spanned!(w.span()=> let #w = #r::__private::symbol!(#name);)
    });

    let replacements = arms.iter().map(|arm| {
        let lhs = lower(root, &arm.lhs);
        let rhs = lower(root, &arm.rhs);
        let replacement = quote!(#r::__private::Replacement::new(
            #r::__private::pattern(#lhs),
            #r::__private::pattern(#rhs),
        ));
        match &arm.guard {
            None => replacement,
            Some(cond) => {
                let wcs = &arm.wildcards;
                let n = Literal::usize_unsuffixed(wcs.len());
                // Inside the guard each wildcard name is the matched `Atom`, shadowing the
                // `Symbol` binding; `let _ = ..` keeps unused ones warning-free.
                quote!(#replacement.when(#r::__private::guard(
                    [#(#wcs),*],
                    move |[#(#wcs),*]: [#r::__private::Atom; #n]| -> bool {
                        let _ = (#(&#wcs,)*);
                        #cond
                    },
                )))
            }
        }
    });

    Ok(quote!({
        #(#decls)*
        #r::__private::Rule::new(vec![#(#replacements),*])
    }))
}

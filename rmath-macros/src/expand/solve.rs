//! `solve!`: lower each `lhs == rhs` to `lhs - rhs` and hand the system to Symbolica.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

use crate::{
    Root,
    parse::{error::Errors, solve::parse_solve},
};

use super::expr::lower;

pub fn expand(root: &Root, body: TokenStream) -> Result<TokenStream, Errors> {
    let input = parse_solve(body)?;
    let r = root.tokens();

    let equations = input.equations.iter().map(|(lhs, rhs)| {
        let lhs = lower(root, lhs);
        let rhs = lower(root, rhs);
        quote!(#r::__private::sub(#lhs, #rhs))
    });
    let unknowns = input
        .unknowns
        .iter()
        .map(|u| quote_spanned!(u.span()=> #r::__private::into_expr(&#u)));
    let domain = match &input.domain {
        Some(d) => {
            quote_spanned!(d.span()=> ::core::option::Option::Some(#r::__private::SolveDomain::#d))
        }
        None => quote!(::core::option::Option::None),
    };

    Ok(quote!(#r::__private::solve(&[#(#equations),*], &[#(#unknowns),*], #domain)))
}

//! `func!`: compile the body for the listed parameters and wrap the evaluator in a closure of
//! exactly that arity.

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};

use crate::{
    Root,
    parse::{error::Errors, func::parse_func},
};

use super::expr::lower;

pub fn expand(root: &Root, body: TokenStream) -> Result<TokenStream, Errors> {
    let input = parse_func(body).map_err(Errors::from)?;
    let r = root.tokens();
    let params = &input.params;
    let body = lower(root, &input.body);
    // Mixed-site names cannot collide with the user's parameters.
    let evaluator = Ident::new("evaluator", Span::mixed_site());
    let compiled = Ident::new("compiled", Span::mixed_site());
    // `param` only accepts `&Symbol`, so a non-symbol parameter is a compile error here.
    let param_atoms = params
        .iter()
        .map(|p| quote_spanned!(p.span()=> #r::__private::param(&#p)));

    Ok(quote!({
        let #compiled: #r::__private::Atom = #body;
        #r::__private::compile_f64(&#compiled, &[#(#param_atoms),*]).map(|mut #evaluator| {
            // The parameters shadow the `Symbol` bindings with the `f64` arguments.
            move |#(#params: f64),*| -> f64 { #evaluator.evaluate_single(&[#(#params),*]) }
        })
    }))
}

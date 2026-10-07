//! Procedural macros backing the `rmath` crate.
//!
//! This crate is an implementation detail of `rmath`. Depend on `rmath` and use the macros it
//! re-exports; nothing here is stable on its own.
//!
//! Every entry point receives `<crate root> ; <body>`: the `macro_rules!` shims in `rmath`
//! forward `$crate` so generated code can name `rmath` even when the dependency is renamed.

use proc_macro2::{TokenStream, TokenTree};

use crate::parse::error::{Error, Errors};

mod expand;
// TODO(rmath-v1 phase 2): drop once `Mode::Pattern` is consumed by `rule!`/`find!`.
#[allow(dead_code)]
mod parse;

/// The crate-root path forwarded by the shim (`$crate`), used as a prefix in generated code.
pub(crate) struct Root(TokenStream);

impl Root {
    /// Split `<root tokens> ; <body>` into the root and the remaining body.
    fn split(input: TokenStream) -> Result<(Root, TokenStream), Error> {
        let mut root = TokenStream::new();
        let mut iter = input.into_iter();
        for tt in iter.by_ref() {
            match &tt {
                TokenTree::Punct(p) if p.as_char() == ';' => {
                    return Ok((Root(root), iter.collect()));
                }
                _ => root.extend(std::iter::once(tt)),
            }
        }
        Err(Error::new(
            proc_macro2::Span::call_site(),
            "rmath internal error: macro invoked without its crate-root prefix; use the `rmath::expr!` shim",
        ))
    }

    fn tokens(&self) -> &TokenStream {
        &self.0
    }
}

/// Shared driver: split off the root, run the expander, render errors as `compile_error!`.
fn run(
    input: proc_macro::TokenStream,
    expand: fn(&Root, TokenStream) -> Result<TokenStream, Errors>,
) -> proc_macro::TokenStream {
    let out = match Root::split(input.into()) {
        Ok((root, body)) => expand(&root, body).unwrap_or_else(|e| e.to_compile_error()),
        Err(e) => e.to_compile_error(),
    };
    out.into()
}

#[doc(hidden)]
#[proc_macro]
pub fn __expr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    run(input, expand::expr::expand)
}

#[doc(hidden)]
#[proc_macro]
pub fn __symbols(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    run(input, expand::symbols::expand)
}

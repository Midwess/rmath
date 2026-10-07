//! `expr!`: lower a parsed expression to Symbolica calls.

use proc_macro2::{Literal, TokenStream};
use quote::{quote, quote_spanned};

use crate::{
    Root,
    parse::{
        ast::{BinOp, Expr, ExprKind},
        cursor::Cursor,
        error::Errors,
        expr::{Mode, parse_all},
        lit::Num,
    },
};

/// Entry point for `__expr`.
pub fn expand(root: &Root, body: TokenStream) -> Result<TokenStream, Errors> {
    let mut cursor = Cursor::new(body);
    let expr = parse_all(&mut cursor, Mode::Expr).map_err(Errors::from)?;
    Ok(lower(root, &expr))
}

/// Emit the code computing `expr`, as an expression of type `Atom`.
pub fn lower(root: &Root, expr: &Expr) -> TokenStream {
    let r = root.tokens();
    match &expr.kind {
        ExprKind::Num(Num::I64(v)) => {
            let lit = Literal::i64_suffixed(*v);
            quote!(#r::__private::into_expr(&#lit))
        }
        ExprKind::Num(Num::I128(v)) => {
            let lit = Literal::i128_suffixed(*v);
            quote!(#r::__private::into_expr(&#lit))
        }
        ExprKind::Num(Num::Suffixed(lit)) => quote!(#r::__private::into_expr(&#lit)),
        ExprKind::Num(Num::Float(lit)) => quote!(#r::__private::float(#lit as f64)),
        ExprKind::Ident(id) | ExprKind::Wildcard { ident: id, .. } => {
            // Spanned at the identifier so E0425 and trait errors land on the user's token.
            // A wildcard is a local `Symbol` bound by `rule!`/`find!`, so it lowers the same way.
            quote_spanned!(id.span()=> #r::__private::into_expr(&#id))
        }
        ExprKind::Splice(group) => quote!(#r::__private::into_expr(&#group)),
        ExprKind::Neg(inner) => {
            let inner = lower(root, inner);
            quote!(#r::__private::neg(#inner))
        }
        ExprKind::Factorial(inner) => {
            let inner = lower(root, inner);
            quote!(#r::__private::factorial(#inner))
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let lhs = lower(root, lhs);
            let rhs = lower(root, rhs);
            let helper = match op {
                BinOp::Add => quote!(add),
                BinOp::Sub => quote!(sub),
                BinOp::Mul => quote!(mul),
                BinOp::Div => quote!(div),
                BinOp::Pow => quote!(pow),
            };
            quote!(#r::__private::#helper(#lhs, #rhs))
        }
        ExprKind::Call { func, args } => {
            // The arity is spelled out so a wrong count fails on the `Callable<N>` bound
            // (and its message) instead of on an array-size mismatch.
            let n = Literal::usize_unsuffixed(args.len());
            let args = args.iter().map(|a| lower(root, a));
            quote_spanned!(func.span()=> #r::__private::call::<_, #n>(&#func, [#(#args),*]))
        }
    }
}

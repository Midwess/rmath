//! Pratt parser for the math grammar.

use proc_macro2::{Delimiter, Group, Ident, Span, TokenTree};

use super::{
    ast::{BinOp, Expr, ExprKind, Wild},
    cursor::Cursor,
    error::{Error, SpanRange},
    lit,
};

#[cfg(test)]
mod tests;

/// Whether trailing-underscore identifiers denote wildcards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Expr,
    Pattern,
}

/// Binding power of a prefix `-`: looser than `^` (so `-x^2` is `-(x^2)`), tighter than `*`.
const NEG_BP: u8 = 5;
/// Binding power of postfix `!`: tighter than everything, so `n!^2` is `(n!)^2`.
const FACTORIAL_BP: u8 = 9;

/// Identifiers that end an expression instead of starting a new operand (`for x`, `if cond`,
/// `over Reals`).
const KEYWORDS: &[&str] = &["for", "if", "over"];

/// Parse one complete expression from the cursor.
pub fn parse_expr(c: &mut Cursor, mode: Mode) -> Result<Expr, Error> {
    expr_bp(c, 0, mode)
}

/// Parse a whole token stream as one expression; trailing tokens are an error.
pub fn parse_all(c: &mut Cursor, mode: Mode) -> Result<Expr, Error> {
    let expr = expr_bp(c, 0, mode)?;
    match c.peek() {
        None => Ok(expr),
        Some(_) => Err(trailing_token_error(c)),
    }
}

/// Explain a token that cannot continue the expression.
fn trailing_token_error(c: &Cursor) -> Error {
    let tok = c.peek().expect("caller checked");
    if c.peek_op("=") {
        return Error::new(tok.span(), "`=` is not an operator")
            .with_help("use `==` to write an equation in `solve!`");
    }
    if c.peek_op("::") {
        return Error::new(tok.span(), "paths are not supported in math expressions")
            .with_help("wrap the Rust expression in braces: `{ a::b }`");
    }
    if c.peek_op(".") {
        return Error::new(
            tok.span(),
            "method calls and field access are not supported in math expressions",
        )
        .with_help("wrap the Rust expression in braces: `{ x.y }`");
    }
    Error::new(
        tok.span(),
        format!("unexpected `{tok}` after the expression"),
    )
}

/// Pratt loop: parse operands while the next operator binds at least as tightly as `min_bp`.
fn expr_bp(c: &mut Cursor, min_bp: u8, mode: Mode) -> Result<Expr, Error> {
    let mut lhs = primary(c, mode)?;
    loop {
        if c.peek_op("!") {
            if let Some(TokenTree::Group(g)) = c.peek_nth(1) {
                let bang = c.peek().expect("peeked");
                let head = c.last_token().map(ToString::to_string).unwrap_or_default();
                return Err(Error::new(
                    SpanRange {
                        start: bang.span(),
                        end: g.span(),
                    },
                    "macro-call syntax is not a math expression",
                )
                .with_help(format!(
                    "to splice Rust code, wrap it in braces: `{{ {head}!{g} }}`"
                )));
            }
            if FACTORIAL_BP < min_bp {
                break;
            }
            let bang = c.bump().expect("peeked");
            lhs = Expr {
                span: SpanRange {
                    start: lhs.span.start,
                    end: bang.span(),
                },
                kind: ExprKind::Factorial(Box::new(lhs)),
            };
            continue;
        }

        if let Some(next) = c.peek()
            && starts_operand(next)
        {
            let prev = c.last_token().map(ToString::to_string).unwrap_or_default();
            return Err(
                Error::new(next.span(), "implicit multiplication is not supported")
                    .with_help(format!("insert `*` between `{prev}` and `{next}`")),
            );
        }

        let Some((op, l_bp, r_bp)) = peek_binop(c) else {
            break;
        };
        if l_bp < min_bp {
            break;
        }
        c.bump();
        let rhs = expr_bp(c, r_bp, mode)?;
        let span = SpanRange {
            start: lhs.span.start,
            end: rhs.span.end,
        };
        lhs = Expr {
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span,
        };
    }
    Ok(lhs)
}

/// Could `tok` begin an operand? Directly after another operand that means the user forgot `*`.
fn starts_operand(tok: &TokenTree) -> bool {
    match tok {
        TokenTree::Ident(id) => !KEYWORDS.contains(&id.to_string().as_str()),
        TokenTree::Literal(_) => true,
        TokenTree::Group(g) => g.delimiter() != Delimiter::Bracket,
        TokenTree::Punct(_) => false,
    }
}

/// The binary operator at the cursor with its (left, right) binding powers.
///
/// Left-associative operators have `right = left + 1`; the right-associative `^` has
/// `right = left - 1` so `2^3^2` parses as `2^(3^2)`.
fn peek_binop(c: &Cursor) -> Option<(BinOp, u8, u8)> {
    const TABLE: &[(&str, BinOp, u8, u8)] = &[
        ("+", BinOp::Add, 1, 2),
        ("-", BinOp::Sub, 1, 2),
        ("*", BinOp::Mul, 3, 4),
        ("/", BinOp::Div, 3, 4),
        ("^", BinOp::Pow, 8, 7),
    ];
    TABLE
        .iter()
        .find(|(sym, ..)| c.peek_op(sym))
        .map(|&(_, op, l, r)| (op, l, r))
}

/// A prefix operator applied to an operand, a parenthesised expression, a call, a splice,
/// a literal, or an identifier.
fn primary(c: &mut Cursor, mode: Mode) -> Result<Expr, Error> {
    if c.peek_op("-") {
        let minus = c.bump().expect("peeked");
        let operand = expr_bp(c, NEG_BP, mode)?;
        return Ok(Expr {
            span: SpanRange {
                start: minus.span(),
                end: operand.span.end,
            },
            kind: ExprKind::Neg(Box::new(operand)),
        });
    }
    match c.bump() {
        Some(TokenTree::Ident(id)) if id == "_" => Err(Error::new(
            id.span(),
            "`_` is not a valid name",
        )
        .with_help(
            "wildcards need a prefix, like `a_`, and are only meaningful in `rule!` and `find!` patterns",
        )),
        Some(TokenTree::Ident(id)) => match c.peek() {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                let g = g.clone();
                c.bump();
                call(id, &g, mode)
            }
            _ => match (mode, wildcard_arity(&id)?) {
                (Mode::Pattern, Some(arity)) => Ok(Expr {
                    span: id.span().into(),
                    kind: ExprKind::Wildcard { ident: id, arity },
                }),
                _ => Ok(Expr {
                    span: id.span().into(),
                    kind: ExprKind::Ident(id),
                }),
            },
        },
        Some(TokenTree::Literal(l)) => Ok(Expr {
            span: l.span().into(),
            kind: ExprKind::Num(lit::classify(&l)?),
        }),
        Some(TokenTree::Group(g)) => match g.delimiter() {
            Delimiter::Parenthesis => parenthesised(&g, mode),
            Delimiter::Brace | Delimiter::None => Ok(Expr {
                span: g.span().into(),
                kind: ExprKind::Splice(g),
            }),
            Delimiter::Bracket => Err(Error::new(
                g.span(),
                "expected an expression, found a bracketed list",
            )),
        },
        Some(other) => Err(Error::new(
            other.span(),
            format!("expected an expression, found `{other}`"),
        )),
        None => {
            let msg = match c.last_token() {
                Some(TokenTree::Punct(op)) => format!("expected an expression after `{op}`"),
                _ => "expected an expression".to_string(),
            };
            Err(Error::new(
                c.last_span().unwrap_or_else(Span::call_site),
                msg,
            ))
        }
    }
}

/// Symbolica's wildcard convention: a name with one to three trailing underscores.
///
/// Returns `Ok(None)` for ordinary names and an error for four or more underscores. Only
/// consulted in pattern mode; in expression mode `a_` is a plain Rust identifier.
pub fn wildcard_arity(id: &Ident) -> Result<Option<Wild>, Error> {
    let name = id.to_string();
    let underscores = name.len() - name.trim_end_matches('_').len();
    match underscores {
        0 => Ok(None),
        _ if underscores == name.len() => Ok(None), // `_` alone is rejected elsewhere
        1 => Ok(Some(Wild::One)),
        2 => Ok(Some(Wild::OneOrMore)),
        3 => Ok(Some(Wild::ZeroOrMore)),
        _ => Err(Error::new(
            id.span(),
            format!("`{name}` has {underscores} trailing underscores; wildcards have at most three trailing underscores"),
        )
        .with_help("`a_` matches one argument, `a__` one or more, `a___` zero or more")),
    }
}

/// `( expr )`: no node of its own, but the span widens to the parentheses.
fn parenthesised(g: &Group, mode: Mode) -> Result<Expr, Error> {
    let mut inner = Cursor::new(g.stream());
    if inner.peek().is_none() {
        return Err(Error::new(
            g.span(),
            "expected an expression inside the parentheses",
        ));
    }
    let mut expr = parse_all(&mut inner, mode)?;
    expr.span = g.span().into();
    Ok(expr)
}

/// `f(a, b)`: each top-level comma-separated piece is one argument.
fn call(func: Ident, args: &Group, mode: Mode) -> Result<Expr, Error> {
    let parsed = Cursor::split_top_level(args.stream(), ',')
        .into_iter()
        .map(|piece| {
            let mut inner = Cursor::new(piece);
            if inner.peek().is_none() {
                return Err(Error::new(args.span(), "empty argument in function call"));
            }
            parse_all(&mut inner, mode)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Expr {
        span: SpanRange {
            start: func.span(),
            end: args.span(),
        },
        kind: ExprKind::Call { func, args: parsed },
    })
}

//! The compile-time AST produced by the parser and consumed by the expanders.
//!
//! It lives only inside a macro invocation; nothing here reaches user code.

use std::fmt;

use proc_macro2::{Group, Ident};

use super::{error::SpanRange, lit::Num};

/// An expression node with the source range it was parsed from.
pub struct Expr {
    pub kind: ExprKind,
    pub span: SpanRange,
}

pub enum ExprKind {
    Num(Num),
    /// Re-emitted verbatim; rustc resolves it (symbol, splice or built-in).
    Ident(Ident),
    /// `{ rust expression }` (or an invisible-delimiter group), spliced through `IntoExpr`.
    Splice(Group),
    /// Unary minus.
    Neg(Box<Expr>),
    /// Postfix `!`.
    Factorial(Box<Expr>),
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `f(a, b)`: the head is re-emitted verbatim and dispatched through `Callable`.
    Call {
        func: Ident,
        args: Vec<Expr>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Pow => "^",
        }
    }
}

/// S-expression rendering, used by tests and diagnostics: `x + 1` prints as `(+ x 1)`.
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ExprKind::Num(Num::I64(v)) => write!(f, "{v}"),
            ExprKind::Num(Num::I128(v)) => write!(f, "{v}"),
            ExprKind::Num(Num::Suffixed(l) | Num::Float(l)) => write!(f, "{l}"),
            ExprKind::Ident(id) => write!(f, "{id}"),
            ExprKind::Splice(_) => write!(f, "{{...}}"),
            ExprKind::Neg(e) => write!(f, "(neg {e})"),
            ExprKind::Factorial(e) => write!(f, "(! {e})"),
            ExprKind::Binary { op, lhs, rhs } => write!(f, "({} {lhs} {rhs})", op.symbol()),
            ExprKind::Call { func, args } => {
                write!(f, "(call {func}")?;
                for a in args {
                    write!(f, " {a}")?;
                }
                write!(f, ")")
            }
        }
    }
}

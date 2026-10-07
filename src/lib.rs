#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod __private;
mod call;
mod into_expr;
mod macros;
pub mod prelude;
mod rule;

pub use call::Callable;
pub use into_expr::IntoExpr;
pub use rule::{ApplyRule, Rule};
pub use symbolica;

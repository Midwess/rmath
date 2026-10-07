//! Code generation: AST → calls on `<root>::__private::*`.
//!
//! Generated code uses full paths only (never a glob import), so user locals named `add`,
//! `pow` or `call` are never shadowed, and never names `::symbolica` directly.

pub mod expr;
pub mod rule;
pub mod symbols;

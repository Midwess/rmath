//! Compile-time parser for the rmath math grammar.
//!
//! Works directly on `proc_macro2` token trees (no lexer of our own) and produces a
//! short-lived AST that the `expand` modules lower to Symbolica calls.

pub mod ast;
pub mod cursor;
pub mod error;
pub mod expr;
pub mod lit;
pub mod rule;
pub mod symbols;

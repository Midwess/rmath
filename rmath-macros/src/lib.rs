//! Procedural macros backing the `rmath` crate.
//!
//! This crate is an implementation detail of `rmath`. Depend on `rmath` and use the macros it
//! re-exports; nothing here is stable on its own.

// TODO(rmath-v1 task 1.8): drop once `expand` consumes the parser.
#[allow(dead_code)]
mod parse;

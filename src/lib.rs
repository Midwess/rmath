//! Native math-syntax macros for the Symbolica computer algebra system.
//!
//! Write expressions as math (`expr!(x^2 + sin(y)/2)`), checked by the compiler and expanded
//! at compile time into calls on Symbolica's public API. The result is always a plain
//! Symbolica `Atom`, so the full Symbolica API stays available.

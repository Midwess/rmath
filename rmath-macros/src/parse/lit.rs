//! Classification of numeric literals.

use proc_macro2::Literal;
use syn::Lit;

use super::error::Error;

/// A numeric literal as the grammar understands it.
#[derive(Debug)]
pub enum Num {
    /// An unsuffixed integer that fits `i64`.
    I64(i64),
    /// An unsuffixed integer that fits `i128` but not `i64`.
    I128(i128),
    /// An integer with an explicit Rust suffix (`2i8`), re-emitted verbatim.
    Suffixed(Literal),
    /// A decimal or exponent literal (`0.5`, `1e3`), re-emitted verbatim as an `f64`.
    Float(Literal),
}

/// Rust's integer type suffixes; anything else after digits is not a number (`2x`).
const INT_SUFFIXES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
];

/// Rust's float type suffixes.
const FLOAT_SUFFIXES: &[&str] = &["", "f32", "f64"];

/// Classify a literal token, rejecting anything the grammar does not accept.
pub fn classify(lit: &Literal) -> Result<Num, Error> {
    match Lit::new(lit.clone()) {
        Lit::Int(int) if int.suffix().is_empty() => {
            if let Ok(v) = int.base10_parse::<i64>() {
                return Ok(Num::I64(v));
            }
            int.base10_parse::<i128>().map(Num::I128).map_err(|_| {
                Error::new(
                    lit.span(),
                    "integer literal is too large; rmath supports literals up to i128::MAX",
                )
                .with_help("splice a Rust value instead, e.g. `{ big_integer }`")
            })
        }
        Lit::Int(int) if INT_SUFFIXES.contains(&int.suffix()) => Ok(Num::Suffixed(lit.clone())),
        Lit::Int(int) => Err(implicit_multiplication(lit, int.suffix())),
        Lit::Float(f) if FLOAT_SUFFIXES.contains(&f.suffix()) => Ok(Num::Float(lit.clone())),
        Lit::Float(f) => Err(implicit_multiplication(lit, f.suffix())),
        _ => Err(Error::new(
            lit.span(),
            "only numeric literals are allowed in math expressions",
        )
        .with_help("to splice a Rust value, wrap it in braces: `{ value }`")),
    }
}

/// `2x` lexes as the integer `2` with suffix `x`: the user meant `2 * x`.
fn implicit_multiplication(lit: &Literal, suffix: &str) -> Error {
    let text = lit.to_string();
    let digits = &text[..text.len() - suffix.len()];
    Error::new(lit.span(), "implicit multiplication is not supported")
        .with_help(format!("write `{digits} * {suffix}`"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Literal;
    use std::str::FromStr;

    fn lit(src: &str) -> Literal {
        Literal::from_str(src).unwrap()
    }

    #[test]
    fn plain_integer_literals_are_i64() {
        assert!(matches!(classify(&lit("42")).unwrap(), Num::I64(42)));
        assert!(matches!(classify(&lit("1_000")).unwrap(), Num::I64(1000)));
        assert!(matches!(classify(&lit("0x1F")).unwrap(), Num::I64(31)));
    }

    #[test]
    fn large_integers_widen_to_i128_until_they_overflow() {
        let big = "9223372036854775808"; // i64::MAX + 1
        assert!(matches!(classify(&lit(big)).unwrap(), Num::I128(v) if v == i64::MAX as i128 + 1));

        let too_big = "170141183460469231731687303715884105728"; // i128::MAX + 1
        let err = classify(&lit(too_big)).unwrap_err();
        let msg = err.to_compile_error().to_string();
        assert!(msg.contains("integer literal is too large"), "{msg}");
    }

    #[test]
    fn integer_suffixes_are_preserved() {
        assert!(
            matches!(classify(&lit("2i8")).unwrap(), Num::Suffixed(l) if l.to_string() == "2i8")
        );
        assert!(
            matches!(classify(&lit("3u32")).unwrap(), Num::Suffixed(l) if l.to_string() == "3u32")
        );
    }

    #[test]
    fn decimal_literals_are_floats() {
        assert!(matches!(classify(&lit("0.5")).unwrap(), Num::Float(l) if l.to_string() == "0.5"));
        assert!(matches!(classify(&lit("2.0")).unwrap(), Num::Float(l) if l.to_string() == "2.0"));
        assert!(matches!(classify(&lit("1e3")).unwrap(), Num::Float(l) if l.to_string() == "1e3"));
    }

    #[test]
    fn digits_glued_to_an_identifier_are_implicit_multiplication() {
        let msg = classify(&lit("2x"))
            .unwrap_err()
            .to_compile_error()
            .to_string();
        assert!(
            msg.contains("implicit multiplication is not supported"),
            "{msg}"
        );
        assert!(msg.contains("help: write `2 * x`"), "{msg}");

        let msg = classify(&lit("2.5x"))
            .unwrap_err()
            .to_compile_error()
            .to_string();
        assert!(msg.contains("help: write `2.5 * x`"), "{msg}");
    }

    #[test]
    fn non_numeric_literals_are_rejected() {
        for src in [r#""text""#, "'c'", r#"b"bytes""#] {
            let msg = classify(&lit(src))
                .unwrap_err()
                .to_compile_error()
                .to_string();
            assert!(
                msg.contains("only numeric literals are allowed"),
                "{src}: {msg}"
            );
        }
    }
}

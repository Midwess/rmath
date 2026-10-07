//! Span-carrying parse errors, rendered as `compile_error!` invocations.

use proc_macro2::{Literal, Span, TokenStream};
use quote::quote_spanned;

/// The first and last span of the offending tokens.
///
/// `Span::join` is nightly-only, so both ends are kept and attached to the two halves of the
/// emitted `compile_error!`; rustc then underlines the whole range (the technique `syn` uses).
#[derive(Clone, Copy, Debug)]
pub struct SpanRange {
    pub start: Span,
    pub end: Span,
}

impl From<Span> for SpanRange {
    fn from(span: Span) -> Self {
        SpanRange {
            start: span,
            end: span,
        }
    }
}

/// One parse error with an optional `help:` suggestion.
#[derive(Debug)]
pub struct Error {
    span: SpanRange,
    msg: String,
    help: Option<String>,
}

impl Error {
    pub fn new(span: impl Into<SpanRange>, msg: impl Into<String>) -> Self {
        Error {
            span: span.into(),
            msg: msg.into(),
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Render as `::core::compile_error! { "msg\n\nhelp: ..." }` spanning the error range.
    pub fn to_compile_error(&self) -> TokenStream {
        let mut text = self.msg.clone();
        if let Some(help) = &self.help {
            text.push_str("\n\nhelp: ");
            text.push_str(help);
        }
        let mut lit = Literal::string(&text);
        lit.set_span(self.span.end);

        let start = self.span.start;
        let end = self.span.end;
        let head = quote_spanned!(start=> ::core::compile_error!);
        let body = quote_spanned!(end=> { #lit });
        let mut out = head;
        out.extend(body);
        out
    }
}

/// Errors from independent items (declarations, rule arms, equations), all reported at once.
#[derive(Debug, Default)]
pub struct Errors(Vec<Error>);

impl From<Error> for Errors {
    fn from(err: Error) -> Self {
        Errors(vec![err])
    }
}

impl Errors {
    pub fn push(&mut self, err: Error) {
        self.0.push(err);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// One `compile_error!` per collected error.
    pub fn to_compile_error(&self) -> TokenStream {
        self.0.iter().map(Error::to_compile_error).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_error_carries_message_and_help() {
        let err = Error::new(
            Span::call_site(),
            "implicit multiplication is not supported",
        )
        .with_help("write `2 * x`");
        let out = err.to_compile_error().to_string();
        assert!(out.contains("compile_error"), "{out}");
        assert!(
            out.contains("implicit multiplication is not supported"),
            "{out}"
        );
        assert!(out.contains("help: write `2 * x`"), "{out}");
    }

    #[test]
    fn errors_report_every_collected_error() {
        let mut errs = Errors::default();
        assert!(errs.is_empty());
        errs.push(Error::new(Span::call_site(), "first problem"));
        errs.push(Error::new(Span::call_site(), "second problem"));
        let out = errs.to_compile_error().to_string();
        assert_eq!(out.matches("compile_error").count(), 2, "{out}");
        assert!(
            out.contains("first problem") && out.contains("second problem"),
            "{out}"
        );
    }
}

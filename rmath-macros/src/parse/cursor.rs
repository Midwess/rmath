//! Token cursor: a peekable stream of `TokenTree`s with joint-punctuation awareness.

use proc_macro2::{Spacing, Span, TokenStream, TokenTree};

/// A peekable cursor over a flat sequence of token trees.
pub struct Cursor {
    toks: Vec<TokenTree>,
    pos: usize,
    /// The most recently consumed token; anchors "unexpected end of input" errors and names
    /// the left neighbour in "insert `*` between ..." help.
    last: Option<TokenTree>,
}

impl Cursor {
    pub fn new(ts: TokenStream) -> Self {
        Cursor {
            toks: ts.into_iter().collect(),
            pos: 0,
            last: None,
        }
    }

    /// Look at the next token without consuming it.
    pub fn peek(&self) -> Option<&TokenTree> {
        self.toks.get(self.pos)
    }

    /// Look `n` tokens ahead without consuming (`peek_nth(0)` is `peek`).
    pub fn peek_nth(&self, n: usize) -> Option<&TokenTree> {
        self.toks.get(self.pos + n)
    }

    /// Consume and return the next token.
    pub fn bump(&mut self) -> Option<TokenTree> {
        let tok = self.toks.get(self.pos).cloned();
        if tok.is_some() {
            self.last = tok.clone();
            self.pos += 1;
        }
        tok
    }

    /// The most recently consumed token, if any.
    pub fn last_token(&self) -> Option<&TokenTree> {
        self.last.as_ref()
    }

    /// Span of the most recently consumed token, if any.
    pub fn last_span(&self) -> Option<Span> {
        self.last.as_ref().map(TokenTree::span)
    }

    /// Does the upcoming punctuation spell `op` exactly?
    ///
    /// Multi-character operators are joint punctuation (`==`, `=>`), so every character but
    /// the last must be `Joint`. A shorter operator never matches the prefix of a longer
    /// known one (`=` does not match the start of `==`), but adjacent punctuation that forms
    /// no operator is fine: in `x^-1` the `^` is `Joint` with `-` and still matches.
    pub fn peek_op(&self, op: &str) -> bool {
        let mut chars = op.chars().peekable();
        let mut i = self.pos;
        while let Some(ch) = chars.next() {
            let Some(TokenTree::Punct(p)) = self.toks.get(i) else {
                return false;
            };
            if p.as_char() != ch {
                return false;
            }
            let is_last = chars.peek().is_none();
            match (is_last, p.spacing()) {
                (false, Spacing::Alone) => return false,
                (true, Spacing::Joint) if self.extends_to_known_op(op, i + 1) => return false,
                _ => {}
            }
            i += 1;
        }
        true
    }

    /// Split `ts` on `sep` punctuation at the top level only; delimited groups stay intact.
    ///
    /// Empty pieces between two separators are kept (so callers can report "empty argument"),
    /// but a trailing separator does not produce an empty final piece.
    pub fn split_top_level(ts: TokenStream, sep: char) -> Vec<TokenStream> {
        let mut parts = Vec::new();
        let mut current = TokenStream::new();
        for tt in ts {
            match &tt {
                TokenTree::Punct(p) if p.as_char() == sep => {
                    parts.push(std::mem::take(&mut current));
                }
                _ => current.extend(std::iter::once(tt)),
            }
        }
        if !current.is_empty() {
            parts.push(current);
        }
        parts
    }

    /// Would `op` followed by the punctuation at `next` be the start of a longer operator?
    fn extends_to_known_op(&self, op: &str, next: usize) -> bool {
        let Some(TokenTree::Punct(q)) = self.toks.get(next) else {
            return false;
        };
        let mut longer = op.to_string();
        longer.push(q.as_char());
        MULTI_CHAR_OPS
            .iter()
            .any(|known| known.starts_with(&longer))
    }
}

/// Every multi-character operator the grammar (or its error paths) recognises.
const MULTI_CHAR_OPS: &[&str] = &["==", "=>", "||", "::"];

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn cursor(src: &str) -> Cursor {
        Cursor::new(TokenStream::from_str(src).unwrap())
    }

    #[test]
    fn peek_does_not_advance_but_bump_does() {
        let mut c = cursor("x + 1");
        assert_eq!(c.peek().unwrap().to_string(), "x");
        assert_eq!(c.peek().unwrap().to_string(), "x");
        assert_eq!(c.bump().unwrap().to_string(), "x");
        assert_eq!(c.peek().unwrap().to_string(), "+");
    }

    #[test]
    fn peek_op_matches_joint_punctuation_exactly() {
        let c = cursor("== 1");
        assert!(c.peek_op("=="));
        assert!(!c.peek_op("="), "`=` must not match the first half of `==`");

        let c = cursor("= 1");
        assert!(c.peek_op("="));
        assert!(!c.peek_op("=="));

        let c = cursor("=> x");
        assert!(c.peek_op("=>"));
        assert!(!c.peek_op("="));
    }

    #[test]
    fn peek_op_accepts_joint_spacing_when_no_longer_operator_exists() {
        // `x^-1` without spaces: `^` is Joint with the following `-`, but `^-` is not an
        // operator, so `^` must still be recognised.
        let c = cursor("^-1");
        assert!(c.peek_op("^"));
        let c = cursor("+-1");
        assert!(c.peek_op("+"));
    }

    #[test]
    fn last_span_tracks_the_most_recently_consumed_token() {
        let mut c = cursor("x + 1");
        assert!(c.last_span().is_none(), "nothing consumed yet");
        c.bump();
        assert!(c.last_span().is_some());
    }

    #[test]
    fn split_top_level_ignores_separators_inside_groups() {
        let parts = Cursor::split_top_level(TokenStream::from_str("a, f(b, c), d").unwrap(), ',');
        let parts: Vec<String> = parts.iter().map(|p| p.to_string()).collect();
        assert_eq!(parts, ["a", "f (b , c)", "d"]);
    }

    #[test]
    fn split_top_level_handles_empty_input_and_trailing_separator() {
        assert!(Cursor::split_top_level(TokenStream::new(), ',').is_empty());
        let parts = Cursor::split_top_level(TokenStream::from_str("a, b,").unwrap(), ',');
        assert_eq!(
            parts.len(),
            2,
            "a trailing separator does not create an empty piece"
        );
    }
}

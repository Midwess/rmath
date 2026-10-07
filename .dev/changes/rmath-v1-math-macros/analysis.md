# Codebase Analysis: rmath-v1-math-macros

Source: `code-explorer` run during `/dev-workflow:init` (2026-10-07), scoped to `Cargo.toml`,
`Cargo.lock`, `.gitignore`, `src/`. Re-used here because the codebase has not changed since.
`.dev/symbolica/` was deliberately excluded (license forbids AI analysis; see ADR-0001).

## Current State

- Unmodified `cargo new --lib` stub: `src/lib.rs` (15 lines) with `pub fn add` and one test.
- `Cargo.toml`: single package `rmath` 0.1.0, edition 2024, empty `[dependencies]`, no
  workspace, features, profiles or dev-dependencies.
- `Cargo.lock` v4 with only the local package.
- `.gitignore`: `/target`, `.DS_Store`, `.dev`.
- No CI, no `rustfmt.toml`/`clippy.toml`, no `tests/`, no docs, no commits.

## Similar Features in the Codebase

None. This proposal establishes the first real code.

## Architecture Layers Relevant to This Change

None exist. The proposal introduces two layers (see `blueprint.md`): a compile-time layer
(`rmath-macros`: parse → expand) and a runtime layer (`rmath`: traits and helpers over
Symbolica).

## Dependencies

- **Internal:** none yet.
- **External (to add):** `symbolica 3.0` (public API only), `proc-macro2 1`, `quote 1`,
  `syn 2` (minimal features), dev `trybuild 1`.
- **Configuration:** workspace manifest, feature passthrough to Symbolica, MSRV 1.96.

## Conventions to Follow

- Rust defaults: `rustfmt`, `clippy` default lints, inline `#[cfg(test)] mod tests`.
- Proc-macro ecosystem conventions: `<crate>-macros` companion crate, `#[doc(hidden)]
  __private` module for expansion helpers, generated code uses absolute `::crate::` paths,
  `trybuild` for UI tests.
- Vocabulary from `CONTEXT.md` must be used in specs, docs and identifiers.

## OpenSpec Integration Notes

- `.dev/specs/` is empty; every requirement in this change is `ADDED`.
- Domains introduced: `grammar`, `symbols-and-expr`, `rules-and-find`, `solve`, `func`,
  `distribution`.

# Project Context

## Overview

**rmath** (v0.1.0): a Rust library that adds native math-syntax macros on top of the
[Symbolica](https://crates.io/crates/symbolica) computer algebra system. Users write
expressions such as `expr!(x^2 + sin(y)/2)` directly in Rust source. The macros parse them at
compile time and expand into calls on Symbolica's public API.

**Current state:** an unmodified `cargo new --lib` stub (`src/lib.rs` holds only the template
`add` function and one test). No dependencies, modules or commits yet. Design is in progress
(brainstorming phase).

## Tech Stack

### Languages
- Rust, edition 2024
- MSRV: 1.96 (planned; inherited from Symbolica 3.0.1)

### Frameworks
- None yet
- Planned: `symbolica = "3.0.1"` (runtime), plus `proc-macro2`, `quote` and possibly `syn` (macro crate)

### Testing
- Built-in Rust test harness (`cargo test`); one template test currently
- Planned: `trybuild` for compile-fail tests of macro diagnostics

### Build Tools
- Cargo (lockfile v4)
- No `rustfmt.toml`, `clippy.toml` or CI configured yet

## Key Directories

| Directory | Purpose |
|-----------|---------|
| `src/` | Crate source (currently the `cargo new` stub) |
| `.dev/` | dev-workflow specs, changes and archive. Also holds a local `symbolica` clone (see Notes) |
| `target/` | Cargo build output (gitignored) |

## Architecture

### Style
None established yet: a single flat crate.

**Planned (proposed, not yet approved):** a Cargo workspace using the `serde` / `serde_derive`
split.

### Layers (planned)
| Layer | Directory | Purpose |
|-------|-----------|---------|
| Runtime crate | `src/` (root package `rmath`) | Re-exports the macros and `symbolica`; `IntoExpr` trait; `#[doc(hidden)] __private` helpers called by macro output; built-ins (`sin`, `cos`, `exp`, `log`, `sqrt`, `abs`, `I`) as Rust items in the prelude |
| Macro crate | `rmath-macros/` (`proc-macro = true`) | Entry points `expr!`, `symbols!`, `rule!`, `solve!`, `func!` |
| Parser | `rmath-macros/src/parse/` | Pratt parser over `proc_macro2` tokens producing a compile-time-only AST (unit-testable) |
| Codegen | `rmath-macros/src/expand/` | One module per macro that emits Symbolica API calls |

### Key Patterns (decided)
- **Math-style DSL:** a custom precedence parser, needed because Rust's own `^` (XOR) binds more loosely than `+`.
- **Explicit, compiler-checked names:** symbols must be declared with `symbols!`. An identifier inside a macro expands to the Rust identifier itself, so a typo is a compile error.
- **Direct expansion:** the macros emit Symbolica calls that return a plain `symbolica::atom::Atom`. No wrapper type and no runtime string parsing.
- **Hygiene:** public macros are `macro_rules!` shims forwarding `$crate`; generated code references only `$crate::__private::…` full paths, never `::symbolica::…` and never a glob import.
- **Wildcards** in `rule!` are declared implicitly by a trailing `_` / `__` / `___`.

## Conventions

### Naming
- Crates: `rmath`, `rmath-macros`
- Not yet defined beyond Rust defaults (snake_case items, CamelCase types)

### Code Style
- Default `rustfmt`; no custom config yet
- `cargo clippy` with default lints (no config yet)

### Testing
- Inline `#[cfg(test)] mod tests` (cargo default)
- Planned: parser unit tests via `proc_macro2`, `trybuild` compile-fail tests for diagnostics,
  and equivalence tests (macro output == `symbolica::parse!` output)

### Git
- No commits yet. Main branch is expected to be `main` (the working branch is currently `master`).
- Commit format not yet defined.

## Build Commands

| Command | Purpose |
|---------|---------|
| `cargo build` | Build the project |
| `cargo test` | Run tests |
| `cargo clippy --all-targets` | Run linter |
| `cargo fmt --check` | Check formatting |

## Notes

### Symbolica license constraints (important)
Symbolica is **source-available, not open source** ("Symbolica Source-Available License 1.0").
- **Allowed (§3):** depending on an unmodified Symbolica as a Cargo dependency and distributing
  rmath. Requirements:
  - Ship a full copy of Symbolica's License.
  - Preserve all notices.
  - State in the docs that Symbolica runtime rights are not included; each end user needs their
    own Symbolica runtime license (unlicensed use runs on one core).
- **Not allowed:**
  - Using an AI system to analyze, explain or summarize Symbolica's source internals (§5(3)).
  - Using its source as a blueprint for re-implementing CAS functionality (§4(4)–(5)).
  - Redistributing its source (§4(1)).
  - Using Symbolica branding to suggest endorsement (§4(9)).
- **Working rule:** develop against Symbolica's *public API docs only* (docs.rs/symbolica and
  symbolica.io/docs). Never read, grep or analyze `.dev/symbolica/` with AI tools.
- "Better" means a better API layer on top (macros, ergonomics, type safety). rmath must
  never re-implement Symbolica's math engine.
- The MIT-licensed sub-crates `numerica` and `graphica` may be studied and reused, keeping
  the MIT notice.

### Version control
- `.dev` is currently fully gitignored. To share specs, ignore only `.dev/symbolica/` instead.
  The Symbolica clone must never be committed or pushed, since publishing its source is
  prohibited by §4(1).

### Design decisions (resolved 2026-10-07)
All eight brainstorming questions are settled and recorded in
`.dev/changes/rmath-v1-math-macros/design.md`: attribute syntax `t: Real + Positive`;
built-ins as prelude items; `0.5` float / `1/2` exact; no implicit multiplication; implicit
wildcards; `func!` freezes captures and returns `Result<impl FnMut>`; factorial `n!` kept;
`find!` included. Architecture decisions with long-term consequences live in `.dev/adr/`.

## Latest Analysis

**Change:** `rmath-v1-math-macros` (status: draft) — see `.dev/changes/rmath-v1-math-macros/`.

### Architecture summary
Two layers. **Compile time** (`rmath-macros`): Pratt parser over `proc_macro2` tokens → short-lived
AST → `quote!` expansion emitting only `$crate::__private::*` calls. **Run time** (`rmath`):
`IntoExpr`, `Callable<const N>`, prelude built-ins, `ApplyRule`/`Rule`, `solve`/`compile_f64`
helpers, all delegating to Symbolica's public API. No wrapper expression type (ADR-0002).

### Key patterns
- `<crate>` + `<crate>-macros` split (serde/serde_derive style)
- `#[doc(hidden)] pub mod __private` as the only surface generated code touches
- Const-generic `Callable<N>` + `#[diagnostic::on_unimplemented]` for compile-time arity checks
- Per-call-site generated struct inside a block, exposed via `impl Iterator` (`find!`)
- `trybuild` UI tests pin every diagnostic message

### Conventions extracted
- Vocabulary in `CONTEXT.md` is normative for specs, docs and identifiers
- Each phase starts with "verify on docs.rs" tasks for unconfirmed Symbolica API details
- License boundary (ADR-0001): never read `.dev/symbolica/` except `lib/numerica`, `lib/graphica`

### ADRs
- `0001-depend-on-symbolica-never-reimplement.md`
- `0002-direct-expansion-no-wrapper-type.md`
- `0003-explicit-compiler-checked-symbols.md`

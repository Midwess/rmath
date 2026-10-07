# Delta for Distribution

Packaging, licensing and repository hygiene for a public crates.io release.

## ADDED Requirements

### Requirement: Symbolica license compliance (§3)

The repository and published crate SHALL include a complete copy of the Symbolica
Source-Available License 1.0 as `LICENSE-SYMBOLICA.md`, preserve all Symbolica notices, and
state in `README.md` and the crate-level docs that Symbolica runtime rights are not included
and that each user needs their own runtime license (linking to symbolica.io/license).

#### Scenario: Published package contents

- WHEN `cargo package --list` is run for `rmath`
- THEN `LICENSE-SYMBOLICA.md` and `README.md` are in the list

#### Scenario: README notice

- WHEN a reader opens `README.md`
- THEN a section titled "Licensing" states that rmath depends on Symbolica, that runtime
  rights are not included, and links to `https://symbolica.io/license/`

### Requirement: rmath's own license

rmath's own code SHALL be licensed under the MIT license, with a `LICENSE` file and a
matching `license` field in both crates' `Cargo.toml`. (Owner decision, 2026-10-07.)

#### Scenario: Cargo metadata

- WHEN `cargo metadata` is inspected
- THEN both `rmath` and `rmath-macros` report `license = "MIT"`

### Requirement: Symbolica source never enters version control

The repository SHALL ignore `.dev/symbolica/` while allowing the rest of `.dev/` to be
committed.

#### Scenario: gitignore narrowed

- WHEN `git check-ignore .dev/symbolica/Cargo.toml .dev/project.md` is run
- THEN the first path is ignored and the second is not

### Requirement: Feature passthrough

`rmath` SHALL expose cargo features forwarding to Symbolica's numeric backends and allocator,
with `default-features = false` on the dependency: `default = ["gmp", "faster-alloc"]`,
`gmp`, `pure-rust`, `faster-alloc` (exact upstream feature names verified on docs.rs in
Phase 1).

#### Scenario: Pure-Rust build

- WHEN a user depends on `rmath = { version = "0.1", default-features = false, features =
  ["pure-rust"] }`
- THEN the crate builds without a system GMP library

### Requirement: MSRV and edition

Both crates SHALL declare `rust-version = "1.96"` and `edition = "2024"`.

#### Scenario: Older toolchain

- WHEN building with a toolchain older than 1.96
- THEN cargo reports the MSRV mismatch rather than a cryptic compile error

### Requirement: Documentation builds

`cargo doc --no-deps` SHALL succeed with no broken intra-doc links, and the README SHALL be
included as crate-level docs so its examples are doc-tested.

#### Scenario: README examples compile

- WHEN `cargo test --doc` runs
- THEN every code block in `README.md` compiles and passes (or is marked `ignore` with a reason)

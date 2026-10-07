//! Compile-fail tests: every diagnostic the macros emit is pinned to a `.stderr` snapshot.
//!
//! Regenerate snapshots after an intentional message change with
//! `TRYBUILD=overwrite cargo test --test ui`, then review the diff.

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

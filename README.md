# rmath

Write computer algebra as math, in Rust, checked by the compiler.

rmath adds native math-syntax macros on top of the [Symbolica](https://symbolica.io)
computer algebra system. Expressions are parsed at compile time and expanded into calls on
Symbolica's public API; the result is a plain Symbolica `Atom`, so everything Symbolica can
do stays available.

```rust
use rmath::prelude::*;

symbols!(x, y: Real, f: Symmetric);

let k = 3;
let e = expr!(k * x^2 + sin(y) / 2 - f(y, x));
assert_eq!(e, rmath::symbolica::parse!("3*x^2 + sin(y)/2 - f(x, y)"));
```

- **No runtime parsing.** `expr!` is a procedural macro; the generated code calls Symbolica
  directly.
- **Compiler-checked names.** Symbols are declared with `symbols!`. A typo is an ordinary
  `cannot find value` error with rustc's "did you mean" suggestion.
- **Rust values splice in.** Any integer, `f64`, `Rational`, `Symbol` or `Atom` in scope can
  be used by name; arbitrary Rust expressions go in braces: `expr!(x + {k + 1})`.
- **Real math precedence.** `^` is right-associative and binds tighter than unary minus, so
  `-x^2` is `-(x^2)` and `2^3^2` is `2^9`.

## Installation

```toml
[dependencies]
rmath = "0.1"
```

rmath depends on Symbolica, which needs a *runtime license* to run; see
[Licensing](#licensing). The default features use the GMP/MPFR numeric backends (system
libraries `gmp` and `mpfr` required). For a pure-Rust build:

```toml
rmath = { version = "0.1", default-features = false, features = ["pure-rust"] }
```

| Feature | Default | Effect |
|---|---|---|
| `gmp` | yes | GMP/MPFR integer and float backends (fastest) |
| `pure-rust` | no | Pure-Rust backends (`malachite`, `astro-float`); no system libraries |
| `faster-alloc` | yes | Installs `mimalloc` as the global allocator of the final binary |
| `native-codegen` | yes | Symbolica's JIT / native code generation support |

`gmp` and `pure-rust` are mutually exclusive.

## Macros

### `symbols!`

```rust
use rmath::prelude::*;

symbols!(x, y, z);                     // plain symbols
symbols!(t: Real + Positive);          // attributes, written like trait bounds
symbols!(g: Symmetric, h: Linear);     // functions are symbols too
symbols!(θ, κ);                        // any Rust identifier, Unicode included
symbols!(tau0 = "τ_0");                // Rust name ≠ math name
# let _ = (x, y, z, t, g, h, θ, κ, tau0);
```

Attributes: `Symmetric`, `Antisymmetric`, `Cyclesymmetric`, `Linear`, `Flat`, `Scalar`,
`Real`, `Integer`, `Positive` (Symbolica's `SymbolAttribute`). Symbols are interned globally
by name: `symbols!(x)` in two functions yields the same symbol. Names ending in `_` are
rejected, because Symbolica reserves them for wildcards.

If your crate's only Greek identifiers are single letters, rustc's
`mixed_script_confusables` lint may fire; add `#![allow(mixed_script_confusables)]`.

### `expr!`

```rust
use rmath::prelude::*;
use rmath::symbolica::parse;

symbols!(x, y, f);
let k = 2;
let a = parse!("x + 1");

assert_eq!(expr!(x^2 + 2*x + 1), parse!("x^2 + 2*x + 1"));
assert_eq!(expr!(k * f(x, y)), parse!("2*f(x, y)"));       // Rust value by name
assert_eq!(expr!(a^2 + {k + 1}), parse!("(x+1)^2 + 3"));   // Atom and a Rust block
assert_eq!(expr!(exp(I * x)), (Atom::i() * Atom::var(x)).exp());
assert_eq!(expr!(5!), Atom::num(120));
```

#### Grammar

| Precedence (low → high) | Syntax | Notes |
|---|---|---|
| 1 | `a + b`, `a - b` | left-associative |
| 2 | `a * b`, `a / b` | left-associative; `1/3` is an exact rational |
| 3 | `-a` | unary minus |
| 4 | `a ^ b` | **right**-associative: `2^3^2 = 2^9`; binds tighter than unary minus: `-x^2 = -(x^2)` |
| 5 | `a!` | factorial; exact for integers, `Γ(a + 1)` otherwise |
| 6 | `(a)`, `f(a, b)`, `{ rust }`, literals, names | |

- Integer literals are exact (`i64`, widening to `i128`; larger values must be spliced).
  Decimal and exponent literals (`0.5`, `1e3`) are floating-point. Write `1/2` for an exact
  half.
- Built-in functions come from the prelude as ordinary Rust items: `sin`, `cos`, `exp`,
  `log`, `sqrt`, `abs`; the constant `I` is the imaginary unit. They are arity-checked at
  compile time.
- There is **no implicit multiplication**: `2x` and `x y` are compile errors telling you where
  to insert `*`.
- Paths (`a::b`), method calls (`x.y`) and macro calls (`f!(x)`) are not math; wrap them in
  braces to splice them.

## Roadmap

Planned for v0.1: `rule!` / `find!` (pattern rewriting), `solve!` (equation systems) and
`func!` (compile an expression to a fast `f64` closure).

## Licensing

rmath itself is dual-licensed under MIT or Apache-2.0, at your option.

rmath depends on **Symbolica**, which is *source-available, not open source*
(Symbolica Source-Available License 1.0, included as `LICENSE-SYMBOLICA.md`).
**Symbolica runtime rights are not included or transferred by rmath.** Every user must
qualify under Symbolica's free-use terms or hold a runtime license: see
<https://symbolica.io/license/>.

Without a license key Symbolica runs on a single core and **aborts the process** when used
from several threads concurrently. In particular, run your test suite single-threaded, e.g.
with a `.cargo/config.toml`:

```toml
[env]
RUST_TEST_THREADS = "1"
```

## Contributing

rmath is a thin layer over Symbolica's public API and must stay that way: it never
re-implements computer-algebra functionality, and Symbolica's source is never consulted
(its license forbids using it as a development reference). See the architecture decision
records under `.dev/adr/`.

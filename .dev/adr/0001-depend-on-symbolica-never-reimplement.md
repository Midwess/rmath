# ADR-0001: Depend on Symbolica; never re-implement its functionality

**Date:** 2026-10-07 · **Status:** accepted

## Context

rmath wants "a better version" of writing computer algebra in Rust. The obvious temptation
for any future contributor is to read Symbolica's source (a clone sits in `.dev/symbolica/`)
and port or improve algorithms locally.

Symbolica 3.x is distributed under the *Symbolica Source-Available License 1.0*. It is not
open source. Relevant terms (own reading, not legal advice):

- §3 permits linking against an unmodified Symbolica as a Cargo dependency and distributing
  the result, provided the License is shipped, notices are preserved, and documentation states
  that runtime rights are not included.
- §4(4)–(5) prohibit using the source, or information derived from it, as a reference
  implementation or blueprint for a *Competing Product*, defined broadly enough to include any
  computer-algebra library, even free, internal or in another language.
- §5(3) prohibits using an AI system to analyse, explain or summarise the source.
- §7: breach terminates the license automatically.

The MIT-licensed sub-crates `numerica` and `graphica` are exempt from these restrictions.

## Decision

1. rmath is a **thin API layer**: macros, traits and ergonomics only. It never implements
   simplification, polynomial arithmetic, pattern matching, solving or evaluation itself.
2. rmath is developed against Symbolica's **public API documentation only**
   (docs.rs/symbolica, symbolica.io/docs). Nobody — human or AI tool — reads, greps or
   summarises `.dev/symbolica/` (except `lib/numerica` and `lib/graphica`).
3. `.dev/symbolica/` is never committed, published or redistributed.
4. Distribution complies with §3: `LICENSE-SYMBOLICA.md` bundled, README states runtime
   rights are not included and links to symbolica.io/license.

## Consequences

- "Better" is scoped to developer experience; performance and capability are Symbolica's.
- Unverified API details must be checked on docs.rs, never in source; this adds explicit
  verification tasks to each phase.
- If a needed feature is missing from Symbolica's public API, the options are: request it
  upstream, work around it with public API, or drop the feature. Not: read the source.
- Future contributors are bound by this ADR; it is linked from `CONTEXT.md`,
  `.dev/project.md` and the README's contributing section.

## Alternatives considered

- *Own CAS core inspired by Symbolica:* prohibited by §4/§5 and far beyond scope.
- *Own CAS from literature only:* legal, but a multi-year project with no advantage over
  depending on Symbolica for the stated goal (nicer syntax).

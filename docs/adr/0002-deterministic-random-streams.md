# ADR-0002: Derive independent deterministic random streams

> Status: Accepted
> Date: 2026-07-27
> Cycle: Era 01 / Cycle 01

## Context

One global random-number generator makes unrelated edits perturb an entire
history. Adding a cosmetic name draw could change mortality, harvests, and
politics, undermining debugging, golden seeds, and public demonstrations.

## Decision

Derive a 256-bit seed for each stable domain using BLAKE3 over a version label,
the little-endian root seed, and a stable domain name. Seed `ChaCha12Rng` with
the result.

The initial version label is `merra-rng-v1`. A change to its derivation or
generator requires a new label, an ADR, and explicit compatibility treatment.

## Consequences

Random domains remain reproducible and isolated. Systems must deliberately
select a domain, and new domains become persistent compatibility decisions.

## Alternatives considered

- One global stream was simpler but excessively coupled.
- Platform `DefaultHasher` output is not a stable simulation contract.
- Storing arbitrary numeric offsets is harder to review and evolve than named
  domains.

## Evidence

Core tests prove repeated domain streams match and differently named domains
derive different seeds.

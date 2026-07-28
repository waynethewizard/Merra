# ADR-0003: Keep the public site beside the simulator

> Status: Accepted
> Date: 2026-07-27
> Cycle: Era 01 / Cycle 01

## Context

Merra produces both a simulation and a public development chronicle. The site
needs to publish repository documentation and inspect selected deterministic
runs without creating a second copy of either source.

The first site does not need accounts, visitor-triggered simulation jobs, a
database, or a private application repository.

## Decision

Keep the public site in `site/` within this repository. Build it as a
static-first Next.js application that reads published prose from `docs/` and
curated evidence from `golden/` at build time.

The site treats the versioned manifest, summary, event stream, population
records, and chronicle as public data contracts. Interactive views execute in
the browser over committed evidence; they do not run the simulator.

Deploy the static export as an isolated service. Deployment credentials and
environment-specific values remain in the hosting provider, never in source.

## Consequences

Documentation, schema changes, evidence, and their presentation can be reviewed
atomically. Public contributors can inspect the entire publication path.

The site adds a Node.js toolchain beside Cargo, and CI must validate both. A
future live simulation laboratory will require separately designed API, worker,
quota, and storage boundaries rather than silently turning the static site into
a compute service.

## Alternatives considered

- A private website repository would conceal public presentation code and make
  evidence changes cross-repository work without protecting deployment secrets.
- A separate public repository would add release coordination before the site
  has independent maintainers or a genuinely separate lifecycle.
- Running simulations on page requests would introduce cost and abuse risks
  before the public explorer needs them.

## Evidence

The first build renders the current cycle record and The First Clock directly
from repository sources, passes content/schema validation, and exports without
a runtime backend.

# Changelog

All notable project-facing changes are recorded here. This file is concise by
design; experiments and implementation reasoning belong in the
[development chronicle](docs/devlog/).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Merra does not claim semantic-version compatibility before its public
interfaces stabilize.

## [Unreleased]

### Added

- Rust 2024 Cargo workspace with headless core, simulation, CLI, testkit, and
  repository automation crates.
- Deterministic smoke scenario with structured events and reproducible reports.
- Architecture, development chronicle, newsletter, licensing, security, and
  contribution foundations.
- Deterministic population initialization, stable person records, aging, and
  data-driven annual mortality over a canonical hundred-year scenario.
- Data-driven named seasons with validated calendar coverage, exact boundary
  events, and mortality outcomes invariant to caller-selected step sizes.
- Opt-in families with stable parentage, partnerships, surname-bearing
  households, scheduled childbirth, dissolution after death, and a
  four-generation genealogy inspector.
- Story-first and ANSI-free terminal field report with a derived Overview,
  resolved History, searchable/sortable people and households, union-aware
  lineage, stable-ID focus controls, and responsive snapshots.
- Golden century evidence, a multi-seed simulation laboratory, and GitHub
  Actions that publish reproducible run evidence and package Era releases.
- Static public project site with a development chronicle and interactive
  explorers for curated event evidence and the five-view terminal showcase.
- Railway container and deployment configuration for the public site.
- Deterministic `merra-worldgen` crate with tectonic, elevation, climate,
  hydrology, biome, resource, mythic-feature, place, and route passes over a
  canonical continent and separated island.
- Versioned surface-world and portable place-graph contracts, stable region,
  feature, route, population, lineage, culture, faith, institution, and polity
  identities, and independently derived world/history random streams.
- Aggregate Bevy macro-history schedule for population growth, migration,
  settlements, cultures, faiths, institutions, polities, navigation, first
  contact, mixed populations, cultural synthesis, faith spread, and schism.
- Data-driven human and orc founding populations with inherited physiology
  separated from learned culture and faith; no lineage-specific behavior
  branch is required.
- World-generation and historical CLI commands with complete machine evidence,
  chronicles, SVG atlases, and ANSI-free text atlases.
- Interactive world TUI with terrain, biome, habitability, resource, and mythic
  layers plus completed-history summaries.
- Canonical six-century first-contact golden evidence, exact regression tests,
  twenty-seed structural tests, an orbital-habitat portability fixture, and a
  reusable multi-world cohort laboratory.
- Public World Atlas page sourced from golden evidence, visualizing generation
  order, separate homelands, lineage parameters, learned cultures, first
  contact, contradictory lore, and the five-settlement handoff.
- ADRs and extensive Cycle 3 and Cycle 4 records covering world-first
  generation, theme portability, and the separation of lineage, culture,
  faith, and polity.
- Exact five-settlement macro-to-local projection with weighted household
  reconciliation, one residence per household, kin- and road-aware migration,
  located births and deaths, inherited institutions and lore, and event schema
  v3 residence evidence.
- Story-first five-village terminal inspector with consequence, shortest-path
  road matrix, settlements, migrations, and household-context views plus exact
  golden snapshots.
- Versioned five-village playback evidence and an interactive public-site
  network that replays 108 named lives through settlement, birth, death, and
  household migration events across four generations.
- Dedicated public History & Lore reader spanning Years 0–660, with
  event-referenced milestones, an explicit authoritative first-contact record,
  competing sourced cultural claims, and the exact macro and local chronicles.

### Changed

- GitHub Actions now use the current immutable-SHA releases of artifact
  transfer, Node setup, provenance attestation, and secret scanning, including
  stricter artifact digest verification and Node 24 action runtimes.
- The public-site toolchain now uses TypeScript 7 through Next.js's supported
  CLI compatibility path, OpenNext for Cloudflare 1.20, and current Node,
  React, and TSX type/build dependencies.

### Fixed

- Final family records now clear a dead person's current household membership,
  and households emptied by partnership-driven departures dissolve on the same
  simulation day instead of remaining temporarily active and empty.

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
- Interactive and ANSI-free terminal views for inspecting events and lives.
- Golden century evidence, a multi-seed simulation laboratory, and GitHub
  Actions that publish reproducible run evidence and package Era releases.
- Static public project site with a development chronicle and interactive
  explorer for curated, reproducible golden runs.
- Railway container and deployment configuration for the public site.

### Fixed

- Final family records now clear a dead person's current household membership,
  and households emptied by partnership-driven departures dissolve on the same
  simulation day instead of remaining temporarily active and empty.

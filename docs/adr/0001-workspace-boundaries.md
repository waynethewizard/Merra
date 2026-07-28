# ADR-0001: Begin with four product crates

> Status: Accepted
> Date: 2026-07-27
> Cycle: Era 01 / Cycle 01

## Context

The roadmap names many eventual systems and proposed a crate for nearly every
one. Those interfaces do not exist yet, and premature crates would create
dependency and coordination overhead without protecting known boundaries.

Era I needs a Bevy-independent model, headless ECS orchestration, a runnable
application, and shared test fixtures.

## Decision

Begin with `merra-core`, `merra-sim`, `merra-cli`, and `merra-testkit`, plus the
repository-only `xtask`.

Subsystems begin as modules. Extract one only when it has a stable interface,
multiple consumers, or a material compile-time benefit. Era II will introduce
graphical application and presentation boundaries without making simulation
depend on presentation.

## Consequences

The workspace has clear dependency direction without forcing speculative public
APIs. Later extraction will require deliberate moves, but those moves will be
informed by working causal slices.

## Alternatives considered

- One monolithic application crate would let Bevy and presentation types leak
  into persistent domain contracts.
- Creating every roadmap crate now would encode guesses as architecture.

## Evidence

The foundation smoke path passes from CLI to Bevy simulation to portable event
and report types without pulling rendering or windowing dependencies.

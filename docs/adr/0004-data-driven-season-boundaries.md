# ADR 0004: Data-Driven Season Boundaries

- Status: Accepted
- Date: 2026-07-28

## Context

Era I needs seasons before harvests, travel, fertility, and household labor can
be modeled. A calendar that stores only `days_per_year` cannot identify those
boundaries. Evaluating every system on arbitrary caller-selected steps would
also make simulation outcomes depend on whether a century was requested in one
operation or many.

Adding seasonal steps creates a specific determinism hazard: the existing
mortality table is annual. Running it once per season would change both death
probabilities and the mortality random stream.

## Decision

Each scenario defines an ordered list of seasons. Every season has a stable ID,
a display name, and a positive integer day length. Validation requires at least
one season, unique nonblank IDs, nonblank names, and a total length equal to
`days_per_year`.

`Simulation::advance` divides an explicit duration at every crossed season
boundary. The Bevy schedule then runs three ordered sets:

1. advance the clock and record the requested substep;
2. emit `SeasonBegan` when the new time is an exact boundary;
3. age living people and evaluate mortality only at a year boundary.

The mortality system accumulates elapsed days between annual evaluations.
Seasonal scheduling therefore records more evidence without consuming extra
mortality rolls. Uneven caller steps may create additional `TimeAdvanced`
events, but they must not change final people or death payloads.

## Consequences

- A whole-century request still exposes every named seasonal transition.
- Later seasonal systems receive exact, data-defined scheduling points.
- Golden event streams become larger but more causally legible.
- Calendar changes are history-changing scenario changes and must be reviewed
  like other simulation rules.
- Before the first tagged Cycle 1 contract, the scenario-v1 shape gained a
  required `seasons` field. After the tag, incompatible scenario changes will
  require an explicit migration or schema version.

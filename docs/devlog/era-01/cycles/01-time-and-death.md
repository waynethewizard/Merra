+++
era = 1
cycle = 1
slug = "time-and-death"
title = "Time and Death"
status = "in_progress"
started = "2026-07-27"
completed = ""
code_tag = ""
scenario = "scenarios/era-01/century.ron"
seeds = [42]
+++

# Era I / Cycle 1: Time and Death

## Question

Can Merra advance a century deterministically while people age, die, and leave
an inspectable causal history?

This cycle now answers the smallest useful version of the question: one
hundred initialized people can age through a century, die according to an
explainable integer mortality table, and leave byte-stable evidence.

## Intended Evidence

- A headless command that runs without rendering or window dependencies.
- Versioned structured events, a compact summary, and a readable chronicle.
- Identical deterministic output for repeated scenario, seed, duration, and
  source inputs.
- Independent random streams ready for mortality and later domains.

## Decisions and Rationale

- Rust-independent historical contracts live in `merra-core`; Bevy orchestration
  lives in `merra-sim`.
- The foundation uses stable IDs and an explicit calendar rather than Bevy
  entities or wall-clock time.
- Scenario configuration uses RON; machine output uses JSON and JSONL.
- Generated exploratory runs are ignored, while selected compact evidence may
  become a golden fixture.

## Implementation Notes

The Bevy plugin owns a custom deterministic schedule with explicitly ordered
time-advancement and mortality sets. A `Simulation` façade initializes
resources, runs the schedule, and produces reports without exposing the Bevy
`World` as a persistence interface.

Random stream seeds are derived with the versioned scheme documented in
`docs/architecture/determinism-and-replay.md`. Population shape, names, and
mortality use separate streams. Mortality processes living people in stable
`PersonId` order and compares a random integer against deaths-per-10,000; no
floating-point state participates in the authoritative decision.

The terminal inspector renders the completed report through Ratatui. Its
`TestBackend` can emit ANSI-free fixed-size screens, making the same interface
interactive locally, readable in pull requests, and exact in golden tests.

## Experiments and Results

Canonical century seed: `42`.

- 100 people initialized with starting ages from 0 through 70.
- 100 people died; the final death occurred in Year 77.
- 203 ordered events were emitted over 36,000 simulated days.
- Runa Barrow was the first recorded death, Alda Vale reached the greatest age
  at 92, and Leof Stone was the final recorded death.

A fixed cohort of seeds 1 through 100 produced extinction years from 73 through
94 and mean age-at-death values from 68.69 through 75.36 years. All 100 runs
ended with no living people because this intentionally narrow cycle has death
but not birth.

## Failures and Surprises

The first seed-laboratory report measured only living population at Year 100.
Every run returned zero, so the metric had no comparative signal. We preserved
that result and added extinction year and age-at-death ranges instead of
silently tuning the mortality table until the chart looked interesting.

The current mortality values are illustrative mechanics, not a historical or
demographic claim. Their value in this cycle is that the inputs and outcomes
are inspectable.

## Tests and Invariants

- Scenario schemas and calendars must validate before simulation.
- Identical reports must serialize to identical bytes.
- Random streams repeat for the same domain and differ across domains.
- Mortality bands must be ordered, bounded by 10,000, and cover every possible
  age.
- Mortality evaluation and death event order follow stable person identity.
- The seed-42 century summary, chronicle, event TUI, and people TUI are exact
  golden fixtures.
- A finished simulation cannot advance or finish again.
- The headless simulation dependency tree must exclude rendering and windowing.

## Reproduction

```sh
cargo merra run \
  --scenario scenarios/era-01/century.ron \
  --seed 42 \
  --years 100 \
  --output runs/century-seed-42

cargo tui --snapshot --view events
cargo tui --snapshot --view people
cargo xtask seed-lab --output runs/seed-lab
```

The command must create `manifest.json`, `events.jsonl`, `population.json`,
`summary.json`, and `chronicle.md` in a previously nonexistent output
directory.

## Known Limitations and Debt

- There are no births, households, relationships, locations, or causes of death.
- Everyone is guaranteed to die eventually under the final age band, making
  extinction an expected property rather than a discovered equilibrium.
- Mortality is checked at deterministic simulation steps and is not yet a
  calibrated survival model.
- The calendar has a year length but no named seasons yet.
- The TUI inspects completed evidence rather than a live or pausable world.

## Newsletter Candidates

- Why a historical simulation needs stable IDs separate from Bevy entities.
- How independent RNG streams keep a cosmetic name roll from changing a death.
- The distinction between an authoritative world event and a later historical
  claim.
- Why a statistically uninteresting result can reveal a better measurement.
- How one Ratatui renderer serves interaction, golden tests, and CI summaries.

## Next Questions

- How do births, partnerships, and household formation prevent deterministic
  extinction without hiding the causal story?
- Which details should a person inherit, learn, remember, or invent?
- What makes a century chronicle a history rather than a mortality report?

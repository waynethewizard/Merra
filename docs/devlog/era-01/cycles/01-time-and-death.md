+++
era = 1
cycle = 1
slug = "time-and-death"
title = "Time and Death"
status = "in_progress"
started = "2026-07-27"
completed = ""
code_tag = ""
scenario = "scenarios/era-01/smoke.ron"
seeds = [42]
+++

# Era I / Cycle 1: Time and Death

## Question

Can Merra advance a century deterministically while people age, die, and leave
an inspectable causal history?

The repository-foundation slice answers only the prerequisite: can the same
explicit calendar input produce byte-identical structured evidence?

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

The initial Bevy plugin owns a custom deterministic schedule containing one
ordered time-advancement set. A `Simulation` façade initializes resources,
runs the schedule, and produces reports without exposing the Bevy `World` as a
persistence interface.

Random stream seeds are derived with the versioned scheme documented in
`docs/architecture/determinism-and-replay.md`.

## Experiments and Results

Foundation smoke seed: `42`.

Expected result for one 360-day year: three ordered events representing start,
time advancement, and completion.

## Failures and Surprises

None recorded yet. Preserve failures here as mortality and population behavior
are introduced.

## Tests and Invariants

- Scenario schemas and calendars must validate before simulation.
- Identical reports must serialize to identical bytes.
- Random streams repeat for the same domain and differ across domains.
- A finished simulation cannot advance or finish again.
- The headless simulation dependency tree must exclude rendering and windowing.

## Reproduction

```sh
cargo merra run \
  --scenario scenarios/era-01/smoke.ron \
  --seed 42 \
  --years 1 \
  --output runs/foundation-smoke
```

The command must create `manifest.json`, `events.jsonl`, `summary.json`, and
`chronicle.md` in a previously nonexistent output directory.

## Known Limitations and Debt

- No population, aging, mortality, seasons, or accelerated century exists yet.
- The chronicle summarizes only the clock.
- Golden fixtures will be promoted only after the foundation output is stable.

## Newsletter Candidates

- Why a historical simulation needs stable IDs separate from Bevy entities.
- How independent RNG streams keep a cosmetic name roll from changing a death.
- The distinction between an authoritative world event and a later historical
  claim.

## Next Questions

- What mortality model is simple enough to explain but rich enough to produce
  different age structures?
- Which population invariants should fail loudly?
- What does a useful century-scale terminal chronicle show without overwhelming
  the reader?

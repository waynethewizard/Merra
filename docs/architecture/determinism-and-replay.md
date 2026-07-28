# Determinism and Replay

> Status: Accepted foundation
> Last reviewed: 2026-07-28

A Merra run is identified by its source version, dependency lockfile, scenario
bytes, schema versions, root seed, duration, and future player commands.

## Rules

1. Simulation time is explicit world state, never wall-clock time.
2. Persistent identities use typed stable IDs rather than Bevy `Entity`.
3. Stateful systems have explicit schedule ordering.
4. Order-dependent work sorts by stable IDs before making decisions or emitting
   events.
5. Randomness uses independently derived, versioned domain streams.
6. Hashing for simulation behavior never uses randomized platform hashers.
7. Floating-point state is avoided for authoritative rules unless its
   reproducibility boundary is documented and tested.
8. Output excludes timestamps, absolute paths, hostnames, and other machine
   state.

## Random streams

Each stream receives a 256-bit seed derived with BLAKE3 from:

```text
"merra-rng-v1\0" + root_seed_little_endian + "\0" + stable_domain_name
```

The result seeds `ChaCha12Rng`. Adding a cosmetic name draw therefore cannot
change mortality or weather. Changing the derivation scheme requires a new
version label and an ADR because it changes existing histories.

Initial demographics, names, and mortality already use separate domains.
Annual mortality checks collect living people and sort them by `PersonId`
before drawing or emitting events. That ordering is part of the reproducibility
contract even though Bevy is free to store entities in a different order.

Scenario calendars provide ordered, named seasons whose positive lengths must
exactly fill the configured year. A request to advance many days is divided at
those boundaries. The simulation therefore emits every season transition even
when a caller asks for a whole century in one operation.

Mortality remains annual. Seasonal scheduling accumulates age continuously but
draws from the mortality stream only at a year boundary, so adding seasons did
not silently quadruple a person's chance of death or consume extra random
numbers. Tests compare one large advance with uneven caller advances and require
equal person records and death payloads. The `TimeAdvanced` evidence may differ
because the caller's explicit requests are themselves recorded facts.

Family-enabled scenarios add a household stream independent of names,
mortality, and the reserved birth stream. Eligible unpartnered adults are
sorted by generation and `PersonId`; the first same-generation person who is
not a parent, child, or sibling becomes the deterministic partner. Household
surname choice uses the household stream. Given names continue from the name
stream after founder initialization.

Births currently follow explicit age, interval, child-count, and generation
limits rather than a probabilistic fertility claim. Every child receives the
next stable `PersonId`, two earlier parent IDs, its household surname, and one
generation greater than its parents.

Family report determinism covers the full events, people, households, summary,
and chronicle result. A fixed 100-seed regression cohort additionally checks
that current membership is bidirectional, dead people retain no current
household or partner, active households are nonempty, dissolved households are
empty, and every causal reference points to an earlier event.

## Run products

The headless runner writes:

- `manifest.json`: version and input identity;
- `events.jsonl`: stable structured facts;
- `population.json`: final inspectable person records;
- `summary.json`: compact deterministic measurements;
- `chronicle.md`: human-readable evidence.

Exploratory products live in ignored `runs/`. Selected golden runs must state
their exact reproduction command and remain small enough to review.

Replay from commands is a later capability. The foundation first guarantees
repeatable state transitions and output for identical explicit inputs.

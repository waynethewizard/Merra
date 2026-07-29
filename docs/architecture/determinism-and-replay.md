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

World generation extends the domain list without changing any existing stream:
tectonics, elevation, climate, hydrology, resources, mythic traces, places,
macro population, culture, faith, institutions, and world names each receive
their own stable label. Existing labels retain their original bytes, so adding
the generator does not rewrite the tagged family histories.

Authoritative world and macro-history calculations use integers. Geography
records each generation pass with a deterministic evidence hash. The world
manifest identifies template bytes, root seed, generator version, and complete
world hash. The history manifest independently identifies world bytes, history
configuration bytes, seed, and duration.

The macro-history Bevy schedule executes one historical year per update. Work
is ordered by stable population, location, route, culture, faith, institution,
and polity identifiers before decisions or event emission. Affiliation shares
use integer parts per 10,000 and must normalize exactly.

Canonical testing regenerates the entire surface and six-century history twice,
then compares the reports and selected golden artifacts. A twenty-seed
structural cohort additionally requires a continent, separated island, rivers,
places, locked route, completed history, backward-causal events, and a
five-settlement starting region in every run.

## Run products

The headless runner writes:

- `manifest.json`: version and input identity;
- `events.jsonl`: stable structured facts;
- `population.json`: final inspectable person records;
- `summary.json`: compact deterministic measurements;
- `chronicle.md`: human-readable evidence.

World generation additionally writes `world.json`, pass, feature and place
evidence, and SVG/text atlases. Macro-history writes its own manifest and event
stream plus populations, settlements, cultures, faiths, institutions, polities,
lore, important places, starting region, chronicle, and historical atlases.
The selected repository golden omits the multi-megabyte full world while
retaining exact commands and reviewable products.

Exploratory products live in ignored `runs/`. Selected golden runs must state
their exact reproduction command and remain small enough to review.

Replay from commands is a later capability. The foundation first guarantees
repeatable state transitions and output for identical explicit inputs.

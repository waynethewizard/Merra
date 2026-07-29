+++
era = 1
cycle = 5
slug = "five-villages"
title = "Five Villages"
status = "complete"
started = "2026-07-28"
completed = "2026-07-28"
code_tag = "era-01-cycle-05"
scenario = "scenarios/era-01/five-villages.ron"
seeds = [42]
+++

# Era I / Cycle 5: Five Villages

## Question

Can Merra turn the five-settlement handoff from aggregate history into
detailed household lives without losing population totals, geography,
institutions, or causal explanation?

The result is a weighted local sample. It is detailed enough to locate births,
deaths, households, and migration while retaining an exact bridge back to the
40,751 aggregate people present in the selected region in Year 600.

## Intended Evidence

- Exactly five stable locations inherited from the Cycle 4 starting region.
- Ten deterministic pairwise shortest paths over historically available
  routes, with travel cost, days, route IDs, and intermediate places.
- One authoritative residence for every household.
- Exact macro-population allocations across initial sampled households.
- Living-kin support, road distance, and an isolated seeded tie-break for each
  new household's destination.
- Births and deaths with authoritative locations and backward local causes.
- Household-scale culture, faith, institution, and contradictory lore
  references.
- Story-first Overview, Roads, Settlements, Migrations, and Households terminal
  views.

## Decisions and Rationale

- Household residence and projection semantics are fixed in
  [ADR 0008](../../../adr/0008-household-residence-and-local-projection.md).
- The existing detailed family simulation runs unchanged. A local-history
  projector replays its stable events into historical space, inserts typed
  residence decisions, and renumbers local causes. Earlier Cycle 1 and Cycle 2
  evidence therefore remains byte-stable.
- A sampled household receives an exact amount of aggregate population
  evidence. That weight is not copied when descendants form new households.
- Residence selection is lexicographic rather than a blended score. A reviewer
  can say whether kin, distance, or a seeded tie actually decided the result.
- Pairwise road output is a cost matrix and shortest-path tree, not a false
  geographic sketch. The place graph does not contain local drawing
  coordinates.
- Food, housing capacity, employment, congestion, travel-in-progress, and
  seasonal route closures remain outside this cycle.

## Implementation Notes

`RegionalHistoryV1` is the durable Cycle 4 handoff. The `history` CLI now emits
`regional-history.json`, including selected populations, places, relevant
macro events and lore, culturally connected institutions, and routes open in
Year 600.

`run_local_history` executes the configured person-and-family scenario, assigns
the epoch households proportionally across the five places, and splits each
aggregate cohort exactly across those households. It then replays the event
stream. When a household forms, the projector:

1. records each founder's previous household residence;
2. counts living parents, children, and siblings by candidate location;
3. calculates shortest paths over open routes;
4. chooses kin, then road cost, then the migration-domain rank;
5. emits `HouseholdSettled` with a complete causal explanation.

People do not acquire independent residence state. The event replay resolves a
person's current household and copies that household's residence to births,
deaths, partnerships, and dissolution. The final report contains people,
households, events, historical contexts, residence decisions, connections,
settlements, institutions, lore, summary, and chronicle.

The terminal inspector reads that one report and provides five views. Stable
`--focus-settlement` and `--focus-household` flags support direct evidence
links. Selecting a settlement and pressing Enter filters the household view to
that place.

## Experiments and Results

Canonical world, history, and local seed: `42`. Projection: Year 600. Detailed
duration: 60 years.

- 40,751 aggregate people represented exactly as 40,751 weighted people.
- 30 initial detailed people in 15 households.
- 78 detailed births, 34 deaths, and 74 living people at the end.
- 37 post-projection residence decisions; 22 crossed a settlement boundary.
- 316 of 800 local events carry an authoritative place.
- Ten pairwise selected-settlement connections.
- The longest selected journey costs 82 and takes 164 configured days.
- All 22 boundary-crossing decisions were led by living kin in Seed 42.
  Unit fixtures separately exercise the road-cost and seeded-tie fallbacks.

The useful story is comparative:

- **Fenstead** grew from 12 to 37 sampled residents.
- **Junipercross** grew from 4 to 18.
- **Yarrowmere** grew from 4 to 17.
- **Alderholm** fell from 6 to 2.
- **Fenholm** fell from 4 to 0 after four births, three deaths, no arrivals,
  and five departures.

Fenholm's disappearance is not a missing row. Its births, deaths, former
households, routes, and macro origin remain inspectable after no sampled
household remains.

The integrated seeds 1 through 20 all preserve exact projection, located vital
events, five connected settlements, and local migration. At least one produces
an empty sampled village, which protects divergence without making Seed 42's
outcome a hard-coded deliverable.

## Failures and Surprises

The first summary called every post-projection residence decision a migration,
including a new household that stayed in its founders' settlement. The final
contract separates 37 residence decisions from 22 actual boundary crossings.

The selected settlements contain no institution headquarters in Seed 42.
Filtering institutions only by physical location therefore made a completed
historical system disappear from household scale. The handoff now retains
institutions connected through represented cultures, while settlement records
still distinguish physically local institutions.

A pictorial ASCII road map would have implied coordinates the portable graph
does not provide. The terminal now presents exact shortest paths and a cost
matrix. That is less decorative and more truthful.

The canonical run never reaches the road or seeded fallback because all 22
migrations have differentiated living-kin support. Deterministic synthetic
fixtures are required to prove those rules independently.

## Tests and Invariants

- Identical world, regional handoff, local configuration, and seed produce
  equal complete reports.
- Exactly five distinct selected locations resolve in the source world.
- Every pair has an available deterministic shortest path.
- Every aggregate person is allocated exactly once at the projection boundary.
- Every local household has exactly one residence.
- Person location is derived from household residence.
- Every birth and death has a location and a backward residence cause.
- Local event IDs are contiguous; every cause refers to an earlier event.
- Every residence decision records origins, destination, travelers, reason,
  routes, travel cost, and days.
- Kin outranks distance; distance outranks the isolated seeded rank.
- Household contexts retain culture, faith, institution, and lore evidence.
- Canonical summaries, settlement records, connections, chronicle, and all
  five 120×36 TUI views exactly match committed golden evidence.
- The compact playback projection retains all 108 named sampled people and the
  164 ordered settlement, birth, and death events used by the public
  generation-by-generation visualization.
- Compact rendering contains no ANSI escapes and gives an explicit minimum-size
  fallback.

## Reproduction

```sh
cargo merra worldgen \
  --template scenarios/era-01/before-memory.ron \
  --seed 42 \
  --output runs/before-memory-42

cargo merra history \
  --world runs/before-memory-42 \
  --scenario scenarios/era-01/first-histories.ron \
  --seed 42 \
  --years 600 \
  --output runs/first-histories-42

cargo merra villages \
  --history runs/first-histories-42 \
  --scenario scenarios/era-01/five-villages.ron \
  --seed 42 \
  --output runs/five-villages-42

cargo tui villages --input runs/five-villages-42
cargo tui villages \
  --input runs/five-villages-42 \
  --snapshot \
  --view roads

cargo test --locked -p merra-sim -p merra-testkit -p merra-tui
```

## Known Limitations and Debt

- The detailed people are a weighted sample, not a statistically calibrated
  synthetic population.
- Household matching is not yet constrained by road distance; geography
  currently decides the new household's residence after a partnership forms.
- Households settle immediately. Travel has cost and duration evidence but no
  in-progress state, route risk, or seasonal interruption.
- Kin support counts close living kin equally. It does not distinguish care,
  hostility, wealth, age, obligation, or household capacity.
- Institutions and claims are inherited references, not person-level
  knowledge, membership, office, or belief.
- Settlement growth has no food, work, housing, land, disease, or resource
  constraint. Those pressures begin in Cycle 6.
- The local projector remains a module in `merra-sim`; future systems may
  justify a more explicit detailed-region schedule.

## Newsletter Candidates

- Why 30 people can represent 40,751 without pretending to be them.
- One residence per household as a schema decision.
- Why the road view is a matrix instead of an invented ASCII map.
- How a village can disappear while its history remains queryable.
- The bug hidden inside the phrase "37 migrations."
- Why culture can carry an institution into a region where it has no office.

## Next Questions

- Which harvest and storage differences make Fenholm's loss materially
  consequential rather than only demographic?
- How should food pressure compete with kin support in a migration decision?
- When should a journey become an in-progress household state?
- Which historical claims does an individual actually know, and from whom?

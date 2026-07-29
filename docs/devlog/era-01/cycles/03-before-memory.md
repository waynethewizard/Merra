+++
era = 1
cycle = 3
slug = "before-memory"
title = "Before Memory"
status = "complete"
started = "2026-07-28"
completed = "2026-07-28"
code_tag = "era-01-cycle-03"
scenario = "scenarios/era-01/before-memory.ron"
seeds = [42]
+++

# Era I / Cycle 3: Before Memory

## Question

Can Merra generate a complete physical context before it places historical
populations, while remaining fast, deterministic, inspectable, and portable to
settings that do not have medieval terrain?

The result is a coarse world substrate rather than a finished geological
model. It gives later history constraints and affordances without trying to
simulate every remote person at local resolution.

## Intended Evidence

- A deterministic continent and a genuinely separated island.
- Explicit tectonic, elevation, climate, hydrology, biome, resource, mythic,
  place, and route passes.
- Rivers that follow an acyclic drainage graph to the ocean.
- Candidate places selected from habitability and history-facing affordances.
- One maritime route that exists geographically but begins unavailable.
- A portable place graph which can also describe an orbital habitat.
- SVG and ANSI-free terminal atlases generated from authoritative output.
- Stable pass evidence, compact summaries, and a fixed multi-seed cohort.

## Decisions and Rationale

- World context precedes population history; see
  [ADR 0006](../../../adr/0006-world-first-place-graph.md).
- The generator is a separate `merra-worldgen` crate with no Bevy dependency.
  Bevy schedules are useful for historical state transitions, not required for
  deterministic array and graph construction.
- `SurfaceWorldV1` retains cells for visualization and future material
  questions. `PlaceGraphV1` is the portable boundary consumed by history.
- Geography provides generic tags such as `primary_homeland` and
  `isolated_homeland`. It does not contain `human_home` or `orc_home`.
- Every authoritative calculation uses integers. The canonical grid is
  128×96—large enough to expose drainage and separation defects while remaining
  cheap enough for cohort tests.
- Passes use independent deterministic random domains. Adding a later world
  naming draw cannot silently move a river or change a resource deposit.
- Mythic traces are generated before historical populations. They are
  unexplained affordances whose later meanings belong to cultures and faiths.

## Implementation Notes

The generator starts with deterministic moving plate seeds and combines plate
pressure, integer noise, a configured land fraction, and a separated-island
mask into elevation. Climate derives latitude temperature and moisture.
Hydrology computes each non-ocean cell's downstream neighbor, prevents
drainage cycles, accumulates upstream flow, and marks high-flow cells as rivers.

Biome classification consumes landform, temperature, moisture, and water.
Resources combine geology and biome rather than placing generic loot. Mythic
motifs select stable surface cells after the physical passes. Place selection
then scores habitability, access, resources, rivers, coasts, and mythic traces,
while enforcing spatial separation.

The route builder connects nearby places across land and ensures the graph is
usable. It adds one locked maritime route between the separated regions. That
route is real in the world substrate, but history must earn navigation before
populations can use it.

The CLI writes:

- `world.json`;
- `manifest.json`;
- `summary.json`;
- `passes.json`;
- `features.json`;
- `places.json`;
- `atlas.svg`;
- `atlas.txt`.

The TUI can load the same directory and switch among terrain, biome,
habitability, resource, and mythic layers. Snapshot mode uses the interactive
renderer without ANSI control codes.

## Experiments and Results

Canonical world seed: `42`.

- 12,288 surface regions.
- 5,898 land regions.
- 471 separated-island regions.
- 786 river regions.
- 6 represented biomes.
- 15 prehuman mythic features.
- 30 candidate historical places.
- 43 routes.
- 1 locked maritime route.

Generation is byte-stable for identical template and seed. The selected golden
evidence commits summaries, atlases, and later history products rather than the
5.4 MB full world file. The complete world remains exactly reproducible from
the template and seed.

Seeds 1 through 20 all generated a continent, separated island, rivers, places,
and locked route. The same cohort then completed 600 historical years, making
the geography checks evidence for an integrated system rather than attractive
maps in isolation.

## Failures and Surprises

The most important change was conceptual rather than algorithmic. The earlier
roadmap warned against expanding geography before local depth worked. That was
correct for fully detailed actor simulation but too strong for causal context:
five villages hand-placed in a void would make every route, resource, and ruin
arbitrary. The replacement is a coarse world first and selective local detail
later.

The first draft also tried to make terrain select a human and orc homeland.
That coupled a fantasy lineage to a surface generator and failed the setting
portability test. Generic homeland tags leave the historical scenario in
control.

Rendering each region as a separate verbose SVG node made evidence needlessly
large. Grouping contiguous biome runs keeps the canonical atlas around 100 KB
without hiding the underlying grid.

## Tests and Invariants

- Identical configuration and seed produce equal worlds, hashes, pass evidence,
  summaries, SVG atlases, and TUI snapshots.
- The canonical world contains both a main landmass and separated island.
- Every surface ID, feature ID, place ID, and route ID is stable and unique.
- Drainage is acyclic and river cells have accumulated upstream flow.
- Every route resolves to known locations.
- At least one route is locked and requires navigation.
- The place graph contains three primary-homeland candidates and one isolated
  homeland candidate.
- Every configured mythic motif produces its requested features.
- TUI snapshot output contains no ANSI escapes.
- Seeds 1 through 20 satisfy the structural world and integrated history
  invariants.

## Reproduction

```sh
cargo merra worldgen \
  --template scenarios/era-01/before-memory.ron \
  --seed 42 \
  --output runs/before-memory-42

cargo tui world \
  --input runs/before-memory-42 \
  --layer terrain

cargo tui world \
  --input runs/before-memory-42 \
  --snapshot \
  --layer mythic

cargo test --locked -p merra-worldgen
cargo test --locked -p merra-testkit canonical_world_history_matches_golden_evidence
```

## Known Limitations and Debt

- Plate motion, climate, hydrology, and ecology are intentionally suggestive
  integer models, not scientific simulations.
- The surface is a square grid without spherical topology, erosion, glaciers,
  changing sea level, groundwater, or geological eras.
- Rivers are drainage evidence rather than navigable spatial networks.
- Resources have no reserves, depletion, renewal, ownership, or trade yet.
- Routes have no path geometry, travel time, seasonal access, or construction
  history.
- Place names use a temporary fantasy lexicon. Theme-specific naming must be a
  separate data layer.
- The generator creates one canonical continent-and-island composition. Other
  templates need stronger morphology controls.

## Newsletter Candidates

- Why five villages need a world even when the game cannot render one yet.
- What Bevy does—and deliberately does not do—in a Rust world generator.
- How domain-separated RNG keeps a naming change from moving a river.
- Why the 5.4 MB canonical world is reproducible but not committed.
- Turning a square grid into a portable place graph for orbital habitats.
- The atlas as a test artifact rather than concept art.

## Next Questions

- Can aggregate populations create distinct histories without becoming racial
  stereotypes or deterministic cultures?
- Will navigation emerge late enough for isolated histories to matter?
- What evidence should survive when global history becomes five local
  settlements?

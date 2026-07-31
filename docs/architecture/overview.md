# Architecture Overview

> Status: Accepted foundation
> Last reviewed: 2026-07-28

Merra separates historical meaning from engine orchestration and presentation.

```text
world template ─▶ merra-worldgen ─▶ SurfaceWorldV1
                                      │
                                      ▼
                                 PlaceGraphV1
                                      │
history config ───────────────────────┤
                                      ▼
                              RegionalHistoryV1
                                      │
local config ─────────────────────────┤
                                      ▼
merra-cli ───────┐                merra-sim ─────▶ merra-core
merra-tui ───────┤                    ▲
future game ─────┘────────────────────┘

merra-testkit ───▶ worldgen + simulation + portable contracts
xtask ───────────▶ repository and cohort workflows
public site ─────▶ selected golden evidence
```

## Boundaries

### `merra-core`

Owns portable values and rules: stable identifiers, validated named-season
calendars, scenarios, event schemas, output schemas, and deterministic
random-stream derivation. It does not depend on Bevy. A different runtime
should be able to consume its data contracts.

### `merra-sim`

Owns the authoritative Bevy `World`, simulation schedules, plugins, resources,
components, and the façade used to run a world. It depends on focused Bevy
crates rather than rendering or windowing.

Subsystems begin as modules inside this crate. A subsystem becomes a crate only
when it has a stable interface, independent consumers, or a meaningful compile
cost. The roadmap's conceptual plugin list is not a mandate for one crate per
concept.

The macro-history schedule consumes `PlaceGraphV1`, not surface terrain. It
advances aggregate populations, settlements, cultures, faiths, institutions,
polities, capabilities, and historical events one year at a time. This is a
different resolution from the person-and-household schedule, but both use
stable domain identities and structured causal evidence.

Its configuration contains data-defined founders rather than permanent
`human_cultures` or `orc_faith` fields. Each founder selects a lineage, homeland
tag, and culture. Faith and lore sources reference culture keys. A third
lineage can therefore enter a theme without changing the contract.

The local-history projector consumes the selected `RegionalHistoryV1` handoff,
not the full surface grid. It runs the existing detailed person-and-household
schedule, reconciles aggregate cohorts exactly across sampled epoch
households, and replays stable events into place. Household residence is the
authoritative local state; people derive their current place from it.

Item-enabled scenarios add stable durable objects after initial households
form. The simulation owns identity, provenance, condition, legal ownership,
and custody without requiring geography. The local projector resolves item
events through custody, inserts settlement escheat and relocation evidence
where necessary, and never infers location from legal ownership.

### `merra-worldgen`

Owns deterministic, Bevy-independent construction of physical context:
tectonics, integer elevation, climate, acyclic drainage, rivers, biomes,
resources, prehuman features, candidate places, and routes. It produces both a
surface world and a portable place graph.

The crate also renders SVG and ANSI-free text atlases. Those are views over the
generated world rather than authoritative inputs.

### Applications and presentation

`merra-cli` is Era I's headless batch composition root. `merra-tui` is an
optional terminal inspector over completed simulation evidence; it does not
participate in authoritative world updates. Era II will add a graphical
application and presentation boundary. Simulation crates must not depend on
rendering, UI, audio, or platform windows.

The default terminal application is a unified historical observatory. It
indexes the immutable surface world, optional macro history, and optional local
history into typed cross-scale identities and four presentation workspaces:
Atlas, Chronicle, Relations, and Catalog. The canonical launch generates the
seed-42 stack in memory; custom launches read and hash-check existing run
artifacts. Its combined timeline advances macro history by recorded event and
the focused local period by exact year. Neither path reruns or mutates history
after the evidence has been loaded.

The observatory derives reversible yearly presentation indexes from immutable
playback evidence. These indexes contain exact living-person locations and
recorded household migration trails; family trees derive only from stable
parent identities visible at the selected year. Reserved typed media wells are
presentation slots keyed by domain identity, not new simulation state.
The versioned media registry validates captions, alt text, provenance, safe
manifest-relative paths, file availability, and optional content hashes before
the presentation layer can resolve an asset.

The terminal inspector may derive population series, generation and surname
outcomes, partnership histories, and household timelines from an immutable
`SimulationReport`. Those are presentation indexes, not competing world state.
Interactive and ANSI-free snapshot modes share the same renderer and stable
domain-ID focus controls.

The legacy world mode reads completed world or historical run directories. It can inspect
terrain, biome, habitability, resource, and mythic layers and summarize the
macro-history handoff. It never advances either simulation.

The legacy five-village mode reads `LocalHistoryReportV1`. Its consequence overview,
shortest-path road matrix, settlement comparison, causal migration list, and
household historical context are all views over immutable output. It never
changes residence or recalculates the simulation.

The current schedule orders time advancement, season transition, annual
mortality, and family maintenance explicitly. Large advances are split at
data-defined season boundaries before that schedule runs. On year boundaries,
death is resolved before partnerships end, households change, and new births
are created.

### Test and development tooling

`merra-testkit` contains reusable fixtures and invariant helpers but no
production behavior. `xtask` automates repository policy using Rust so the same
workflows run across supported platforms.

## Capability, configuration, state

- Code defines capabilities such as whether time advances or obligations decay.
- Versioned scenario data configures how a setting uses those capabilities.
- The Bevy world and event stream record what happened in one run.

Setting-specific parameters must not become permanent engine law, and Bevy
entity identifiers must not become serialized historical identities.

The current portability boundary is deliberate:

```text
surface generator ─┐
orbital fixture ───┼─▶ PlaceGraphV1 ─▶ shared macro-history
future themes ─────┘
```

A theme can provide desert basins, space habitats, religious medieval
provinces, or another topology. Shared history code sees locations, routes,
affordances, capabilities, and stable tags.

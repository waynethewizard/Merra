# ADR 0006: Generate the World Before Historical Populations

- Status: Accepted
- Date: 2026-07-28

## Context

The original Era I sequence moved directly from households to five local
villages. That is enough to test settlement mechanics, but it leaves every
later culture, route, ruin, resource pressure, and boundary dependent on
hand-authored local context. Merra's larger premise requires the playable
region to be the consequence of a world that existed before its inhabitants.

Generating every individual across an entire world at playable resolution
would be expensive and would make local work wait on global detail. We need
world-scale causes without pretending every remote household receives the same
simulation resolution.

The history engine must also survive different themes. A medieval continent, a
desert realm, and an orbital colony should not require separate migration and
institution engines.

## Decision

Merra generates a coarse, immutable world substrate before it seeds historical
populations:

```text
tectonics and elevation
→ climate and hydrology
→ biomes and resources
→ mythic traces
→ places, affordances, and routes
→ population and culture seeds
→ macro-history
→ selected five-settlement starting region
```

`merra-worldgen` owns the deterministic, Bevy-independent surface generator.
Its authoritative product is `SurfaceWorldV1`. The simulation does not consume
terrain cells directly. It consumes a portable `PlaceGraphV1` containing stable
locations, routes, tags, and affordances.

The canonical surface template produces one main landmass, a separated island,
drainage, rivers, resources, prehuman traces, candidate places, land routes,
and one initially locked maritime route. Named homeland tags select founding
areas. They do not name a lineage in the geography contract.

Macro-history runs at aggregate population resolution. Detailed people and
households remain the local simulation's responsibility. After the historical
age, Merra selects a five-settlement region and its relevant event evidence for
future playable detail.

The same macro-history engine must run against a non-terrain fixture. An
orbital habitat graph is the initial portability test.

## Consequences

- The first playable villages can inherit migration, contact, faith, and
  institutional history rather than beginning in an empty scenario.
- Themes can replace the world producer while retaining the place-graph and
  history contracts.
- World generation remains pure Rust and cheap to test without Bevy schedules.
- Global history is deliberately aggregate. It cannot answer person-level
  questions until a region is promoted to detailed simulation.
- Place tags and affordances become a durable API and must remain descriptive
  rather than encoding genre- or lineage-specific behavior.
- A generated map is not sufficient evidence. Pass summaries, stable hashes,
  graph invariants, event streams, atlases, and cohort measurements are part of
  the feature.

# Historical Event Model

> Status: Accepted foundation
> Last reviewed: 2026-07-28

Every meaningful authoritative change should be capable of emitting a
structured world event.

```text
WorldEventV1
├── stable event ID
├── simulation time
├── event kind
├── actors and location
├── causal event IDs
├── searchable tags
└── typed payload
```

World events describe what the simulation says happened. They are not the
player-facing truth and are not themselves memories, rumors, chronicles, or
claims. Later systems will derive those representations through witnesses,
transmission, incentives, preservation, distortion, and forgetting.

## Requirements

- Event identifiers are monotonic within a run and stable for identical input.
- Causes reference earlier events; missing or forward references are invalid.
- Actors and locations use stable domain IDs, never Bevy `Entity`.
- Payloads are typed and versioned rather than arbitrary JSON maps.
- Tags support inspection but do not carry authoritative rules.
- Event ordering follows simulation time and then stable emission order.

An event schema change that breaks old readers increments the schema version.
The run manifest records the event and scenario schema versions used. Cycle 1
time, season, and mortality runs remain event schema v1; family-enabled runs
declare event schema v2 because household, partnership, and birth variants are
new exhaustive event categories.

The first implemented causal chain is deliberately small:

```text
SimulationStarted
→ PopulationInitialized
→ SeasonBegan(Thaw)
→ TimeAdvanced(to next boundary)
→ SeasonBegan
  ├→ PersonDied (at an annual boundary)
  ├→ PartnershipEnded
  ├→ HouseholdDissolved
  ├→ HouseholdFormed → PartnershipFormed
  ├→ PersonBorn
  └→ next TimeAdvanced
→ SimulationCompleted
```

Each season transition cites the `TimeAdvanced` event that reached its exact
boundary. Deaths at a year boundary cite the new season event, preserving the
ordered claim that time advanced and the new year began before annual mortality
was resolved. Death emission order follows stable person identity, not Bevy
query order.

Family events preserve the difference between kinship, partnership, and
co-residence. A birth cites the season boundary and the partnership event for
its household. Its actors include both parents and the new child. A
partnership ending cites the death that ended it; it does not erase the birth
events that preserve earlier parentage.

Events support debugging, causal inspection, golden tests, historical records,
future replay tooling, and the concrete stories used in development writing.

## Macro-history events

World generation records stable pass summaries and hashes but does not pretend
that plate placement is a witnessed human event. Once populations are seeded,
the aggregate history uses a parallel `HistoricalEventV1` contract:

```text
HistoricalEventV1
├── stable event ID and historical year
├── typed subjects: population, culture, faith, institution, polity, feature
├── optional stable location
├── backward causal event IDs
├── searchable tags
└── typed payload
```

Its initial categories cover population and settlement founding, migration,
culture, faith, institutions, polities, route opening, first contact, mixed
populations, faith spread and schism, abandonment, and completion.

Route opening is capability-based. A surface-world adapter may emit
`SeaRouteOpened`; an abstract graph emits `RouteOpened`. First contact must cite
the earlier opening event and cannot occur merely because two homelands exist
in the same scenario.

`LoreClaimV1` is explicitly not another authoritative event. It identifies its
source culture and optional faith, references stable events, and records
confidence. The canonical first contact therefore supports two incompatible
claims without making the event stream contradictory.

The person-level and macro-history schemas are currently separate because they
operate at different resolutions. Projecting a five-settlement macro region
into people and households will require an explicit handoff rather than
silently treating a population cohort as an individual actor.

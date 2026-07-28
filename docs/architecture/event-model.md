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
The run manifest records the event and scenario schema versions used.

The first implemented causal chain is deliberately small:

```text
SimulationStarted
→ PopulationInitialized
→ SeasonBegan(Thaw)
→ TimeAdvanced(to next boundary)
→ SeasonBegan
  ├→ PersonDied (at an annual boundary)
  └→ next TimeAdvanced
→ SimulationCompleted
```

Each season transition cites the `TimeAdvanced` event that reached its exact
boundary. Deaths at a year boundary cite the new season event, preserving the
ordered claim that time advanced and the new year began before annual mortality
was resolved. Death emission order follows stable person identity, not Bevy
query order.

Events support debugging, causal inspection, golden tests, historical records,
future replay tooling, and the concrete stories used in development writing.

# Architecture Overview

> Status: Accepted foundation
> Last reviewed: 2026-07-28

Merra separates historical meaning from engine orchestration and presentation.

```text
merra-cli ───────┐
merra-tui ───────┤
                 ▼
             merra-sim ─────▶ merra-core
                 ▲
future game ─────┘

merra-testkit ───▶ merra-sim + merra-core
xtask ───────────▶ repository workflows only
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

### Applications and presentation

`merra-cli` is Era I's headless batch composition root. `merra-tui` is an
optional terminal inspector over completed simulation evidence; it does not
participate in authoritative world updates. Era II will add a graphical
application and presentation boundary. Simulation crates must not depend on
rendering, UI, audio, or platform windows.

The terminal inspector may derive population series, generation and surname
outcomes, partnership histories, and household timelines from an immutable
`SimulationReport`. Those are presentation indexes, not competing world state.
Interactive and ANSI-free snapshot modes share the same renderer and stable
domain-ID focus controls.

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

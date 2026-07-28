# Architecture Overview

> Status: Accepted foundation
> Last reviewed: 2026-07-27

Merra separates historical meaning from engine orchestration and presentation.

```text
merra-cli ───────┐
                 ▼
             merra-sim ─────▶ merra-core
                 ▲
future game ─────┘

merra-testkit ───▶ merra-sim + merra-core
xtask ───────────▶ repository workflows only
```

## Boundaries

### `merra-core`

Owns portable values and rules: stable identifiers, calendar values, scenarios,
event schemas, output schemas, and deterministic random-stream derivation. It
does not depend on Bevy. A different runtime should be able to consume its data
contracts.

### `merra-sim`

Owns the authoritative Bevy `World`, simulation schedules, plugins, resources,
components, and the façade used to run a world. It depends on focused Bevy
crates rather than rendering or windowing.

Subsystems begin as modules inside this crate. A subsystem becomes a crate only
when it has a stable interface, independent consumers, or a meaningful compile
cost. The roadmap's conceptual plugin list is not a mandate for one crate per
concept.

### Applications and presentation

`merra-cli` is Era I's headless composition root. Era II will add a graphical
application and presentation boundary. Simulation crates must not depend on
rendering, UI, audio, or platform windows.

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

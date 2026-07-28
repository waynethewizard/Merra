# Determinism and Replay

> Status: Accepted foundation
> Last reviewed: 2026-07-27

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

## Run products

The headless runner writes:

- `manifest.json`: version and input identity;
- `events.jsonl`: stable structured facts;
- `summary.json`: compact deterministic measurements;
- `chronicle.md`: human-readable evidence.

Exploratory products live in ignored `runs/`. Selected golden runs must state
their exact reproduction command and remain small enough to review.

Replay from commands is a later capability. The foundation first guarantees
repeatable state transitions and output for identical explicit inputs.

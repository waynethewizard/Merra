# Canonical Four-Generation Dynasty

This is the review baseline for Era I / Cycle 2:

```sh
cargo merra run \
  --scenario scenarios/era-01/dynasty.ron \
  --seed 42 \
  --years 60 \
  --output runs/dynasty-seed-42

cargo tui \
  --scenario scenarios/era-01/dynasty.ron \
  --seed 42 \
  --years 60 \
  --snapshot \
  --view overview

cargo tui --snapshot --view lineage --focus-person 1
cargo tui --snapshot --view households --focus-household 1
```

The compact evidence protects a four-generation result without committing the
complete population, household, and event reports. The five terminal fixtures
cover the outcome overview, story-first history, biographies, Garin Thorn's
union-aware lineage, and the first Thorn household's reconstructed history.
CI can regenerate larger products from the scenario, seed, duration, and source
revision.

The tagged source revision for this baseline is `era-01-cycle-02`. The
scenario, summary, chronicle, and terminal snapshot are original Merra
material covered by the repository's source and documentation licenses; they
contain no third-party assets.

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
  --view genealogy
```

The compact evidence protects a four-generation result without committing the
complete population, household, and event reports. CI can regenerate those
larger products from the scenario, seed, duration, and source revision.

The tagged source revision for this baseline is `era-01-cycle-02`. The
scenario, summary, chronicle, and terminal snapshot are original Merra
material covered by the repository's source and documentation licenses; they
contain no third-party assets.

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

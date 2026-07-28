# Canonical First Century

This is the compact review baseline for Era I / Cycle 1:

```sh
cargo merra run \
  --scenario scenarios/era-01/century.ron \
  --seed 42 \
  --years 100 \
  --output runs/century-seed-42

cargo tui --snapshot --view events
cargo tui --snapshot --view people
```

The summary and chronicle are exact golden tests. TUI snapshots are checked by
the terminal crate and provide a readable preview in source review. Full event
and population files are generated and attached by GitHub Actions rather than
committed here. The baseline contains 904 events: initialization, four exact
season transitions per year, annual mortality, and completion.

# Foundation Smoke Golden Run

This evidence is generated from the repository foundation with:

```sh
cargo merra run \
  --scenario scenarios/era-01/smoke.ron \
  --seed 42 \
  --years 1 \
  --output runs/foundation-smoke
```

The event stream, summary, and chronicle are deterministic golden files. The
manifest is intentionally excluded because source revision and dirty-state
provenance vary across commits.

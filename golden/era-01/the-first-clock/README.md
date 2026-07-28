# The First Clock

The foundation golden run proves deterministic calendar advancement and the
versioned event/output contracts.

```sh
cargo merra run \
  --scenario scenarios/era-01/smoke.ron \
  --seed 42 \
  --years 1 \
  --output runs/foundation-smoke
```

The committed artifacts were generated from the source revision recorded in
`manifest.json`. The scenario hash covers the exact input bytes.

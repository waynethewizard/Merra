# First Histories — Seed 42

Canonical evidence for the world-first pivot: a generated temperate continent,
a remote orc island, and six hundred years of aggregate history.

Regenerate the evidence:

```sh
cargo merra worldgen \
  --template scenarios/era-01/before-memory.ron \
  --seed 42 \
  --output runs/before-memory-42

cargo merra history \
  --world runs/before-memory-42 \
  --scenario scenarios/era-01/first-histories.ron \
  --seed 42 \
  --years 600 \
  --output runs/first-histories-42

cargo tui world \
  --input runs/first-histories-42 \
  --snapshot \
  --layer terrain
```

The full `world.json` remains a generated run artifact rather than committed
repository weight. This curated directory preserves compact summaries,
chronicle, structured cultures and faiths, the selected starting region,
terminal atlas, and browser-ready SVG evidence.

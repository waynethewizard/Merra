# Five Villages — Seed 42

Canonical Cycle 5 evidence for the exact macro-to-local handoff, household
residence decisions, road costs, births and deaths by place, and inherited
historical context.

Regenerate it after producing the Cycle 4 history:

```sh
cargo merra villages \
  --history runs/first-histories-42 \
  --scenario scenarios/era-01/five-villages.ron \
  --seed 42 \
  --output runs/five-villages-42

cargo tui villages \
  --input runs/five-villages-42 \
  --snapshot \
  --view overview
```

The compact committed evidence proves that 40,751 aggregate people are
allocated exactly across the initial household sample. During the following
60 detailed years, Fenstead grows from 12 to 37 sampled residents while
Fenholm falls from 4 to 0. The full person, event, and household files remain
reproducible run artifacts.

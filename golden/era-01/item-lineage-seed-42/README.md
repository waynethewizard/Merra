# Item Lineage / Seed 42

This compact evidence set traces durable working tools across authoritative
use, repair, inheritance, custody, place, and identity-changing rework.

Reproduce it after generating the canonical Seed 42 world and history:

```sh
cargo merra villages \
  --history runs/first-histories-42 \
  --scenario scenarios/era-01/item-lineage.ron \
  --seed 42 \
  --output runs/item-lineage-42

cargo tui villages \
  --input runs/item-lineage-42 \
  --snapshot \
  --view items \
  --width 120 \
  --height 36
```

The selected run contains 60 stable item identities, 15 active final
descendants, 135 repairs, 45 transformations, 40 ownership transfers, and four
item generations. `items.json` is the final provenance graph; lifecycle facts
remain in the reproducible local event stream rather than being duplicated
here.

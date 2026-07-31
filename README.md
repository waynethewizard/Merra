# Merra

Merra is an open-source historical simulation game built with Rust and Bevy.
Its long-term goal is a world that creates causal history, remembers it
imperfectly, and lets the player live inside the result.

Era I is deliberately headless. Merra can now generate a deterministic physical
world before placing aggregate populations into it, run separate human and orc
histories through first contact, and project five historically meaningful
settlements into household-scale local history.

## Run the foundation smoke scenario

Install Rust through `rustup`; the repository automatically selects its pinned
stable toolchain.

```sh
cargo merra run \
  --scenario scenarios/era-01/smoke.ron \
  --seed 42 \
  --years 1 \
  --output runs/smoke
```

The new output directory contains a manifest, JSONL event stream, population
and household records, machine summary, and Markdown chronicle. The scenario's
named seasons are data, and each exact boundary appears in the causal event
stream. `runs/` is intentionally ignored.

## Repository guide

- [`site/`](site/) is the static public project site and golden-run explorer.
- [`docs/design-principles.md`](docs/design-principles.md) defines the
  simulation's non-negotiable design tests.
- [`docs/roadmap.md`](docs/roadmap.md) describes the long-range Eras and cycles.
- [`docs/architecture/`](docs/architecture/) records current technical
  boundaries.
- [`docs/devlog/`](docs/devlog/) holds the extensive cycle-by-cycle build
  chronicle.
- [`docs/newsletter/`](docs/newsletter/) holds Era-level Rusting newsletter
  sources.
- [`CHANGELOG.md`](CHANGELOG.md) is the concise release-facing change record.

See [`CONTRIBUTING.md`](CONTRIBUTING.md) before submitting changes.

## Explore the historical observatory

Launch the connected, interactive view of the canonical seed-42 world:

```sh
cargo tui
```

The observatory generates one causal stack in memory: the physical world,
600 years of aggregate history, and the exact Year 600–660 five-village
projection with households, named people, heirlooms, and competing lore. Its
four workspaces are complementary views over those same stable identities:

- **Atlas** overlays terrain, routes, settlements, and local population.
- **Chronicle** joins macro milestones to exact local events on one timeline.
- **Relations** follows typed links or visualizes a time-aware family tree for
  a selected person or household.
- **Catalog** searches and inspects the complete archive by record type.

Use `1`–`4` to switch workspaces, arrows or `hjkl` to move, `Enter` to follow
evidence, `/` to search all named records, `,`/`.` to step backward or forward
one year, and `[`/`]` to jump between events. `Space` plays forward and `r`
plays backward; repeating either key pauses. The macro era advances by recorded
event and the detailed local era advances year by year.

During Years 600–660 the Atlas plots exact living-person counts, highlights a
focused person, and draws that year's household migration trails. Press `p` to
cycle through residents at the selected settlement and `Enter` to open the
person's family tree. In Relations, `g` toggles between the family tree and the
general typed network. Family trees reveal births and deaths at the selected
year rather than leaking future descendants. Entity detail panes reserve stable
portrait, family, object, place, culture, event, and lore image wells keyed by
typed identity for future media.

`L` changes the Atlas layer, `+`/`-` zooms, and `?` opens the complete keyboard
and mouse guide.

Portable snapshots use the same renderer and contain no ANSI escapes:

```sh
cargo tui --snapshot --workspace atlas
cargo tui --snapshot --workspace chronicle --year 600
cargo tui --snapshot --workspace relations --focus person:17
cargo tui --snapshot --workspace catalog --focus item:1
```

Use `--theme monochrome`, the standard `NO_COLOR` environment variable, or
`--no-motion` for restrained terminal environments. Existing generated
artifacts can replace the canonical in-memory stack:

```sh
cargo tui --world runs/before-memory-42
cargo tui --history runs/first-histories-42
cargo tui \
  --history runs/first-histories-42 \
  --local runs/item-lineage-42
```

The original focused inspectors remain available as explicit subcommands. For
example, the Cycle 2 dynasty and Cycle 1 century are directly reproducible:

```sh
cargo tui dynasty
cargo tui dynasty \
  --scenario scenarios/era-01/century.ron \
  --seed 42 \
  --years 100 \
  --view history
```

The headless report for a family-enabled scenario also includes
`households.json`; the terminal views resolve stable event, person, partner,
descendant, and household identities without exposing Bevy entity IDs.

Run a fixed multi-seed cohort with:

```sh
cargo xtask seed-lab --output runs/seed-lab
```

GitHub Actions repeats the canonical century and dynasty on changes, exposes
their story-first terminal overviews in the job summary, and runs the 100-seed
laboratories each week.

## Generate a world before its people

The canonical world has one main landmass, a separated island, rivers,
resources, prehuman traces, thirty candidate places, and a maritime route that
history cannot use until navigation develops:

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
```

Inspect the completed world or history with:

```sh
cargo tui world --input runs/first-histories-42
cargo tui world \
  --input runs/first-histories-42 \
  --snapshot \
  --layer biome
```

World mode cycles terrain, biome, habitability, resources, and mythic layers.
The macro-history uses a portable place graph rather than terrain-specific
rules; an orbital habitat fixture runs through the same Bevy schedule.

Run a deterministic cohort of complete generated histories with:

```sh
cargo xtask world-lab \
  --first-seed 1 \
  --count 20 \
  --years 600 \
  --output runs/world-lab
```

The canonical seed's reviewable evidence is in
[`golden/era-01/first-histories-seed-42/`](golden/era-01/first-histories-seed-42/).
The public site's World Atlas page is generated from those same checked
artifacts. Its History & Lore reader follows the canonical record from Year 0
through the detailed Year 660 outcome while preserving the distinction between
authoritative events and the competing claims cultures make about them.

## Enter the five villages

Project the selected Year 600 region into 60 years of detailed households:

```sh
cargo merra villages \
  --history runs/first-histories-42 \
  --scenario scenarios/era-01/five-villages.ron \
  --seed 42 \
  --output runs/five-villages-42
```

The handoff reconciles all 40,751 aggregate inhabitants exactly across the
initial weighted household sample. New households choose one residence through
living-kin support, shortest road cost, and an isolated seeded tie-break.
Births and deaths retain authoritative places, while household contexts carry
culture, faith, institutions, and competing historical claims.

The run also writes `playback.json`, a compact, versioned projection of the
named people and settlement, birth, and death events required to replay the
local history without turning the website into a second simulation.

Inspect the consequence, roads, settlements, migrations, and households:

```sh
cargo tui villages --input runs/five-villages-42
cargo tui villages \
  --input runs/five-villages-42 \
  --snapshot \
  --view overview
```

Seed 42 makes the comparison legible: Fenstead grows from 12 to 37 sampled
residents while Fenholm falls from 4 to 0. Exact compact evidence is in
[`golden/era-01/five-villages-seed-42/`](golden/era-01/five-villages-seed-42/).
The public Five Villages page uses that same evidence to animate all four
generations year by year.

## Trace a working heirloom

Run the item-enabled local scenario against the same Year 600 handoff:

```sh
cargo merra villages \
  --history runs/first-histories-42 \
  --scenario scenarios/era-01/item-lineage.ron \
  --seed 42 \
  --output runs/item-lineage-42

cargo tui villages \
  --input runs/item-lineage-42 \
  --view items
```

Durable tools receive stable identities independently from their owners.
Meaningful work wears them, condition changes their effective labor, repairs
preserve identity, and reforging creates descendants with typed source links.
The item inspector separates authoritative provenance, legal ownership,
physical custody, and location while the Cycle 5 scenario remains byte-stable.

## Run the public site

The site reads published prose from `docs/` and selected deterministic evidence
from `golden/`; those remain the canonical sources.

```sh
npm install
npm run site:dev
```

Use `npm run site:validate`, `npm run site:check`, and `npm run site:build`
before publishing. `npm run site:build:sites` also verifies the managed
OpenNext artifact used by the connected public deployment.

## License

Rust source code is available under either the [MIT License](LICENSE) or the
[Apache License 2.0](LICENSE-APACHE), at your option.

Original documentation, prose, images, and other creative material are
available under [Creative Commons Attribution 4.0](LICENSE-CC-BY-4.0), unless a
file or [`assets/ATTRIBUTION.md`](assets/ATTRIBUTION.md) says otherwise.

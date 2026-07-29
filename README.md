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

## Explore a generated history

Open the interactive Cycle 2 field report:

```sh
cargo tui
```

The default seed-42 dynasty runs for 60 years and opens on a derived Overview:
population shape, generation survival, surname outcomes, household totals, and
Garin Thorn's three partnerships. `Tab` or `1`–`5` switches among Overview,
History, People, Lineage, and Households. Use arrow keys or `j`/`k` to move,
`/` to search, `s` to sort, `f` to reveal History filters including the full
clock/debug stream, and `Enter`, `h`, or `e` to follow related evidence.

Portable snapshots use the same renderer and contain no ANSI escapes:

```sh
cargo tui --snapshot --view overview
cargo tui --snapshot --view history
cargo tui --snapshot --view people
cargo tui --snapshot --view lineage --focus-person 1
cargo tui --snapshot --view households --focus-household 1
```

The Cycle 1 century remains directly reproducible:

```sh
cargo tui \
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

# Merra

Merra is an open-source historical simulation game built with Rust and Bevy.
Its long-term goal is a world that creates causal history, remembers it
imperfectly, and lets the player live inside the result.

The project is in its repository-foundation stage. Era I is deliberately
headless: it will prove deterministic time, people, households, resources, and
succession before the visible village arrives.

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

## Review a generated century

Open the interactive Events, People, and Genealogy inspector:

```sh
cargo tui
```

Use arrow keys or `j`/`k` to navigate, `Tab` to switch collections, and `q` to
quit. For a portable review snapshot:

```sh
cargo tui --snapshot --view events
cargo tui --snapshot --view people
```

Inspect the current four-generation Cycle 2 history with:

```sh
cargo tui \
  --scenario scenarios/era-01/dynasty.ron \
  --seed 42 \
  --years 60 \
  --view genealogy
```

The headless report for a family-enabled scenario also includes
`households.json`; the genealogy view resolves stable parent, partner,
descendant, and household identities without exposing Bevy entity IDs.

Run a fixed multi-seed cohort with:

```sh
cargo xtask seed-lab --output runs/seed-lab
```

GitHub Actions repeats the canonical century on changes, exposes its chronicle
in the job summary, and runs the 100-seed laboratory each week.

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

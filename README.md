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

The new output directory contains a manifest, JSONL event stream, machine
summary, and Markdown chronicle. `runs/` is intentionally ignored.

## Repository guide

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

## License

Rust source code is available under either the [MIT License](LICENSE) or the
[Apache License 2.0](LICENSE-APACHE), at your option.

Original documentation, prose, images, and other creative material are
available under [Creative Commons Attribution 4.0](LICENSE-CC-BY-4.0), unless a
file or [`assets/ATTRIBUTION.md`](assets/ATTRIBUTION.md) says otherwise.

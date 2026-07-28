# Contributing to Merra

Merra is built in narrow, causal slices. A contribution should make its outcome
observable and explain why the added complexity matters.

## Before opening a pull request

1. Read `docs/design-principles.md` and the current Era record.
2. Keep Bevy-free domain rules in `merra-core` and presentation out of
   `merra-sim`.
3. Add or update tests for behavior and invariants.
4. Update `CHANGELOG.md` for project-facing behavior.
5. Update the active cycle record with decisions, experiments, failures, and
   reproduction instructions.
6. Add an ADR when changing a durable architectural boundary.
7. Run `cargo xtask verify-docs` and `cargo xtask preflight`.

`preflight` requires `cargo-deny` and `gitleaks` to be installed:

```sh
cargo install cargo-deny --locked --version 0.20.2
```

Install Gitleaks from its official release packages for your platform.

## Generated evidence

Exploratory output belongs under ignored `runs/`. Commit only small, intentional
golden files or publication media. Every committed artifact must document the
exact scenario, seed, command, source revision, and license.

Never commit `.env` files, tokens, private keys, credentials, private
correspondence, or unredacted terminal and editor screenshots. See
`SECURITY.md` for reporting.

## Dependency changes

Use the newest stable crates compatible with the pinned stable toolchain. Avoid
Git dependencies and prereleases without an accepted ADR. Bevy breaking
upgrades are dedicated cycles rather than incidental feature changes.

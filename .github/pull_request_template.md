## Question and outcome

What causal or technical question does this address, and what can a reviewer
observe afterward?

## Evidence

- [ ] Tests protect the changed behavior or invariant.
- [ ] Reproduction commands, scenarios, seeds, and outputs are documented.
- [ ] The active cycle record includes decisions, experiments, and failures.
- [ ] `CHANGELOG.md` is updated when behavior changed.
- [ ] Durable architecture changes have an ADR.

## Public-repository safety

- [ ] No secret, `.env` file, credential, private data, or local path is present.
- [ ] Terminal output and screenshots are scrubbed.
- [ ] New assets and copied material have compatible licenses and attribution.
- [ ] Generated exploratory runs remain outside version control.
- [ ] `cargo xtask preflight` passes.

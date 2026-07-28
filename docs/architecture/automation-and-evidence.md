# Automation and Public Evidence

> Status: Accepted foundation
> Last reviewed: 2026-07-28

GitHub Actions is part of Merra's simulation laboratory, not only a build gate.

## Continuous integration

Every push and pull request checks the pinned toolchain on Linux, macOS, and
Windows. Formatting, Clippy, documentation, scenario-note structure, dependency
policy, and secret scanning are independent visible gates.

## Canonical century

The Simulation Laboratory reruns `scenarios/era-01/century.ron` with seed 42,
verifies golden behavior, renders the Overview, History, and People views, and
places the resulting chronicle and story-first overview in the job summary.
The complete run is a short-lived workflow artifact rather than repository
noise.

Cycle 2 adds a parallel canonical-dynasty job. It regenerates the seed-42
four-generation history, publishes the terminal overview in the job summary,
and attaches the complete people, household, event, chronicle, history,
lineage, and household-screen evidence.

The summary is evidence, not an approval oracle. Reviewers should ask whether
the causal result remains legible, not only whether its bytes remain stable.

## Scheduled seed cohort

The Wednesday seed laboratory evaluates seeds 1 through 100 for one hundred
years. It publishes aggregate lifespan and extinction ranges and attaches CSV
and JSON evidence. Fixed cohorts make changes comparable; exploratory larger
cohorts remain local or manually dispatched.

A second scheduled cohort evaluates 100 dynasty seeds for 60 years. In addition
to population and lifespan ranges, the generic seed laboratory now measures
births, households formed, generations reached, and distinct surnames. This
turns "the golden family tree still looks right" into a broader stability check
without treating one seed as a demographic model.

The testkit also runs those same 100 dynasty seeds through structural
invariants on every repository test pass. It requires reciprocal living
partnerships, exact parent generations, bidirectional current household
membership, empty dissolved households, typed and backward-causal events, and
the published cohort ranges. The scheduled laboratory remains the inspectable
CSV/JSON artifact; the test is the fast regression gate over the same cohort.

Statistical gates should be added only after a behavior has an intentional
acceptable range. Early cohort output informs tuning without pretending the
first model is correct.

## Era releases

Pushing an intentional `era-*` tag builds CLI and TUI bundles on all supported
platforms. Each bundle includes scenarios, selected golden evidence, newsletter
sources, and licenses. The release job uses GitHub's short-lived token with
write permission only in the final release job. Every platform archive receives
a GitHub build-provenance attestation through short-lived OIDC credentials, and
the release includes SHA-256 checksums.

## Secret and artifact policy

- Workflows receive read-only repository permission unless a release requires
  more.
- Checkout credentials are not persisted in simulation or packaging jobs.
- No pull-request workflow receives maintainer secrets.
- Generated manifests contain source identity but no environment variables,
  absolute paths, hostnames, or credentials.
- Gitleaks scans committed history, while GitHub push protection blocks known
  provider credentials before acceptance.
- Artifacts are retained briefly and contain only reproducible public data.

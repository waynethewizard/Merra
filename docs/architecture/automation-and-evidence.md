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

Cycles 3 and 4 add a canonical world-history job. It generates the seed-42
surface, runs 600 years of aggregate history, renders the historical SVG and
ANSI-free world TUI, verifies golden contracts, and publishes the chronicle,
world measurements, first-contact year, and atlas as one review surface. The
full run remains a short-lived artifact.

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

The weekly and manual laboratory also runs a 100-world cohort through
`cargo xtask world-lab`. Its CSV and JSON report geography, population,
settlement, culture, faith, institution, contact, and mixing outcomes. The
initial twenty-seed local baseline reached first contact in every run between
Years 286 and 302; this is documented as a tuning limitation, not blessed as a
permanent acceptable range.

The canonical job catches exact drift. The cohort catches structural collapse.
The public site makes one selected history understandable. These are
complementary evidence layers rather than three copies of the same assertion.

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
- The public site embeds only checked-in, script-free generated SVG and public
  JSON/text evidence. Content validation rejects an atlas containing script
  elements or terminal control codes.

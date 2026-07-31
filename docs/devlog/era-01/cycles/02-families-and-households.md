+++
era = 1
cycle = 2
slug = "families-and-households"
title = "Families and Households"
status = "complete"
started = "2026-07-28"
completed = "2026-07-28"
code_tag = "era-01-cycle-02"
scenario = "scenarios/era-01/dynasty.ron"
seeds = [42]
+++

# Era I / Cycle 2: Families and Households

## Question

Can Merra preserve understandable kinship while people form and lose
partnerships, move between households, inherit surnames, and create four
generations of descendants?

The completed implementation reaches that observable target. It remains a
deliberately explicit family mechanism rather than a demographic claim.

## Intended Evidence

- A family-enabled scenario that leaves the tagged Cycle 1 scenarios unchanged.
- Stable parent, partner, household, birth, death, and generation identities.
- Typed events for household formation and dissolution, partnerships, and
  childbirth.
- A headless `households.json` report alongside population and event evidence.
- A story-first terminal overview plus searchable history, people, lineage,
  household, and fixed-size golden screens.
- A reproducible seed with at least four generations and reviewable invariants.

## Decisions and Rationale

- Kinship, current partnership, and household membership are separate state;
  see [ADR 0005](../../../adr/0005-kinship-partnership-and-household.md).
- We use "partnership" rather than "marriage" until culture, law, religion, and
  property make marriage a meaningful simulated institution.
- Family behavior is an additive, opt-in scenario-v1 field. Missing family data
  defaults to disabled, preserving Cycle 1 reports and golden evidence.
- Family-enabled manifests declare event schema v2. The scenario shape is
  backward-compatible, but exhaustive household, partnership, and birth event
  variants are not honestly a v1 event stream.
- Founders are generation 0. Every simulated child has two earlier stable
  parent IDs and a generation exactly one higher than its parents.
- A new household chooses one partner's surname through an isolated household
  random stream. Partners retain their personal names; children inherit the
  household surname.
- Pairing is deterministic: eligible adults are ordered by generation and
  stable identity, remain within a generation, and cannot pair with a parent,
  child, or sibling.

## Implementation Notes

The Bevy schedule now runs family maintenance after annual mortality. A death
therefore becomes authoritative before it ends a partnership or empties a
household. Family maintenance then forms eligible partnerships and finally
creates scheduled births.

People carry stable personal and family evidence: given name, surname, birth
day, parent IDs, current household, current partner, and generation.
Households are ECS entities with separate stable `HouseholdId` values, current
members, surname, founding and dissolution days, birth spacing, and child
counts. Neither output contract serializes Bevy `Entity`.

Final membership is genuinely current state: death clears the person's current
household and partnership references, and a household emptied when its last
member forms a new household dissolves in the same family-maintenance pass.
The departure-caused dissolution cites the new partnership event rather than
waiting for an unrelated future season boundary.

The CLI writes `households.json` for every run; it is empty when families are
disabled. Ratatui presents five views over the completed report. Overview
derives outcomes rather than replaying raw records. History defaults to
meaningful population and family events, while its all/debug filter preserves
the complete clock and season stream. People, Lineage, and Households resolve
stable identities and reconstruct historical unions and membership without
mutating or duplicating authoritative simulation state.

## Experiments and Results

Canonical dynasty seed: `42`, duration: 60 years.

- 16 eighteen-year-old founders formed 8 initial households.
- 49 children were born, producing 65 people total.
- 45 people remained alive and 20 had died at Year 60.
- 31 households formed.
- Generations 0, 1, 2, and 3 contained 16, 16, 18, and 15 people respectively.
- The run emitted 644 ordered events.
- Mara Mere recorded the first death at age 2 in Year 28.
- Leof Marsh reached age 78, the longest completed life at the boundary.
- Cerdic Oak recorded the final death in Year 60.

A fixed cohort of seeds 1 through 100 also ran for 60 years:

- All 100 runs reached four generations.
- Births ranged from 34 through 50.
- Households formed ranged from 22 through 34.
- Living population at Year 60 ranged from 31 through 51, with a mean of
  42.49.
- Distinct surnames represented at the end of each complete record ranged from
  8 through 13.

The golden showcase opens on Garin Thorn. It resolves his founder status,
children Mara Thorn and Garin Thorn under his first union with Garin Gorse,
later partnership with Runa Oak, and current partnership with Garin Fen. The
overview also makes the exact generation cohorts visible, shows Fen surviving
across G0–G3, and identifies Gorse as extinct at the Year 60 boundary.

The terminal showcase was completed as a presentation follow-up to the tagged
Cycle 2 simulation. It changes no scenario, schema, event, population,
household, summary, or chronicle contract: every displayed result is derived
from the same report used by the headless evidence writers.

## Failures and Surprises

The first design temptation was to make a `Family` component contain parents,
partner, surname, and household. That would make leaving home or losing a
partner look like rewriting ancestry. Separating the concepts made both the
events and inspector much easier to explain.

A strict two-child limit per household does not imply two children per person.
After death ends a partnership, a survivor may form another household whose
own child counter begins at zero. The canonical generations therefore contain
slightly different cohort sizes rather than a perfect binary tree.

The current title was changed from a generic invented house name to "Thorn and
Fen" after the actual seed evidence showed which surnames the inspector made
prominent. Scenario prose should follow the simulation rather than predeclare
its story.

The completion audit found two final-state bugs that the canonical genealogy
screen did not reveal. Dead people were removed from household member lists but
still retained a `household_id`, contradicting the field's current-state
contract. Separately, an adult leaving a singleton household could leave that
household active and empty until the next annual maintenance pass. Both defects
were fixed before the tag, with focused regression scenarios for death and
departure.

## Tests and Invariants

- Family age thresholds must be ordered and enabled limits must be positive.
- Family-enabled manifests declare event schema v2 while family-disabled
  Cycle 1 scenarios still declare event schema v1.
- Disabled family rules preserve the Cycle 1 smoke and century golden evidence.
- Birth parents have stable IDs lower than their child.
- The canonical run reaches generation 3 and exactly matches summary,
  chronicle, and genealogy-screen fixtures.
- Current partnerships are symmetric.
- Partnerships stay within one generation and do not pair siblings.
- Every living person belongs to exactly one active household, every household
  member resolves back to that household, and dead people have no current
  household or partner.
- Active households cannot be empty, dissolved households cannot retain
  members, and a departure that empties a household dissolves it immediately.
- Every causal event reference points backward.
- Event IDs are contiguous, event time is nondecreasing, actors resolve to
  stable people, and event kinds match their typed payloads.
- Repeated canonical reports are equal across events, people, households,
  summaries, and chronicles.
- Every one of the fixed 100 dynasty seeds satisfies the same family, household,
  event, and summary invariants; the aggregate ranges are exact regression
  evidence.
- Existing annual mortality and caller-step invariants remain green.

## Reproduction

```sh
cargo merra run \
  --scenario scenarios/era-01/dynasty.ron \
  --seed 42 \
  --years 60 \
  --output runs/dynasty-seed-42

cargo tui dynasty \
  --view overview

cargo tui dynasty \
  --snapshot \
  --view lineage \
  --focus-person 1

cargo tui dynasty \
  --snapshot \
  --view households \
  --focus-household 1

cargo xtask seed-lab \
  --scenario scenarios/era-01/dynasty.ron \
  --first-seed 1 \
  --count 100 \
  --years 60 \
  --output runs/dynasty-seed-lab

cargo test --locked -p merra-testkit dynasty_cohort_preserves_family_invariants
cargo xtask verify-docs
cargo xtask preflight
```

The headless command creates `manifest.json`, `events.jsonl`,
`population.json`, `households.json`, `summary.json`, and `chronicle.md` in a
previously nonexistent directory.

## Known Limitations and Debt

- Births are scheduled from explicit age, spacing, household-child, and
  generation limits; fertility, pregnancy, sex, health, preference, and chance
  are not modeled.
- The current system models partnerships, not culturally or legally defined
  marriage.
- Pairing avoids direct parent-child and sibling relationships but does not yet
  calculate cousin distance or a complete ancestor closure.
- Household child limits reset when a new household forms.
- Partner choice is deterministic and same-generation; there are no courtship,
  preference, conflict, divorce, adoption, guardianship, or plural households.
- Surnames belong to people and households but have no cultural naming rules.
- The final household report intentionally contains current rather than
  time-sliced membership; the inspector reconstructs its historical timeline
  from authoritative events.
- The family implementation should move out of the growing simulation root
  module when a later cycle needs to extend its internal system boundary.

## Newsletter Candidates

- Why `PersonId`, `HouseholdId`, and Bevy `Entity` are three different things.
- Why marriage is not a synonym for two IDs in a component.
- How a death event ends a partnership without changing a child's parents.
- The surprising difference between "two children per household" and "two
  children per person."
- How stable parent IDs become an ANSI-free family-tree artifact in CI.
- Why the story title changed after inspecting the seed.

## Next Questions

- Which birth and partnership rules should become probabilistic without making
  the canonical family tree unreadable?
- How should kinship distance constrain partnerships across cousins and clans?
- When does a household name become a lineage name, and when can it vanish?
- How should adoption, guardianship, remarriage, and step-relationships appear
  without corrupting ancestry?
- What place do these households inhabit when Cycle 3 adds five villages?

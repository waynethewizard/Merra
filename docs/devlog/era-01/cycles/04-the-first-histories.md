+++
era = 1
cycle = 4
slug = "the-first-histories"
title = "The First Histories"
status = "complete"
started = "2026-07-28"
completed = "2026-07-28"
code_tag = "era-01-cycle-04"
scenario = "scenarios/era-01/first-histories.ron"
seeds = [42]
+++

# Era I / Cycle 4: The First Histories

## Question

Can Merra place several peoples into a preexisting world, let them form
separate cultures, faiths, institutions, and settlements, and make their first
contact a contingent historical event rather than a scenario instruction?

The initial answer uses aggregate cohorts. It establishes historical axes and
event evidence before detailed individual lives are projected into the
selected region.

## Intended Evidence

- Three human founding cohorts on the mainland and one orc cohort in a
  separated island homeland.
- Shared physiology parameters rather than lineage-specific code paths.
- Culture, faith, lineage, and polity represented independently.
- Migration, settlement founding, institutional formation, and political
  aggregation over six centuries.
- A locked sea route opened only after navigation capability develops.
- First contact, mixed population cohorts, cultural synthesis, faith spread,
  and schism as typed causal events.
- Important-place ranking and a five-settlement starting region selected from
  the resulting history.
- Two communities preserving incompatible accounts of first contact.
- A setting-portability test using an orbital habitat place graph.

## Decisions and Rationale

- Lineage, culture, faith, and polity remain separate; see
  [ADR 0007](../../../adr/0007-lineage-culture-faith.md).
- The first non-human society is orc because it makes the separation test
  legible. Orcs use the exact same cohort, migration, institution, faith, and
  contact systems as humans.
- Orc longevity, physical power, speed, and sustenance are lineage
  multipliers. Ritual intensity and religious preservation belong to the
  Keepers of the Ring culture. The Ring Witness is a separate faith.
- Cohorts store normalized affiliation shares. Mixed settlements therefore do
  not need to erase one lineage or choose a single dominant culture.
- The macro simulation runs one deterministic Bevy schedule per historical
  year. Detailed people remain out of scope at this resolution.
- History consumes `PlaceGraphV1`. Surface terrain is optional. A route opening
  on a surface world becomes `SeaRouteOpened`; the same capability transition
  on an abstract graph remains `RouteOpened`.

## Implementation Notes

The historical scenario seeds four populations of 500. Human physiology uses
1.0 baseline multipliers. Orc physiology begins with 0.75 mortality, 1.25
power, 1.0 speed, and 1.125 sustenance multipliers. The annual schedule grows
cohorts, calculates pressure using shared physiology parameters, expands along
available routes, and establishes settlements when a migrating cohort reaches
a new place.

Culture seeds include River Folk, Upland Kin, Western Marches, and Keepers of
the Ring. The Keepers begin with 24 ritual days per year and stronger religious
and institutional transmission. Those are learned parameters: no system asks
whether a cohort is orc before applying them.

The Ring Witness faith begins at a generated mythic feature. Human population
pressure later creates the River Witness. After contact, faith transmission
and accumulated divergence create the Open Hand of the Ring Witness as a
historically related schism.

Institutions and polities grow from settlements and population thresholds.
Navigation knowledge accumulates from access and institutions. Only then can
the locked route open. First contact creates causal events, mixed-lineage
cohorts at contact settlements, and the Tidebound contact culture.

The completed report includes populations, settlements, cultures, faiths,
institutions, polities, lore claims, important places, the selected starting
region, a chronicle, and historical atlas. It copies the source world into the
exploratory run so every history remains independently inspectable.

## Experiments and Results

Canonical world and history seed: `42`, duration: 600 years.

- The four founders began as three human populations and one orc population,
  each containing 500 people.
- First human-orc contact occurred in Year 293.
- Final population was 327,549 across 24 population cohorts and 24 settlements.
- The final state contained 5 cultures, 3 faiths, 7 institutions, and 4
  mixed-lineage populations.
- The event stream contained 69 stable, typed, backward-causal events.
- The five-settlement starting region retained 12 directly relevant events.

The important contact location scored highly because it held 19,514 people,
contained a prehuman trace, became mixed-lineage, and hosted first contact.
Later sources disagreed:

- Continental pilots called it **The Discovery of the Outer Shore**.
- Island rites called it **The Return of the Divided Kin**.

Neither text replaces the authoritative contact event. They are claims with
sources and confidence.

Across seeds 1 through 20, all 20 histories reached first contact between Years
286 and 302. Final populations ranged from 282,028 to 402,635; settlements from
21 to 24; and mixed-lineage populations from 1 to 5. These tight ranges show
that the current process is stable enough to inspect, but they also show that
contact is not yet genuinely avoidable in a 600-year run.

## Failures and Surprises

The first attempt treated the island connection as a sea route everywhere.
That made the historical engine depend on a terrain metaphor and broke the
orbital habitat fixture. The engine now opens a capability-gated route; the
surface adapter supplies maritime vocabulary only when it has that evidence.

The second failure was subtler: making orcs "extremely religious" in their
lineage definition would make conversion and cultural drift meaningless.
Moving ritual and transmission parameters to culture and faith made the
Tidebound synthesis possible without changing anyone's body.

The canonical first-contact year is remarkably stable across twenty seeds.
That is useful regression evidence but not yet satisfactory emergence. The
navigation threshold and annual process currently dominate more than
geographical and institutional variation.

Final population is also too cleanly exponential for a mature model. These
numbers prove scheduling, shares, migration, and evidence—not a believable
demography.

## Tests and Invariants

- The same world, history scenario, and seed produce equal reports.
- Founder composition is exactly three human cohorts and one orc cohort.
- Every lineage, culture, and faith share vector sums to 10,000.
- Historical IDs are stable and never serialize Bevy entities.
- Orc effects are applied by general parameter functions without an `if orc`
  branch.
- Contact cannot precede route opening.
- Event IDs are contiguous, years are nondecreasing, and every cause references
  an earlier event.
- Mixed-lineage populations contain both human and orc shares.
- The selected starting region contains exactly five known settlements.
- The canonical summaries and chronicle exactly match committed golden
  evidence.
- Seeds 1 through 20 complete 600 years and preserve structural world, history,
  event, and starting-region invariants.
- An orbital habitat graph runs through the same history engine and emits a
  generic route-opening event. That fixture also seeds a third synthetic
  lineage to prove the founder schema is not permanently two-species.

## Reproduction

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

cargo tui world \
  --input runs/first-histories-42

cargo xtask world-lab \
  --first-seed 1 \
  --count 20 \
  --years 600 \
  --output runs/world-lab-20

cargo test --locked -p merra-sim --lib
cargo test --locked -p merra-testkit
npm run site:check
```

## Known Limitations and Debt

- Population cohorts have no age structure, births, deaths, households,
  occupations, health, or sex at macro resolution.
- Growth, pressure, migration, culture, faith, and institution rules are
  threshold-heavy first models rather than calibrated social science.
- Contact is effectively inevitable within 600 years across the current
  twenty-seed cohort.
- Lineage physiology is a small multiplier bundle. It does not yet model
  individual variation, mixed inheritance, reproduction, disability, or
  environment interaction.
- Institutions have identity and origin evidence but not offices, resources,
  doctrine, internal factions, or failure modes.
- Polities do not yet govern law, taxation, diplomacy, territory, or war.
- Lore claims are created at first contact rather than transmitted and mutated
  through explicit witnesses and media.
- The five-settlement region is selected by historical importance but has not
  yet been projected into detailed people and households.

## Newsletter Candidates

- Why "orcs are religious" was a schema bug, not just awkward worldbuilding.
- Running six centuries as a deterministic Bevy schedule.
- One route-opening system for ocean crossings and orbital transit.
- How two cultures can tell incompatible truths about one stable event ID.
- What a 20-world cohort reveals that Seed 42 hides.
- Choosing five playable settlements from 12,288 generated regions.
- Why 327,549 simulated people are currently only 24 cohorts.

## Next Questions

- How do the selected five settlements become detailed households without
  contradicting their aggregate history?
- Which institutions and resource pressures make routes matter locally?
- How can contact remain possible without becoming inevitable?
- What should a person in Year 600 actually know about Year 293?

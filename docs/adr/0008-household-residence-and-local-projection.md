# ADR 0008: Let Households Own Residence

- Status: Accepted
- Date: 2026-07-28

## Context

Cycle 4 ends with aggregate populations in a historically selected
five-settlement region. Cycle 5 must make those places visible at person and
household scale without pretending that 30 sampled people literally replace
40,751 aggregate inhabitants.

Residence also needs one authoritative owner. Storing location independently
on every person would permit members of one co-resident household to drift
apart accidentally. Storing only settlement totals would prevent births,
deaths, partnerships, and inherited claims from resolving to a place.

New households need an explainable destination rule. A pure random choice
would make migration difficult to interpret; capacity or food attraction would
prematurely implement Cycle 6.

## Decision

An active household owns exactly one `LocationId` residence. A living person's
current residence derives from their current household. Births, deaths,
partnerships, dissolution, and settlement decisions copy that stable location
onto their authoritative event evidence.

The macro-to-local boundary is an exact weighted projection:

- every selected aggregate population is allocated across initial sampled
  households;
- household allocations sum exactly to every settlement's macro population;
- represented population is evidence weight, not a claim that one sampled
  person is one macro person;
- descendant households inherit culture, faith, institution, and lore
  references, but do not duplicate the projection weight.

When a new household forms, it considers the five selected settlements in a
lexicographic order:

1. maximize the count of living close kin at the destination;
2. minimize summed shortest-path cost from the founders' prior residences;
3. break an exact tie with a hash derived from the isolated `migration`
   random domain, household ID, and location ID.

The event records origins, destination, travelers, route IDs, greatest travel
cost, calendar travel days, living-kin support, reason, and earlier local
causes. Shortest paths use routes available at the Cycle 4 handoff and may
cross intermediate places outside the five selected settlements.

## Consequences

- Household members cannot disagree about where they live.
- Every detailed birth and death can be grouped by place without adding an
  independent person-location state.
- Macro population numbers remain exactly auditable while detailed samples
  remain computationally small.
- Migration decisions are replayable and explainable even when a seeded tie is
  required.
- Cultural institutions headquartered outside the selected region can remain
  visible through a household's inherited culture; locally headquartered
  institutions remain distinguishable in settlement records.
- Existing Cycle 1 and Cycle 2 scenarios serialize no residence field and keep
  event schemas v1 and v2. Local history uses event schema v3.
- Travel is currently evidence attached to a household-formation decision, not
  an in-progress state. Capacity, housing, food, work, seasonal roads, and
  migration pressure remain later-cycle work.

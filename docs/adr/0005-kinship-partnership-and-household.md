# ADR 0005: Keep Kinship, Partnership, and Household Separate

- Status: Accepted
- Date: 2026-07-28

## Context

Cycle 2 needs parentage, family formation, childbirth, surnames, and household
history. Treating all of those as one "family" component would make common
changes destructive: a person may leave a childhood home, form a later
partnership, have children in more than one household, or survive a partner.
None of those changes should rewrite who their parents or children were.

The simulation also does not yet model the cultural, legal, religious, or
property rules required to make "marriage" a universal authoritative concept.

## Decision

Merra models three related but distinct structures:

1. **Kinship** is immutable parentage on a stable person record. A child born
   during a run receives two stable parent IDs and a generation number.
2. **Partnership** is a current relationship between two living people.
   Formation and death-driven ending are historical events.
3. **Household** is a stable entity with current members, a founding day, an
   optional dissolution day, and a surname inherited by children born there.

The first rules call relationships partnerships rather than marriages.
Marriage can later become a culture- and institution-specific event layered on
the same stable people and households.

Scenario-v1 family configuration is additive and opt-in. A scenario without a
`family` field deserializes with family behavior disabled, preserving the
tagged Cycle 1 histories. Family-enabled scenarios provide explicit age,
birth-interval, child-count, and generation limits. Their manifests declare
event schema v2 because the new exhaustive event variants are not readable as
the tagged event schema v1.

## Consequences

- Death can end a partnership without deleting parentage.
- Moving to a new household does not change a person's own surname or parents.
- Children inherit the household surname while partners retain their existing
  names, allowing surnames to spread, persist, or vanish independently.
- The genealogy inspector can reconstruct relations from stable records and
  events rather than ECS entity identifiers.
- Current pairing excludes parent-child and sibling relationships and stays
  within a generation. Cousin distance, adoption, guardianship, divorce,
  plural households, and culturally specific marriage remain future work.

# ADR 0009: Separate Item Identity, Provenance, Ownership, and Custody

- Status: Accepted
- Date: 2026-07-29

## Context

Merra can reconstruct a family because stable person identity and immutable
parentage survive changes in partnership, household, residence, and life
status. Durable objects need the same separation. Treating an object's name,
owner, holder, location, condition, and source materials as one mutable record
would erase the history the system is intended to preserve.

Tracking every unit of food, coin, ore, or timber individually would also
create false precision at the current weighted-household resolution.

## Decision

Individually traced items are movable durable objects such as tools, weapons,
books, jewelry, and future relic candidates. Bulk goods, money, land, and
buildings use separate aggregate or property contracts.

Each item receives a stable `ItemId`. Its source links are immutable and form a
directed acyclic provenance graph. A repair preserves identity. Reforging,
splitting, merging, or consuming physical inputs creates new item identities
and retires the consumed sources. Source links state whether an earlier item
contributed material, a component, or a pattern; they do not claim exact
material shares.

Legal ownership and physical custody are independent typed values. Custody,
not ownership, determines authoritative location. A lost item may retain its
owner while its custody and location are unknown.

Initial local items are recorded as entering detailed history at the projection
boundary. Merra does not invent an earlier crafting event merely because the
object must already have existed.

## Consequences

- Ownership transfer does not imply movement, and movement does not rewrite
  legal title.
- Item biography is reconstructed from authoritative events just as household
  and family history are.
- Work, wear, repair, inheritance, and transformation cite earlier evidence.
- A later culture may promote any ordinary item into an artifact or relic
  without creating a second identity.
- Aggregate production lots, component quantities, theft, authenticity claims,
  and forged provenance remain later systems.

# ADR 0007: Separate Lineage, Culture, Faith, and Polity

- Status: Accepted
- Date: 2026-07-28

## Context

The first non-human population is an isolated orc society. The initial fiction
calls for longer lives, greater strength, and unusually intense religious
practice. A single "race" configuration could encode all of those traits, but
it would turn learned behavior into biology, prevent conversion and cultural
change, and make mixed populations structurally ambiguous.

Merra must eventually support multiple lineages with separate histories,
migration, contact, mixed households, conversion, cultural synthesis, and
political membership. It must do so with shared systems rather than a branch
such as `if orc`.

## Decision

The historical model keeps four independent affiliation axes:

1. **Lineage** describes inherited physiology. The initial portable parameters
   are mortality, power, speed, and sustenance multipliers.
2. **Culture** describes learned and transmitted practice. Initial parameters
   include ritual time, sacred contribution, institutional preservation, and
   faith transmission.
3. **Faith** describes a historical tradition with an origin, optional parent,
   and possible connection to a prehuman feature.
4. **Polity** describes political membership and authority.

Population cohorts store normalized shares for every relevant axis rather than
one exclusive species or culture field. `HistoryConfigV1` accepts a list of
founders. Each founder references a lineage ID, a generic homeland tag, and a
learned culture; scheduled faiths reference cultures by stable keys. Founding
populations can begin with a single affiliation, while migration and contact
can create mixed-lineage, mixed-culture, or mixed-faith cohorts.

The canonical orc lineage begins with:

- mortality multiplier `0.75`;
- power multiplier `1.25`;
- speed multiplier `1.0`;
- sustenance multiplier `1.125`.

Its Keepers of the Ring culture—not its lineage—begins with 24 ritual days per
year, increased sacred contribution, stronger institutional preservation, and
stronger faith transmission. The Ring Witness faith derives its origin claim
from a generated mythic trace.

All physiology is evaluated through shared parameter calculations. Cultural
and religious processes operate independently. Contact can therefore form the
Tidebound culture and later a Ring Witness schism without changing anyone's
lineage.

## Consequences

- An orc child raised in another society is not forced to inherit the Keepers'
  beliefs or institutions.
- A human population can adopt the Ring Witness, and an orc population can
  change culture, without special conversion code.
- Mixed populations remain representable before the detailed person model
  gains multi-lineage households.
- Adding another lineage requires data and suitable homeland affordances, not a
  new schema field. The orbital portability fixture runs humans, orcs, and a
  third synthetic lineage through the same engine.
- Lineage multipliers are fictional model inputs, not claims about real human
  groups or deterministic claims about individual behavior.
- Physiology needs more careful semantics before detailed combat, nutrition,
  reproduction, or disability systems use it. The aggregate multipliers are a
  first structural proof, not a complete biological model.

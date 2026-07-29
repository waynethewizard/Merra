# Scenarios

Scenarios are versioned, human-authored RON inputs. A scenario defines starting
configuration, not a predetermined outcome.

Every committed scenario must:

- declare its schema version and stable identifier;
- define ordered, uniquely identified seasons whose positive day counts exactly
  fill `days_per_year`;
- keep family simulation opt-in and define ordered age, birth-interval,
  household-child, and generation limits when enabled;
- pass validation before entering the simulation;
- avoid credentials, local filesystem paths, and private data;
- name any third-party source material in the appropriate attribution file.

World and macro-history scenarios use separate versioned contracts:

- `before-memory.ron` configures physical generation and mythic motifs. It
  produces generic homeland tags and affordances, never lineage-specific
  terrain.
- `first-histories.ron` configures founding lineages, cultures, faith, aggregate
  historical duration, and capability thresholds. Inherited physiology belongs
  to lineage; ritual, preservation, and transmission belong to culture or
  faith.

Committed world/history scenarios must also keep all affiliation shares
normalizable to 10,000, provide three primary and one isolated founder
locations through their input graph, and remain reproducible without private
paths, external services, or secrets.

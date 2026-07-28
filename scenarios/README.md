# Scenarios

Scenarios are versioned, human-authored RON inputs. A scenario defines starting
configuration, not a predetermined outcome.

Every committed scenario must:

- declare its schema version and stable identifier;
- define ordered, uniquely identified seasons whose positive day counts exactly
  fill `days_per_year`;
- pass validation before entering the simulation;
- avoid credentials, local filesystem paths, and private data;
- name any third-party source material in the appropriate attribution file.

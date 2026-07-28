# Scenarios

Scenarios are versioned, human-authored RON inputs. A scenario defines starting
configuration, not a predetermined outcome.

Every committed scenario must:

- declare its schema version and stable identifier;
- pass validation before entering the simulation;
- avoid credentials, local filesystem paths, and private data;
- name any third-party source material in the appropriate attribution file.

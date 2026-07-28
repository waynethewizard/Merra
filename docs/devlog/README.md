# Development Chronicle

This is Merra's detailed build record and the evidence base for Rusting
newsletter posts.

Each development cycle has one evolving record. Contributors document work as
it happens rather than reconstructing design history at the end. The record
should explain the causal question, not merely list changed files.

## Required practice

- Update the active cycle record for material implementation, tuning, and
  architecture work.
- Include failed approaches and absurd simulation outcomes when they teach
  something.
- Preserve exact scenario paths, seeds, commands, measurements, and source
  tags needed to reproduce claims.
- Link durable decisions to ADRs.
- Link only selected, licensed, metadata-scrubbed media.
- Remove credentials, private paths, user data, and unrelated terminal output.

Use `cargo xtask new-cycle` to create later cycle records. `cargo xtask
verify-docs` checks required sections.

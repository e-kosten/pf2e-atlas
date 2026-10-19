# Historical snapshot: restoration.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/problem-response/restoration.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: problem.repair
    label: Repair
    description: Fixes, restores, mends, rebuilds, or returns damaged objects, equipment, structures, or mechanisms to usable function.
    display:
      group: ProblemResponse
      subgroup: Restoration
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record repairs, mends, rebuilds, restores function, or materially supports fixing a damaged object or structure.
        - Returning something nonliving to usable function is the retrieval hook.
      does_not_apply_when:
        - The record only heals creatures or prevents future damage without fixing a damaged object.

  - id: problem.emergency_restoration
    label: Emergency Restoration
    description: Quickly restores function, consciousness, control, or survivability after incapacitation, loss of control, or sudden failure.
    display:
      group: ProblemResponse
      subgroup: Restoration
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps recover from sudden incapacitation, loss of control, unconsciousness, collapse, or loss of function.
        - Emergency restoration is more important than routine healing, long-term care, death prevention, or general durability.
      does_not_apply_when:
        - The record only gives routine healing, death prevention, or a passive defensive bonus without an emergency restoration role.

````

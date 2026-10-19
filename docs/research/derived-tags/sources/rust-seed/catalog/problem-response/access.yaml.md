# Historical snapshot: access.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/problem-response/access.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: problem.trap_bypass
    label: Trap Bypass
    description: Disarms, suppresses, safely triggers, or helps bypass traps and warded access points.
    display:
      group: ProblemResponse
      subgroup: Access
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps disable, neutralize, safely trigger, or get past a trap or warded access point.
        - Trap-solving matters more than generic scouting, damage prevention, or perception.
      does_not_apply_when:
        - The record only reveals that a trap exists without helping bypass or disable it.

  - id: problem.lock_bypass
    label: Lock Bypass
    description: Opens, unseals, circumvents, or helps defeat locks, sealed containers, secured doors, or restraints.
    display:
      group: ProblemResponse
      subgroup: Access
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record is naturally retrieved to unlock, unseal, open, or bypass a secured entry point, container, manacle, or closure.
        - Accessing something closed matters more than broad movement, damage, or generic object manipulation.
      does_not_apply_when:
        - The record mainly destroys the obstacle or teleports past the whole scene without interacting with the locked access point.

  - id: problem.barrier_bypass
    label: Barrier Bypass
    description: Circumvents walls, force barriers, sealed passages, blocked routes, or other non-lock obstacles.
    display:
      group: ProblemResponse
      subgroup: Access
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps get past a barrier, wall, seal, blocked path, or other access obstacle without simply solving a lock.
        - The retrieval problem is passing the obstacle, not creating battlefield control.
      does_not_apply_when:
        - The record creates a barrier or only improves ordinary movement with no meaningful blocked-access use case.

  - id: problem.barrier_removal
    label: Barrier Removal
    description: Breaks, destroys, demolishes, opens, or otherwise removes a physical or magical access barrier.
    display:
      group: ProblemResponse
      subgroup: Access
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record is naturally retrieved to destroy, breach, demolish, break, or remove an obstacle that blocks access.
        - Removing the obstacle matters more than bypassing it invisibly, teleporting past a whole scene, or using it as a weapon.
      does_not_apply_when:
        - The record only deals object damage incidentally and is not a plausible access-solving tool.

  - id: problem.mechanism_manipulation
    label: Mechanism Manipulation
    description: Operates, alters, jams, disarms, controls, or safely manipulates mechanisms and control interfaces.
    display:
      group: ProblemResponse
      subgroup: Access
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, hazard]
    guidance:
      applies_when:
        - The record helps manipulate a mechanism, control panel, lever, device, lockwork, or trap interface as part of solving access.
        - Understanding or operating the mechanism is more important than brute-force damage.
      does_not_apply_when:
        - The record merely contains a mechanism or has mechanical traits without helping operate or defeat it.

  - id: problem.restraint_escape
    label: Restraint Escape
    description: Escapes, loosens, opens, slips, or breaks bonds, manacles, grapples, immobilization, or confinement.
    display:
      group: ProblemResponse
      subgroup: Access
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps a creature escape physical restraint, confinement, manacles, bonds, immobilization, or similar capture.
        - Escaping restraint is the practical problem being solved.
      does_not_apply_when:
        - The record captures or restrains others without helping the user escape.

````

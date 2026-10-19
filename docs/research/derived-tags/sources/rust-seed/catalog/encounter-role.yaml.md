# Historical snapshot: encounter-role.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/encounter-role.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: encounter.combat.ambusher
    label: Ambusher
    description: Built around hidden approach, surprise openings, sudden strike pressure, or concealed attack vectors.
    display:
      group: EncounterRole
      subgroup: Combat
    applicability:
      any_of:
        - record_kinds: [creature]
    guidance:
      applies_when:
        - The creature is naturally retrieved for stealth openings, burst-from-concealment attacks, or surprise-predator play.
        - Opening from hidden position is central to how it functions in an encounter.
      does_not_apply_when:
        - The creature only uses Stealth incidentally before acting as another clearer combat role.

  - id: encounter.combat.artillery
    label: Artillery
    description: Pressures targets from range through volleys, spell barrages, breath attacks, or other standoff offense.
    display:
      group: EncounterRole
      subgroup: Combat
    applicability:
      any_of:
        - record_kinds: [creature]
    guidance:
      applies_when:
        - The creature prefers distance and is naturally retrieved as a ranged damage or spell-barrage threat.
        - Standoff pressure matters more than command, mobility, or pure control.
      does_not_apply_when:
        - The creature has one ranged option but fundamentally plays as a brute, controller, or support piece.

  - id: encounter.combat.brute
    label: Brute
    description: Pressures the front line through durability, direct damage, and straightforward melee threat.
    display:
      group: EncounterRole
      subgroup: Combat
    applicability:
      any_of:
        - record_kinds: [creature]
    guidance:
      applies_when:
        - The creature is a heavy hitter that advances, endures punishment, and threatens through direct force.
        - Sturdy direct pressure matters more than mobility, command, or precision control.
      does_not_apply_when:
        - The creature mainly operates from range, through battlefield control, or through ally support.

  - id: encounter.combat.controller
    label: Controller
    description: Reshapes the battlefield through debuffs, forced movement, terrain control, or tactical denial.
    display:
      group: EncounterRole
      subgroup: Combat
    applicability:
      any_of:
        - record_kinds: [creature]
    guidance:
      applies_when:
        - The creature is naturally retrieved for immobilizing, repositioning, walling off, slowing, or dictating battlefield shape.
        - Tactical denial matters more than direct damage output.
      does_not_apply_when:
        - The creature only has one incidental debuff while otherwise fighting as a brute, defender, or artillery piece.

  - id: encounter.combat.defender
    label: Defender
    description: Holds space, guards chokepoints, bodyguards allies, or punishes movement through a defended line.
    display:
      group: EncounterRole
      subgroup: Combat
    applicability:
      any_of:
        - record_kinds: [creature]
    guidance:
      applies_when:
        - The creature is naturally retrieved for guarding chokepoints, bodyguarding allies, or making passage costly.
        - Space-holding matters more than raw pursuit, burst damage, or command.
      does_not_apply_when:
        - The creature is merely durable without a real guard, intercept, or line-holding identity.

  - id: encounter.combat.support
    label: Support
    description: Heals, buffs, protects, commands, or otherwise enables allied creatures more than acting as the primary damage source.
    display:
      group: EncounterRole
      subgroup: Combat
    applicability:
      any_of:
        - record_kinds: [creature]
    guidance:
      applies_when:
        - The creature is naturally retrieved because it keeps allies alive, stronger, better positioned, or otherwise more dangerous.
        - Ally enablement matters more than its own direct kill pressure.
      does_not_apply_when:
        - The creature has one incidental buff or heal but is otherwise a brute, artillery, or controller.

  - id: encounter.scene.pack_hunter
    label: Pack Hunter
    description: Functions as part of a coordinated pack, pursuit group, ambush cluster, or hunting group.
    display:
      group: EncounterRole
      subgroup: Scene
    applicability:
      any_of:
        - record_kinds: [creature]
    guidance:
      applies_when:
        - The creature hunts, patrols, ambushes, or pressures prey as part of a coordinated group.
        - Its planning value improves when used with similar creatures or a pack structure.
      does_not_apply_when:
        - The creature is primarily a solitary predator or only appears in groups by encounter-building convenience.
````

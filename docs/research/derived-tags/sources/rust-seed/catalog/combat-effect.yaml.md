# Historical snapshot: combat-effect.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/combat-effect.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: combat.control.barrier_creation
    label: Barrier Creation
    description: Creates a wall, dome, cage, force barrier, or other blocking structure that reshapes access lines.
    display:
      group: CombatEffect
      subgroup: BattlefieldControl
    applicability:
      any_of:
        - record_kinds: [spell, equipment, creature, hazard]
    guidance:
      applies_when:
        - The record creates a discrete obstacle or barrier that changes movement, targeting, or access.
        - The barrier's battlefield shape matters more than raw damage or protection alone.
      does_not_apply_when:
        - The record only grants a defensive bonus without changing space or access.

  - id: combat.control.forced_movement
    label: Forced Movement
    description: Pushes, pulls, drags, teleports, slides, or otherwise repositions a target against its will.
    display:
      group: CombatEffect
      subgroup: ForcedPosition
    applicability:
      any_of:
        - record_kinds: [spell, equipment, creature, hazard]
    guidance:
      applies_when:
        - The record's tactical use includes moving unwilling targets into, out of, or through meaningful positions.
        - Repositioning matters more than incidental knockback flavor.
      does_not_apply_when:
        - The record only changes the user's own position or movement mode.

  - id: combat.control.mobility_denial
    label: Mobility Denial
    description: Slows, grounds, pins, immobilizes, restrains, entangles, or otherwise prevents normal repositioning.
    display:
      group: CombatEffect
      subgroup: BattlefieldControl
    applicability:
      any_of:
        - record_kinds: [spell, equipment, creature, hazard]
    guidance:
      applies_when:
        - The record prevents or sharply limits a target's movement, flight, escape, or repositioning.
        - Movement denial is a repeatable tactical retrieval hook.
      does_not_apply_when:
        - The record only deals damage to a moving target without limiting movement.

  - id: combat.control.line_of_sight
    label: Line of Sight Control
    description: Blocks vision, creates concealment, obscures sight lines, or denies clear observation across an area.
    display:
      group: CombatEffect
      subgroup: SensoryControl
    applicability:
      any_of:
        - record_kinds: [spell, equipment, creature, hazard]
    guidance:
      applies_when:
        - The record materially changes visibility, concealment, observation, or sight lines.
        - Perception control is central to its tactical use.
      does_not_apply_when:
        - The record only improves a single Perception check without creating or denying visual access.

  - id: combat.debilitation.fear_pressure
    label: Fear Pressure
    description: Forces fear, panic, dread, morale collapse, or fear-driven action loss onto targets.
    display:
      group: CombatEffect
      subgroup: Debilitation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, creature, hazard]
    guidance:
      applies_when:
        - The record's primary combat pressure is frightening, panicking, intimidating, or dread-based disruption.
        - Fear consequences matter more than damage or ordinary mental influence.
      does_not_apply_when:
        - The record is merely spooky or horror-flavored without a mechanical fear or morale effect.

  - id: combat.control.silencing
    label: Silencing
    description: Suppresses speech, sound production, verbal casting, audible signals, or voice-dependent actions.
    display:
      group: CombatEffect
      subgroup: SensoryControl
    applicability:
      any_of:
        - record_kinds: [spell, equipment, creature, hazard]
    guidance:
      applies_when:
        - The record prevents or suppresses sound, speech, verbal components, alarms, or audible coordination.
        - Sound denial is a meaningful combat or scene-control hook.
      does_not_apply_when:
        - The record only deals sonic damage or has sound flavor without suppressing sound or speech.

  - id: combat.damage.burst
    label: Burst Damage
    description: Delivers a concentrated spike of damage in a short window rather than steady pressure over time.
    display:
      group: CombatEffect
      subgroup: Damage
    applicability:
      any_of:
        - record_kinds: [spell, equipment, creature, hazard]
    guidance:
      applies_when:
        - The record is naturally retrieved for immediate, high-impact damage in one action, turn, opening, or short burst.
        - Short-window damage output matters more than durability, control, or attrition.
      does_not_apply_when:
        - The record mainly creates sustained damage, persistent damage, or a long-term hazard with no burst payoff.

  - id: combat.damage.persistent
    label: Persistent Damage
    description: Applies recurring damage, bleed, burning, poison ticks, or other damage that continues after the initial effect.
    display:
      group: CombatEffect
      subgroup: Damage
    applicability:
      any_of:
        - record_kinds: [spell, equipment, creature, hazard]
    guidance:
      applies_when:
        - The record applies persistent damage, recurring damage, bleed, burning, or similar ongoing harm.
        - Continued damage pressure is a meaningful tactical retrieval hook.
      does_not_apply_when:
        - The record only deals one-time damage or creates a non-damaging condition.

  - id: combat.debilitation.confusion
    label: Confusion
    description: Disrupts agency, targeting, action choice, or coherent behavior through confusion or similar mental disorientation.
    display:
      group: CombatEffect
      subgroup: Debilitation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, creature, hazard]
    guidance:
      applies_when:
        - The record causes confusion, incoherent action, random targeting, or comparable loss of reliable control.
        - Mental disorientation is the tactical effect being searched for.
      does_not_apply_when:
        - The record only frightens, charms, or deceives without disrupting action choice or coherent behavior.

  - id: combat.protection.resistance
    label: Resistance
    description: Grants resistance, mitigation, immunity-like protection, or damage-type shielding as the main encounter effect.
    display:
      group: CombatEffect
      subgroup: Protection
    applicability:
      any_of:
        - record_kinds: [spell, equipment, creature, hazard]
    guidance:
      applies_when:
        - The record reduces incoming damage, grants resistance, protects against an energy type, or mitigates a specific harm.
        - The focus is the protective effect itself, not a broader character build role.
      does_not_apply_when:
        - The record only improves exploration survival or provides a generic AC or save bonus.
````

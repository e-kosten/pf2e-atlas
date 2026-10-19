# Historical snapshot: status-effect.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/status-effect.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: condition.mechanism.disease
    label: Disease
    description: Centers on disease as a harmful affliction, contagion, staged illness, or pathological threat.
    display:
      group: StatusEffect
      subgroup: AfflictionMechanism
    applicability:
      any_of:
        - record_kinds: [affliction, creature, hazard, spell, equipment]
    guidance:
      applies_when:
        - The record inflicts, carries, spreads, embodies, or is primarily structured around disease.
        - Disease is a central threat mechanism, not merely a descriptive consequence.
      does_not_apply_when:
        - The record only cures, counteracts, diagnoses, or mitigates disease without presenting disease as its own mechanism.

  - id: condition.mechanism.poison
    label: Poison
    description: Centers on poison, venom, toxin delivery, exposure, or poison-driven affliction pressure.
    display:
      group: StatusEffect
      subgroup: AfflictionMechanism
    applicability:
      any_of:
        - record_kinds: [affliction, creature, hazard, spell, equipment]
    guidance:
      applies_when:
        - The record inflicts, carries, delivers, spreads, or is primarily structured around poison or venom.
        - Poison is a central threat mechanism, not only a damage trait.
      does_not_apply_when:
        - The record only cures, counteracts, neutralizes, or mitigates poison without presenting poison as its own mechanism.

  - id: condition.mechanism.curse
    label: Curse
    description: Centers on curses, supernatural affliction, lingering doom, taboo punishment, or malediction.
    display:
      group: StatusEffect
      subgroup: AfflictionMechanism
    applicability:
      any_of:
        - record_kinds: [affliction, creature, hazard, spell, equipment, lore]
    guidance:
      applies_when:
        - The record inflicts, carries, enforces, embodies, or is primarily structured around a curse.
        - Curse logic is central to the threat, story, item, or effect.
      does_not_apply_when:
        - The record only removes, counteracts, diagnoses, or protects against curses without presenting a curse mechanism.

  - id: condition.mechanism.fear
    label: Fear
    description: Centers on fear, panic, dread, morale collapse, or frightened-state pressure.
    display:
      group: StatusEffect
      subgroup: AfflictionMechanism
    applicability:
      any_of:
        - record_kinds: [affliction, creature, hazard, spell, equipment]
    guidance:
      applies_when:
        - The record inflicts, amplifies, embodies, or is primarily structured around fear, panic, dread, or frightened-state pressure.
        - Fear is a central mechanism rather than only genre flavor.
      does_not_apply_when:
        - The record only bolsters courage, removes fear, or mitigates frightened without presenting fear as its own mechanism.

  - id: condition.mechanism.paralysis
    label: Paralysis
    description: Centers on paralysis, total bodily shutdown, action denial through immobility, or loss of voluntary movement.
    display:
      group: StatusEffect
      subgroup: AfflictionMechanism
    applicability:
      any_of:
        - record_kinds: [affliction, creature, hazard, spell, equipment]
    guidance:
      applies_when:
        - The record inflicts, causes, embodies, or is primarily structured around paralysis or comparable total bodily shutdown.
        - Paralysis is a central threat mechanism.
      does_not_apply_when:
        - The record only restores movement, protects against paralysis, or improves mobility without presenting paralysis as its own mechanism.

  - id: condition.mechanism.petrification
    label: Petrification
    description: Centers on petrification, turn-to-stone effects, mineral transformation, or immobilizing body-state conversion.
    display:
      group: StatusEffect
      subgroup: AfflictionMechanism
    applicability:
      any_of:
        - record_kinds: [affliction, creature, hazard, spell, equipment]
    guidance:
      applies_when:
        - The record inflicts, causes, embodies, or is primarily structured around petrification or comparable body-state conversion.
        - Turning a creature to stone or an equivalent immobilizing transformation is central.
      does_not_apply_when:
        - The record only prevents, reverses, counteracts, or mitigates petrification without presenting petrification as its own mechanism.
````

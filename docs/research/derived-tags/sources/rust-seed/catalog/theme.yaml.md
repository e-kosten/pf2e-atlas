# Historical snapshot: theme.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/theme.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: theme.genre.gothic_horror
    label: Gothic Horror
    description: Uses haunted nobility, graveyard dread, cursed lineages, misty superstition, or classic night-monster atmosphere.
    display:
      group: Theme
      subgroup: Genre
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore, spell, equipment]
    guidance:
      applies_when:
        - The record is naturally retrieved for Gothic-horror encounter, setting, or story prep.
        - Atmosphere, cursed lineage, old-world dread, or classic monster-fiction mood is a strong association.
      does_not_apply_when:
        - The record is simply undead, spooky, or cursed without a broader Gothic-horror frame.

  - id: theme.genre.folk_horror
    label: Folk Horror
    description: Uses rural isolation, old rites, local superstition, harvest dread, community secrets, or wilderness cultic unease.
    display:
      group: Theme
      subgroup: Genre
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore, spell, equipment]
    guidance:
      applies_when:
        - The record is naturally retrieved for isolated-village, old-ritual, superstition, or rural horror prep.
        - The theme is rooted in community practice, landscape memory, or local taboo.
      does_not_apply_when:
        - The record is merely primal, spooky, or cultic without a folk-horror scene identity.

  - id: theme.object.clockwork
    label: Clockwork
    description: Features clockwork bodies, gear-driven mechanisms, wound engines, automata, or precision machinery motifs.
    display:
      group: Theme
      subgroup: ObjectSymbol
    applicability:
      any_of:
        - record_kinds: [creature, hazard, equipment, lore]
    guidance:
      applies_when:
        - The record is naturally retrieved for clockwork machinery, automata, gearwork, or precision mechanical motifs.
        - Mechanical construction is a central motif, not only an incidental tool.
      does_not_apply_when:
        - The record is simply a construct or device without a distinct clockwork motif.

  - id: theme.creature.parasite
    label: Parasite
    description: Centers on infestation, hosts, implanted offspring, body invasion, or parasitic life cycles.
    display:
      group: Theme
      subgroup: CreatureArchetype
    applicability:
      any_of:
        - record_kinds: [creature, affliction, hazard, lore]
    guidance:
      applies_when:
        - The record is naturally retrieved for parasites, infestation, host exploitation, or implanted threat stories.
        - The parasitic relationship matters more than generic disease or poison.
      does_not_apply_when:
        - The record merely causes damage over time without host-parasite dynamics.
````

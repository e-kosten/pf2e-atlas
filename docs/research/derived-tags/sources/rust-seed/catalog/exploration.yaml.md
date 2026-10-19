# Historical snapshot: exploration.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/exploration.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: exploration.flight
    label: Flight
    description: Grants flying movement, sustained aerial travel, or practical airborne maneuvering.
    display:
      group: Exploration
      subgroup: Traversal
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record enables flight, airborne movement, or reliable vertical traversal.
        - Aerial movement is a meaningful retrieval hook outside a single combat damage rider.
      does_not_apply_when:
        - The record only references flying targets or airborne enemies without granting useful movement.

  - id: exploration.aquatic_travel
    label: Aquatic Travel
    description: Helps with swimming, underwater breathing, water-surface travel, or aquatic operations.
    display:
      group: Exploration
      subgroup: Traversal
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps a party operate in or across water through breathing, swimming, buoyancy, or aquatic movement.
        - Aquatic travel or survival is the main retrieval value.
      does_not_apply_when:
        - The record only deals water damage or has water flavor without solving aquatic exploration problems.

  - id: exploration.environmental_adaptation
    label: Environmental Adaptation
    description: Helps creatures endure hostile climates, pressure, smoke, altitude, vacuum, or other expedition-grade extremes.
    display:
      group: Exploration
      subgroup: Survival
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps survive extreme heat, cold, altitude, smoke, pressure, or other punishing environmental conditions.
        - Environmental endurance matters more than only resisting one attack form.
      does_not_apply_when:
        - The record mainly grants combat resistance, a brief ward, or aquatic mobility without broader expedition-survival value.

  - id: exploration.field_shelter
    label: Field Shelter
    description: Creates shelter, refuge, protected rest, or field habitation support in hostile territory.
    display:
      group: Exploration
      subgroup: Shelter
    applicability:
      any_of:
        - record_kinds: [spell, equipment]
    guidance:
      applies_when:
        - The record creates or materially supports a safe campsite, refuge, or protected resting place.
        - Prolonged field habitation or protected rest matters more than momentary combat defense.
      does_not_apply_when:
        - The record only creates a brief combat ward, cover effect, or instant defensive barrier.

  - id: exploration.scouting
    label: Scouting
    description: Helps observe at a distance, extend senses, inspect ahead, or gather remote scene information.
    display:
      group: Exploration
      subgroup: Reconnaissance
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps scout, see remotely, inspect ahead, extend senses, or gather information before direct engagement.
        - Information gathering is a primary use case rather than an incidental sensory rider.
      does_not_apply_when:
        - The record only grants a combat perception bonus without a real reconnaissance use case.

  - id: exploration.tracking
    label: Tracking
    description: Locates a specific creature, object, destination, or supernatural trail and helps follow it.
    display:
      group: Exploration
      subgroup: Reconnaissance
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record is naturally retrieved to locate or follow a target, object, place, route, or trace.
        - Finding a specific thing matters more than broad perception or lore.
      does_not_apply_when:
        - The record only reveals general information without helping locate or follow a target.

  - id: exploration.stealth
    label: Stealth
    description: Helps move quietly, avoid notice, suppress noisy presence, or support covert approach.
    display:
      group: Exploration
      subgroup: Reconnaissance
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps a creature move quietly, avoid notice, pass unseen, or keep a covert approach low-profile.
        - Quiet entry or low-profile movement is the retrieval hook.
      does_not_apply_when:
        - The record only creates combat concealment without supporting a covert approach.

  - id: exploration.anti_tracking
    label: Anti-Tracking
    description: Hides trails, obscures passage, defeats pursuit, or prevents creatures and objects from being followed.
    display:
      group: Exploration
      subgroup: Reconnaissance
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps conceal tracks, misdirect pursuit, block tracking, or prevent a specific creature or object from being followed.
        - Avoiding pursuit is a primary exploration use case.
      does_not_apply_when:
        - The record only improves ordinary Stealth or concealment without meaningfully defeating tracking.

  - id: exploration.navigation
    label: Navigation
    description: Helps determine direction, follow routes, avoid becoming lost, or travel reliably through difficult terrain or unfamiliar regions.
    display:
      group: Exploration
      subgroup: Traversal
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps navigate, orient, find a route, avoid getting lost, or maintain course during travel.
        - Route-finding or travel reliability is the retrieval hook.
      does_not_apply_when:
        - The record only teleports to a known place or grants movement speed without helping choose or maintain a route.

  - id: exploration.teleportation
    label: Teleportation
    description: Moves creatures or objects across distance, around terrain, or between locations without ordinary travel.
    display:
      group: Exploration
      subgroup: Traversal
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record provides short-range, long-range, extraction, or route-bypassing teleportation for practical movement.
        - Teleportation as travel or repositioning is more important than forced enemy movement.
      does_not_apply_when:
        - The record only teleports an unwilling target as an attack or battlefield control effect.

  - id: exploration.sustenance
    label: Sustenance
    description: Provides food, water, breathing support, rest support, or other basics needed to keep an expedition alive.
    display:
      group: Exploration
      subgroup: Survival
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps provide food, water, air, rest, endurance, or other baseline survival needs during travel.
        - Expedition sustainment matters more than a brief combat buff.
      does_not_apply_when:
        - The record only restores Hit Points or grants temporary combat durability.
````

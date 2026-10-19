# Historical snapshot: scene-challenge.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/scene-challenge.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: scene.resolution.layered_resolution
    label: Layered Resolution
    description: Asks the party to solve multiple linked pieces rather than one single disable check or obvious answer.
    display:
      group: SceneChallenge
      subgroup: ResolutionPattern
    applicability:
      any_of:
        - record_kinds: [hazard]
    guidance:
      applies_when:
        - The hazard has multiple linked steps, components, anchors, or phases that must be solved together.
        - The puzzle or disable structure is meaningfully more layered than a single check.
      does_not_apply_when:
        - The hazard only has multiple routine checks but no distinct linked-resolution structure.

  - id: scene.resolution.observation_first
    label: Observation First
    description: Rewards careful watching, clue gathering, pattern reading, or environmental interpretation before safe action.
    display:
      group: SceneChallenge
      subgroup: ResolutionPattern
    applicability:
      any_of:
        - record_kinds: [hazard]
    guidance:
      applies_when:
        - Understanding the hazard safely requires observing a pattern, clue, tell, cycle, or environmental signal.
        - Reading the scene matters more than brute-force disabling or enduring exposure.
      does_not_apply_when:
        - The hazard's safe answer is obvious up front or purely mechanical.

  - id: scene.resolution.timing_window
    label: Timing Window
    description: Is best handled by acting during the right cycle, opening, lull, or repeating timing pattern.
    display:
      group: SceneChallenge
      subgroup: ResolutionPattern
    applicability:
      any_of:
        - record_kinds: [hazard]
    guidance:
      applies_when:
        - The hazard creates meaningful windows, cycles, or timing openings that affect the safest solution.
        - Acting at the right moment is a core part of the challenge.
      does_not_apply_when:
        - The hazard simply repeats damage every round without an actionable timing decision.

  - id: scene.spatial.threshold_lockdown
    label: Threshold Lockdown
    description: Controls gates, bridges, chokepoints, stairwells, doors, or forced-passage bottlenecks.
    display:
      group: SceneChallenge
      subgroup: SpatialConstraint
    applicability:
      any_of:
        - record_kinds: [hazard]
    guidance:
      applies_when:
        - The hazard is naturally retrieved for narrow crossings, gates, stairs, doors, or spaces intruders must pass through.
        - Forced-passage geometry is central to the design.
      does_not_apply_when:
        - The hazard only happens to sit near a doorway or bridge without the chokepoint being central.

  - id: scene.resolution.hidden_source
    label: Hidden Source
    description: Makes finding the hidden source, anchor, origin, or engine of the problem part of resolving the scene.
    display:
      group: SceneChallenge
      subgroup: ResolutionPattern
    applicability:
      any_of:
        - record_kinds: [hazard]
    guidance:
      applies_when:
        - The hazard requires or rewards locating a hidden source, anchor, origin, or controlling element before it can be resolved safely.
        - The challenge structure depends on discovering where the problem comes from.
      does_not_apply_when:
        - The hazard has an obvious source and the main task is only enduring, disabling, or bypassing it.

  - id: scene.spatial.blocked_route
    label: Blocked Route
    description: Turns route obstruction, forced detour, sealed passage, or obstructed travel into the central scene constraint.
    display:
      group: SceneChallenge
      subgroup: SpatialConstraint
    applicability:
      any_of:
        - record_kinds: [hazard]
    guidance:
      applies_when:
        - The hazard's main scene function is blocking, rerouting, sealing, or obstructing passage through an area.
        - Navigation through constrained space matters more than the hazard's damage type.
      does_not_apply_when:
        - The hazard merely occupies space without making route obstruction central.

  - id: scene.pressure.reinforcement
    label: Reinforcement Pressure
    description: Escalates a scene by calling, creating, summoning, attracting, or enabling additional threats over time.
    display:
      group: SceneChallenge
      subgroup: EscalationPressure
    applicability:
      any_of:
        - record_kinds: [hazard, creature]
    guidance:
      applies_when:
        - The record escalates the scene by introducing reinforcements, spawn, summoned creatures, attracted guards, or similar additional threats.
        - The pressure comes from the scene becoming more crowded or dangerous over time.
      does_not_apply_when:
        - The record only has allies nearby in encounter text without a meaningful reinforcement or escalation pattern.

  - id: scene.pressure.spreading_threat
    label: Spreading Threat
    description: Expands danger through contagion, contamination, fire, overgrowth, collapse, flooding, or other growth across the scene.
    display:
      group: SceneChallenge
      subgroup: EscalationPressure
    applicability:
      any_of:
        - record_kinds: [hazard, creature]
    guidance:
      applies_when:
        - The record creates a threat that spreads, expands, escalates, infects, contaminates, burns, floods, collapses, or otherwise worsens across space or time.
        - The changing scale of the threat shapes the scene.
      does_not_apply_when:
        - The record only deals ongoing damage to one target without broader scene spread.

  - id: scene.environment.collapse
    label: Collapse
    description: Uses falling structures, cave-ins, floor failure, debris, or unstable architecture as the main environmental threat.
    display:
      group: SceneChallenge
      subgroup: EnvironmentalThreat
    applicability:
      any_of:
        - record_kinds: [hazard]
    guidance:
      applies_when:
        - The hazard centers on collapse, structural failure, falling debris, cave-ins, crumbling floors, or unstable construction.
        - Environmental instability is more important than a generic bludgeoning damage source.
      does_not_apply_when:
        - The hazard only includes rubble or ruins as flavor without collapse shaping the scene.
````

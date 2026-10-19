# Historical snapshot: character-build.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/character-build.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: build.defense.party
    label: Party Defense
    description: Character build options that protect allies, improve group defense, redirect harm away from others, or help the party absorb danger together.
    display:
      group: CharacterBuild
      subgroup: Defense
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record primarily improves ally protection, shared mitigation, group durability, defensive positioning, or party-wide survival.
        - The defensive value meaningfully extends beyond the user's own AC, saves, resistance, or personal durability.
      does_not_apply_when:
        - The record only improves the user's own AC, saves, resistance, damage mitigation, or durability without meaningful ally-facing defensive value.
        - The record is only personally offensive or has no meaningful protective use.

  - id: build.support.healing
    label: Healing Support
    description: Character build options that restore Hit Points, stabilize allies, improve recovery, or support emergency healing.
    display:
      group: CharacterBuild
      subgroup: Support
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record is naturally retrieved for healing, recovery, stabilization, or emergency restoration.
        - Restoring or preserving party health is a primary use case.
      does_not_apply_when:
        - The record only prevents damage or grants resistance without restoring or stabilizing.

  - id: build.mobility.party
    label: Party Mobility
    description: Character build options that help the party move faster, gain movement modes, traverse terrain, or reposition safely.
    display:
      group: CharacterBuild
      subgroup: Mobility
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record grants or improves party movement, travel speed, climbing, swimming, flight, teleportation, or terrain traversal.
        - Shared movement support is a main retrieval hook.
      does_not_apply_when:
        - The record only modifies the user's own attack movement incidentally without practical shared mobility support.

  - id: build.mobility.personal
    label: Personal Mobility
    description: Character build options that improve the user's own speed, movement modes, repositioning, escape, or terrain handling.
    display:
      group: CharacterBuild
      subgroup: Mobility
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record improves the user's own movement, speed, movement mode, escape options, repositioning, or terrain traversal.
        - Self-focused mobility is the main build use case.
      does_not_apply_when:
        - The record primarily moves allies or the whole party rather than improving the user's own mobility.
        - The record only moves enemies against their will as a combat-control effect.

  - id: build.utility.stealth
    label: Stealth Support
    description: Character build options that help avoid notice, move quietly, disguise presence, or support covert approach.
    display:
      group: CharacterBuild
      subgroup: Utility
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record helps hide, avoid detection, reduce noise, disguise presence, or support infiltration.
        - Covert approach is a real use case, not just a small situational bonus.
      does_not_apply_when:
        - The record is only dark-colored, thematic, or assassin-flavored without meaningful stealth utility.

  - id: build.defense.reactive
    label: Reactive Defense
    description: Character build options whose defensive value is triggered in response to danger, attacks, failures, or incoming effects.
    display:
      group: CharacterBuild
      subgroup: Defense
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record is naturally retrieved because it reacts to attacks, failed saves, damage, being targeted, or other incoming danger.
        - Timing and response posture matter more than passive bonuses.
      does_not_apply_when:
        - The record is always-on or proactively activated without a meaningful reactive play pattern.

  - id: build.defense.personal
    label: Personal Defense
    description: Character build options that primarily improve the user's own AC, saves, resistance, mitigation, survivability, or escape from danger.
    display:
      group: CharacterBuild
      subgroup: Defense
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record is naturally retrieved because it protects the user from harm.
        - Self-focused defense, durability, resistance, damage mitigation, emergency survival, or personal escape is a primary use case.
      does_not_apply_when:
        - The record's defensive value is primarily ally-facing, party-wide, or built around protecting other creatures.
        - The record only improves offense, mobility, or exploration utility without meaningful personal protection.

  - id: build.offense.burst
    label: Burst Offense
    description: Character build options that create high short-window damage, decisive opening pressure, or a concentrated offensive payoff.
    display:
      group: CharacterBuild
      subgroup: Offense
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record is naturally retrieved for a build that wants strong immediate damage or a short offensive window.
        - The offensive value is about burst output rather than sustained routine damage.
      does_not_apply_when:
        - The record only adds small ongoing damage, control, or defensive value without a burst payoff.

  - id: build.offense.sustained
    label: Sustained Offense
    description: Character build options that support repeatable attacks, ongoing damage routines, or reliable multi-round offensive pressure.
    display:
      group: CharacterBuild
      subgroup: Offense
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record supports steady damage, repeatable attacks, ongoing pressure, or a build's ordinary offensive routine.
        - Multi-round reliability matters more than one-time burst.
      does_not_apply_when:
        - The record is mainly a single consumable spike, a defensive option, or non-damaging control.

  - id: build.offense.area
    label: Area Offense
    description: Character build options that damage, pressure, or threaten multiple creatures or zones at once.
    display:
      group: CharacterBuild
      subgroup: Offense
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record helps a build affect groups, areas, zones, lines, cones, bursts, emanations, or multiple targets with offense.
        - Multi-target or area pressure is a primary build reason to pick the option.
      does_not_apply_when:
        - The record is only single-target or uses an area for movement, protection, or utility without offensive pressure.

  - id: build.offense.precision
    label: Precision Offense
    description: Character build options that reward accuracy, setup, weak points, critical hits, precision damage, or single-target exploitation.
    display:
      group: CharacterBuild
      subgroup: Offense
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record helps a build exploit accuracy, setup, weak points, critical hits, precision damage, or single-target openings.
        - Focused single-target exploitation is a primary offensive use case.
      does_not_apply_when:
        - The record mainly produces broad area pressure, generic damage, or control without a precision or exploitation hook.

  - id: build.support.buffing
    label: Buffing Support
    description: Character build options that improve ally accuracy, defenses, damage, skills, speed, saves, or other performance.
    display:
      group: CharacterBuild
      subgroup: Support
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record improves allies or companions through bonuses, enhancements, empowerment, coordination, or other performance support.
        - Ally improvement is the main build use case.
      does_not_apply_when:
        - The record only improves the user's own offense or defense without meaningful support value.

  - id: build.support.action_economy
    label: Action Economy Support
    description: Character build options that grant, save, compress, improve, or coordinate actions for the user or allies.
    display:
      group: CharacterBuild
      subgroup: Support
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record grants actions, compresses action costs, improves reactions, supports quickened-style play, or helps allies act more efficiently.
        - Action efficiency is a primary reason to retrieve the option.
      does_not_apply_when:
        - The record only has a low action cost without improving another action pattern or build plan.

  - id: build.support.condition_removal
    label: Condition Removal Support
    description: Character build options that let a character remove, reduce, suppress, or help allies recover from harmful states.
    display:
      group: CharacterBuild
      subgroup: Support
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, character_option]
    guidance:
      applies_when:
        - The record is naturally chosen because it lets the character clean up conditions, afflictions, penalties, or other harmful states.
        - The value is a repeatable build support role, not just a one-off problem response.
      does_not_apply_when:
        - The record only prevents harm to the user or removes a problem once with no meaningful build-support identity.
````

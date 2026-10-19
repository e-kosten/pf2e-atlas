# Historical snapshot: countermeasure.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/problem-response/countermeasure.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: problem.counteract_magic
    label: Counteract Magic
    description: Helps suppress, dispel, counteract, or remove active magical effects.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment]
    guidance:
      applies_when:
        - The record explicitly dispels, counteracts, suppresses, or removes magic.
        - Stopping or unraveling existing magic is the main retrieval hook.
      does_not_apply_when:
        - The record only deals magical damage or detects magic without disrupting it.

  - id: problem.condition_removal
    label: Condition Removal
    description: Delays, suppresses, removes, or helps recover from harmful conditions and ongoing afflictions.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record removes, counteracts, suppresses, or materially mitigates a condition, affliction, or persistent harmful state.
        - Recovery or remediation is the main use case, not merely preventing ordinary damage.
      does_not_apply_when:
        - The record only grants a general bonus with no clear condition or affliction response role.

  - id: problem.curse_removal
    label: Curse Removal
    description: Identifies, suppresses, removes, reverses, or otherwise resolves an active curse.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record explicitly helps counteract, remove, suppress, reveal, or undo a curse.
        - Curse resolution is a primary retrieval hook rather than incidental occult flavor.
      does_not_apply_when:
        - The record only creates, intensifies, or thematically references curses without helping resolve one.

  - id: problem.poison_treatment
    label: Poison Treatment
    description: Prevents, delays, neutralizes, treats, or helps recover from poison exposure or poisoning.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record treats poison, improves recovery from poison, counteracts poison, or materially reduces poison consequences.
        - Poison treatment is the concrete problem being solved.
      does_not_apply_when:
        - The record only applies poison, deals poison damage, or has poison flavor without helping the target respond to poison.

  - id: problem.disease_treatment
    label: Disease Treatment
    description: Prevents, treats, counteracts, suppresses, or helps recover from disease.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record treats, counteracts, suppresses, diagnoses, or improves recovery from disease.
        - Disease treatment is a primary use case rather than a broad Fortitude or vitality bonus.
      does_not_apply_when:
        - The record only creates or spreads disease without helping respond to it.

  - id: problem.fear_recovery
    label: Fear Recovery
    description: Removes, suppresses, prevents, or helps recover from fear, panic, dread, or fear-driven impairment.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps remove, suppress, prevent, or recover from frightened, panic, dread, or similar fear effects.
        - Fear recovery matters more than creating fear pressure or using fear offensively.
      does_not_apply_when:
        - The record mainly frightens enemies or uses fear as a theme without protecting against fear.

  - id: problem.contamination_cleanup
    label: Contamination Cleanup
    description: Purifies, cleanses, neutralizes, or contains contamination, taint, pollution, or corrupting residue.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, hazard]
    guidance:
      applies_when:
        - The record helps clean up, contain, purify, or neutralize a contaminated place, object, creature, or substance.
        - Removing the contaminating source or residue is a key retrieval hook.
      does_not_apply_when:
        - The record only creates contamination or deals acid, poison, or disease damage without cleanup utility.

  - id: problem.bleed_treatment
    label: Bleed Treatment
    description: Stops, reduces, suppresses, or helps recover from bleed and similar ongoing blood-loss effects.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps stop, reduce, suppress, treat, or recover from bleeding or bleed-like ongoing damage.
        - Bleed treatment is the concrete problem being solved, not merely ordinary healing.
      does_not_apply_when:
        - The record only deals bleed damage or restores Hit Points without addressing the ongoing bleed problem.

  - id: problem.petrification_reversal
    label: Petrification Reversal
    description: Reverses, suppresses, counteracts, delays, or helps recover from petrification or being turned to stone.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps reverse, counteract, suppress, delay, or recover from petrification or being turned to stone.
        - Petrification reversal is a primary retrieval hook.
      does_not_apply_when:
        - The record causes petrification or stone transformation without helping reverse it.

  - id: problem.paralysis_recovery
    label: Paralysis Recovery
    description: Removes, suppresses, delays, mitigates, or helps recover from paralysis or similar inability to act.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps remove, suppress, delay, mitigate, or recover from paralysis or similar action-denial states.
        - Restoring the paralyzed creature's ability to act is the concrete problem being solved.
      does_not_apply_when:
        - The record causes paralysis, immobilizes enemies, or only improves general mobility without addressing paralysis.

  - id: problem.possession_expulsion
    label: Possession Expulsion
    description: Expels, suppresses, severs, counteracts, or helps recover from possession, hostile presence, or invasive spiritual control.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, lore]
    guidance:
      applies_when:
        - The record helps expel, suppress, sever, counteract, or recover from possession, hostile presence, or invasive spiritual control.
        - Removing the controlling or invasive presence is the main retrieval hook.
      does_not_apply_when:
        - The record causes possession, summons spirits, or deals spirit-themed damage without helping expel a presence.

  - id: problem.parasite_removal
    label: Parasite Removal
    description: Removes, expels, extracts, suppresses, or helps recover from parasites, implants, invasive organisms, or attached creatures.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, lore]
    guidance:
      applies_when:
        - The record helps remove, expel, extract, suppress, or recover from parasites, implants, invasive organisms, or attached creatures.
        - Removing the invasive thing is the concrete problem being solved.
      does_not_apply_when:
        - The record only summons, creates, or weaponizes parasites without helping remove one.

  - id: problem.source_neutralization
    label: Source Neutralization
    description: Disables, severs, destroys, purifies, cleanses, or otherwise neutralizes the hidden source or anchor of an ongoing problem.
    display:
      group: ProblemResponse
      subgroup: Countermeasure
    applicability:
      any_of:
        - record_kinds: [hazard, spell, equipment, feat, lore]
    guidance:
      applies_when:
        - The record helps disable, sever, destroy, cleanse, purify, or neutralize the source, anchor, or engine behind an ongoing problem.
        - Acting on the source matters more than merely finding it.
      does_not_apply_when:
        - The record only detects, locates, identifies, or explains the source without helping neutralize it.

````

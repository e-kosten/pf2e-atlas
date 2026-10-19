# Historical snapshot: investigation.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/problem-response/investigation.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: problem.source_detection
    label: Source Detection
    description: Helps identify the hidden source, anchor, origin point, contamination engine, or cause behind an ongoing danger.
    display:
      group: ProblemResponse
      subgroup: Investigation
    applicability:
      any_of:
        - record_kinds: [hazard, spell, equipment, lore]
    guidance:
      applies_when:
        - The record is retrieved because detecting what powers or causes the problem is a major part of solving it.
        - Locating the origin matters more than simply enduring exposure or executing a known disable procedure.
      does_not_apply_when:
        - The source is obvious and the remaining task is neutralizing it, surviving it, or moving past it.

  - id: problem.diagnosis
    label: Diagnosis
    description: Determines what kind of problem, affliction, malfunction, hazard, or abnormal state is present.
    display:
      group: ProblemResponse
      subgroup: Investigation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, lore]
    guidance:
      applies_when:
        - The record helps identify the nature, category, symptoms, rules, or needed remedy for an unclear problem.
        - Knowing what the problem is comes before choosing the response.
      does_not_apply_when:
        - The record only gives broad knowledge or perception without diagnosing a concrete problem.

  - id: problem.magic_detection
    label: Magic Detection
    description: Detects, identifies, locates, reads, or reveals magical auras, effects, items, or active magical sources.
    display:
      group: ProblemResponse
      subgroup: Investigation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps discover, identify, locate, or interpret magic so the party can decide what to do next.
        - Magical investigation is the main retrieval hook rather than disrupting magic.
      does_not_apply_when:
        - The record counteracts magic without helping discover or identify it.

  - id: problem.truth_revelation
    label: Truth Revelation
    description: Reveals lies, false appearances, hidden facts, deceptive claims, or concealed truths.
    display:
      group: ProblemResponse
      subgroup: Investigation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, lore]
    guidance:
      applies_when:
        - The record helps reveal deception, verify truth, uncover concealed facts, or see through false information.
        - Establishing truth is the problem being solved.
      does_not_apply_when:
        - The record only improves general social influence without revealing truth or falsehood.

  - id: problem.invisibility_detection
    label: Invisibility Detection
    description: Reveals, locates, senses, or defeats hidden presence created by invisibility or similar unseen states.
    display:
      group: ProblemResponse
      subgroup: Investigation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps locate, reveal, perceive, or counter an invisible or unseen creature or object.
        - Finding the unseen target is the core problem, not merely gaining generic Perception.
      does_not_apply_when:
        - The record makes the user invisible or hidden without helping detect invisibility.

````

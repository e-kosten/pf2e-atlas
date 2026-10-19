# Historical snapshot: risk-mitigation.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/problem-response/risk-mitigation.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: problem.alarm
    label: Alarm
    description: Warns of intrusion, danger, triggering events, tampering, or approaching threats.
    display:
      group: ProblemResponse
      subgroup: RiskMitigation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, hazard]
    guidance:
      applies_when:
        - The record creates or supports an alarm, warning, trigger notification, intrusion alert, or danger signal.
        - Early warning reduces surprise or helps the party respond before the problem fully lands.
      does_not_apply_when:
        - The record only makes noise, attacks intruders, or creates a hazard without a meaningful warning function.

  - id: problem.hazard_mitigation
    label: Hazard Mitigation
    description: Shields, wards, insulates, anchors, or otherwise reduces danger from hazards and hazardous environments.
    display:
      group: ProblemResponse
      subgroup: RiskMitigation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, hazard]
    guidance:
      applies_when:
        - The record helps avoid, withstand, reduce, or prepare for a hazard or hazardous environment.
        - Risk mitigation against a scene danger matters more than ordinary combat defense.
      does_not_apply_when:
        - The record only damages hazards or is itself a hazard without helping targets survive or avoid one.

  - id: problem.scrying_protection
    label: Scrying Protection
    description: Blocks, misdirects, reveals, or reduces remote magical observation and divination surveillance.
    display:
      group: ProblemResponse
      subgroup: RiskMitigation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record protects against scrying, remote magical observation, divination tracking, or supernatural surveillance.
        - Avoiding being watched or located is the concrete risk being mitigated.
      does_not_apply_when:
        - The record performs scrying or scouting without protecting against it.

  - id: problem.fall_protection
    label: Fall Protection
    description: Prevents, slows, cushions, mitigates, or recovers from dangerous falls and forced drops.
    display:
      group: ProblemResponse
      subgroup: RiskMitigation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record reduces fall damage, prevents a fall, slows descent, catches a falling creature, or protects against vertical hazards.
        - Falling is the practical risk being mitigated.
      does_not_apply_when:
        - The record only grants flight or climbing without a meaningful fall-response use case.

  - id: problem.death_prevention
    label: Death Prevention
    description: Prevents death, delays fatal consequences, preserves life at the edge of death, or prevents a dying creature from being lost.
    display:
      group: ProblemResponse
      subgroup: RiskMitigation
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record prevents death, stops a fatal consequence, preserves a dying creature, or creates a last-chance survival window.
        - Avoiding death is the specific risk being mitigated, not merely improving durability.
      does_not_apply_when:
        - The record only heals ordinary damage or grants broad defense without a specific death-prevention role.

````

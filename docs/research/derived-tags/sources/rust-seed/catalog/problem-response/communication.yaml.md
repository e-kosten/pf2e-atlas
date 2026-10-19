# Historical snapshot: communication.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/problem-response/communication.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: problem.language_translation
    label: Language Translation
    description: Bridges spoken or written language barriers through translation, comprehension, deciphering, or shared understanding.
    display:
      group: ProblemResponse
      subgroup: Communication
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps understand, translate, decipher, or be understood across otherwise incompatible languages or scripts.
        - Language access matters more than merely sending a message or reading thoughts.
      does_not_apply_when:
        - The record transmits content farther or more privately without solving a language barrier.

  - id: problem.message_delivery
    label: Message Delivery
    description: Sends, carries, projects, stores, or relays messages when ordinary conversation is unavailable.
    display:
      group: ProblemResponse
      subgroup: Communication
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps convey information across distance, delay, danger, privacy constraints, or separated parties.
        - Message transport is the retrieval hook rather than solving a language barrier.
      does_not_apply_when:
        - The record only translates or understands language without delivering a message.

  - id: problem.telepathic_communication
    label: Telepathic Communication
    description: Enables mind-to-mind communication, silent coordination, or private mental exchange.
    display:
      group: ProblemResponse
      subgroup: Communication
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat, creature]
    guidance:
      applies_when:
        - The record enables telepathy, mental messages, silent coordination, or private thought-based communication.
        - Communication through thought is a meaningful retrieval hook.
      does_not_apply_when:
        - The record only reads thoughts, influences minds, or deals mental damage without enabling communication.

  - id: problem.signaling
    label: Signaling
    description: Creates signals, markers, warnings, calls, beacons, or other simple coordination cues.
    display:
      group: ProblemResponse
      subgroup: Communication
    applicability:
      any_of:
        - record_kinds: [spell, equipment, feat]
    guidance:
      applies_when:
        - The record helps signal allies, mark a location, raise an alert, coordinate timing, or communicate a simple warning.
        - Signaling is distinct from full message delivery or language comprehension.
      does_not_apply_when:
        - The record only creates light, sound, or color as an attack or cosmetic effect.

````

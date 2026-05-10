import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const hazardResolutionProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.HAZARD.RESOLUTION_COUNTERMEASURE_PROFILE, {
    appeasement_countermeasure: {
      concept: "ritual_appeasement",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard best resolved through offerings, ritual respect, social appeasement, or meeting a spiritual demand.",
      appliesWhen: [
        "The hazard meaningfully invites negotiation-by-ritual, restitution, reverence, or satisfying an unmet dead or sacred demand.",
        "The nonviolent spiritual answer is more central than simply disabling mechanics or dispelling magic.",
      ],
      doesNotApplyWhen: [
        "The hazard only needs a standard disable check, counteract, or forceful destruction.",
        "The hazard is haunted but has no appeasement-style resolution path.",
      ],
      adjacentTags: ["exorcism_countermeasure", "judgment_haunt"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    contamination_cleanup_countermeasure: {
      concept: "contamination_cleanup",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard best resolved through decontamination, purification, cleansing residue, or scrubbing the hazardous space back to safety.",
      appliesWhen: [
        "The hazard is naturally retrieved because cleansing tainted ground, polluted air, cursed runoff, spores, or lingering residue is a core answer path.",
        "The cleanup process matters more than only suppressing the effect temporarily or bypassing the area.",
      ],
      doesNotApplyWhen: [
        "The hazard only has an immediate trigger or burst with no meaningful lingering contamination to clean up.",
        "The stronger fit is exorcism_countermeasure or dispel_countermeasure because the answer is purging a presence or ending an effect rather than cleaning a tainted site.",
      ],
      adjacentTags: ["quarantine_containment_countermeasure", "source_cleanup_countermeasure"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    dispel_countermeasure: {
      concept: "active_magic_counteraction",
      label: "countermagic",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard meaningfully invites counteract, dispel, or magical suppression as a core answer path.",
      appliesWhen: [
        "A user would plausibly retrieve the hazard because anti-magic answers are central to resolving it.",
        "Magical suppression matters more than physical disarm or spiritual appeasement.",
      ],
      doesNotApplyWhen: [
        "The hazard is magical but best solved through rituals, offerings, or physical tampering instead.",
        "Counteracting is only a minor optional answer path.",
      ],
      adjacentTags: ["physical_disarm", "ward_trigger"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    exorcism_countermeasure: {
      concept: "hostile_presence_expulsion",
      label: "exorcism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard best resolved through banishment, exorcism, consecration, or another spirit-cleansing answer.",
      appliesWhen: [
        "The hazard is naturally retrieved because cleansing, banishing, or sanctifying the hostile presence is a core answer path.",
        "A spiritual purge matters more than appeasement, anti-magic suppression, or physical mechanism work.",
      ],
      doesNotApplyWhen: [
        "The hazard mainly wants offerings, restitution, or ritual respect rather than expulsion.",
        "The hazard is magical or mechanical but not really spirit-cleansed out of existence.",
      ],
      adjacentTags: ["appeasement_countermeasure", "dispel_countermeasure"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    physical_disarm: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard meaningfully invites physical mechanism tampering, disassembly, or trigger-blocking as the core answer path.",
      appliesWhen: [
        "The hazard is naturally retrieved because tools, hands-on disable work, or mechanism access are central to solving it.",
        "Mechanical tampering matters more than safe sequencing, anti-magic, or spiritual negotiation.",
      ],
      doesNotApplyWhen: [
        "The hazard is mainly solved by learning the right pattern, dispelling an effect, or meeting a spiritual demand.",
        "Physical interaction exists only as one optional fallback rather than the main resolution mode.",
      ],
      adjacentTags: ["procedural_bypass", "dispel_countermeasure"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    procedural_bypass: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard best bypassed through the right route, timing, command phrase, ritual sequence, or other correct procedure rather than direct disarm.",
      appliesWhen: [
        "The clean answer is learning and executing the hazard's safe procedure, sequence, or pattern rather than destroying it.",
        "A GM would plausibly retrieve the hazard for puzzle-like bypass, safe-route discovery, or passphrase-style navigation.",
      ],
      doesNotApplyWhen: [
        "The main answer is simple mechanism tampering, counteracting magic, or appeasing a spirit.",
        "The hazard only has a minor caution or tactical workaround without a real procedural solution.",
      ],
      adjacentTags: ["physical_disarm", "false_safe_route"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    quarantine_containment_countermeasure: {
      concept: "outbreak_containment",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard best managed by isolating victims, sealing off the site, or imposing containment boundaries that stop spread while the danger is being handled.",
      appliesWhen: [
        "The hazard is naturally retrieved because the first meaningful answer is locking down spread, controlling access, or containing dangerous exposure.",
        "Containment procedures matter more than immediately dispelling, disarming, or appeasing the hazard.",
      ],
      doesNotApplyWhen: [
        "The hazard is dangerous but does not meaningfully spread, linger, or demand isolation and access control.",
        "The stronger fit is barrier_lockdown or sentinel_guardian because preventing passage is the hazard's function, not the party's resolution plan.",
      ],
      adjacentTags: ["contamination_cleanup_countermeasure", "source_cleanup_countermeasure"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    source_cleanup_countermeasure: {
      concept: "source_cleanup",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard best resolved by locating and neutralizing the cursed source, leaking node, corrupted remains, or other origin driving the dangerous field.",
      appliesWhen: [
        "The hazard is naturally retrieved because the real answer is finding and dealing with the source object, origin point, or contamination engine.",
        "Neutralizing the origin matters more than only enduring the space or treating downstream symptoms.",
      ],
      doesNotApplyWhen: [
        "The hazard has no meaningful source object, leak point, or origin to clean up beyond the hazard itself.",
        "The stronger fit is procedural_bypass or physical_disarm because the answer is executing a sequence or tampering with a mechanism rather than eliminating an origin source.",
      ],
      adjacentTags: ["quarantine_containment_countermeasure", "contamination_cleanup_countermeasure"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"hazard">[];

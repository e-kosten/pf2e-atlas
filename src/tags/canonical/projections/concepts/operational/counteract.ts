import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalCounteractProjectionDeclarations = [
  defineConceptProjections("active_magic_counteraction", {
    affliction: {
      tag: "countermagic_resolution",
      label: "countermagic",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because counteracting, suppressing, or dispelling an active magical affliction is central to solving it.",
      adjacentTags: ["antidote_resolution", "cursebreaking_resolution"],
    },
    equipment: {
      tag: "countermagic",
      label: "countermagic",
      axis: "utility",
      family: "anti_magic",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Counteracts, dispels, suppresses, or shuts down magic.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      appliesWhen: [
        "The item's main value is actively cancelling, suppressing, or interfering with hostile or ongoing magic.",
        "A user would retrieve it as an anti-magic tool rather than a general protective charm.",
      ],
      doesNotApplyWhen: [
        "The item only protects the wearer from magical harm without disrupting the spell itself.",
        "The item focuses on blocking surveillance or hiding information rather than broader anti-magic interference.",
      ],
      adjacentTags: ["magic_protection", "scrying_protection"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "dispel_countermeasure",
      label: "countermagic",
      axis: "resolution",
      family: "countermeasure_profile",
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
    spell: {
      tag: "countermagic",
      label: "countermagic",
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Counteracts, dispels, suppresses, or shuts down magic.",
      appliesWhen: [
        "The spell is naturally retrieved because stopping, unravelling, or suppressing existing magic is its main job.",
        "Anti-magic response matters more than simple protection, detection, or concealment.",
      ],
      doesNotApplyWhen: [
        "The spell mainly protects targets from harm without actually disrupting hostile magic.",
        "The spell only reveals or warns about magic rather than counteracting it.",
      ],
      adjacentTags: ["magic_detection", "protective_ward"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

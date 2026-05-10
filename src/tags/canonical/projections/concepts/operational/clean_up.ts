import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalCleanUpProjectionDeclarations = [
  defineConceptProjections("affliction_cleanup", {
    spell: {
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Cleanses, cures, neutralizes, or removes disease, poison, curse, or similar afflictions.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("contamination_cleanup", {
    equipment: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps neutralize tainted residue, clean corrupted surfaces, purify contaminated supplies, or scrub a dangerous site back to safety.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      adjacentTags: ["quarantine_containment", "source_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "contamination_cleanup_countermeasure",
      axis: "resolution",
      family: "countermeasure_profile",
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
    spell: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Cleanses tainted residue, neutralizes corrupted ground, removes lingering pollution, or purifies a contaminated space.",
      adjacentTags: ["quarantine_containment", "source_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("source_cleanup", {
    affliction: {
      tag: "source_cleanup_resolution",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because the contaminated site, cursed source, infected carrier chain, or environmental origin must be found and cleaned up.",
      adjacentTags: ["source_tracing", "quarantine_containment_resolution"],
    },
    equipment: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps find, remove, neutralize, or safely dispose of the cursed object, infected material, corrupted remains, or other source driving the problem.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      adjacentTags: ["source_revelation", "contamination_cleanup", "ritual_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    hazard: {
      tag: "source_cleanup_countermeasure",
      axis: "resolution",
      family: "countermeasure_profile",
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
    spell: {
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Neutralizes, destroys, seals, or cleans up the cursed object, infected origin, corrupted site, or anchored source driving the problem.",
      adjacentTags: ["source_revelation", "contamination_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

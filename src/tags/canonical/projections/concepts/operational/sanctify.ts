import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalSanctifyProjectionDeclarations = [
  defineConceptProjections("sacred_taint_sanctification", {
    equipment: {
      tag: "sanctification",
      label: "sanctification",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Supports hallowing, consecration, spiritual purification, or cleansing rites applied to a creature, object, or site.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      appliesWhen: [
        "The item's retrieval value comes from consecrating, hallowing, purifying, or spiritually cleansing a target or place.",
        "It is naturally sought as part of sacred-site cleanup, anti-haunt work, or ritual purification rather than general divine symbolism.",
      ],
      doesNotApplyWhen: [
        "The item is only religious, ceremonial, or devotional without materially helping purification or consecration.",
        "The stronger fit is ritual_support because the item supports a broad rite rather than sanctification in particular.",
      ],
      adjacentTags: ["ritual_support", "ritual_appeasement", "curse_removal"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "sanctification",
      label: "sanctification",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Consecrates, hallowes, purifies, or spiritually cleanses a creature, object, or site to solve a malign supernatural problem.",
      adjacentTags: ["ritual_appeasement", "exorcism", "protective_ward"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

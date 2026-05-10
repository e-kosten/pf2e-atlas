import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalRemediateProjectionDeclarations = [
  defineConceptProjections("bleed_remediation", {
    equipment: {
      tag: "anti_bleed",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps staunch bleeding, end persistent bleed damage, or close ongoing wounds.",
      subcategories: ["consumable"],
    },
    spell: {
      tag: "anti_bleed",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Staunches bleeding, ends persistent bleed damage, or closes wounds that keep draining a target.",
      adjacentTags: ["healing_support", "condition_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("caster_disruption_remediation", {
    equipment: {
      tag: "anti_caster_disruption",
      axis: "effect",
      family: "offensive_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Punishes casting, disrupts concentration, or is naturally retrieved to make enemy spell use unreliable.",
      subcategories: ["ammo", "consumable"],
      adjacentTags: ["silencing", "countermagic"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "anti_caster_disruption",
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Disrupts casting, punishes spell use, or specifically suppresses hostile spellcasters in the moment.",
      adjacentTags: ["countermagic", "silencing"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("confusion_remediation", {
    equipment: {
      tag: "anti_confusion",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps clear confusion, restore mental steadiness, or recover from disordered thinking.",
      subcategories: ["consumable"],
    },
    spell: {
      tag: "anti_confusion",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Ends confusion, steadies a disordered mind, or protects a target from confusion-like mental unraveling.",
      adjacentTags: ["condition_support", "anti_fear"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("curse_remediation", {
    affliction: {
      tag: "cursebreaking_resolution",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because lifting a curse, breaking a doom, or ending a supernatural binding is central to recovery.",
      adjacentTags: ["countermagic_resolution", "ritual_appeasement_resolution"],
    },
    equipment: {
      tag: "curse_removal",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps remove, break, or counteract curses as a direct answer path rather than only easing symptoms.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      adjacentTags: ["sanctification", "source_cleanup"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "curse_removal",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Breaks, removes, or counteracts curses as a direct answer path rather than only suppressing symptoms.",
      adjacentTags: ["exorcism", "sanctification"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("disease_remediation", {
    equipment: {
      tag: "anti_disease",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps resist, prevent, or recover from disease.",
      subcategories: ["consumable"],
    },
    spell: {
      tag: "anti_disease",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Cures disease, counteracts infections, or protects a target against plague, fever, and similar disease effects.",
      adjacentTags: ["affliction_cleanup", "anti_poison"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("fear_remediation", {
    equipment: {
      tag: "anti_fear",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps resist fear, recover from frightened effects, or steady courage.",
      subcategories: ["consumable"],
    },
    spell: {
      tag: "anti_fear",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Counters frightened or fear effects, bolsters courage, or protects a target against fear.",
      adjacentTags: ["condition_support", "anti_confusion"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("paralysis_remediation", {
    equipment: {
      tag: "anti_paralysis",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps break paralysis, restore movement, or free a creature from immobilizing body shutdown.",
      subcategories: ["consumable"],
    },
    spell: {
      tag: "anti_paralysis",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Ends paralysis, restores bodily function, or frees a target from magic or afflictions that leave it unable to move.",
      adjacentTags: ["condition_support", "escape_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("petrification_remediation", {
    equipment: {
      tag: "anti_petrification",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps prevent or reverse petrification and other stone-turning effects.",
      subcategories: ["consumable"],
    },
    spell: {
      tag: "anti_petrification",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Prevents, reverses, or counteracts petrification and other turn-to-stone effects.",
      adjacentTags: ["affliction_cleanup", "condition_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("poison_remediation", {
    affliction: {
      tag: "antidote_resolution",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because antitoxins, antidotes, neutralizing medicine, or poison-specific treatment are central to solving it.",
      adjacentTags: ["countermagic_resolution", "cure_clock_urgency"],
    },
    equipment: {
      tag: "anti_poison",
      axis: "effect",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Helps resist, prevent, or recover from poison.",
      subcategories: ["consumable"],
    },
    spell: {
      tag: "anti_poison",
      axis: "support",
      family: "support",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Cures poison, counters toxic afflictions, or protects a target against venom and similar poisoning effects.",
      adjacentTags: ["affliction_cleanup", "anti_disease"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

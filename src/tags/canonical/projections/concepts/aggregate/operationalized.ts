import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const AggregateOperationalizedProjectionDeclarations = [
  defineConceptProjections("breaching", {
    equipment: {
      axis: "utility",
      family: "breaching",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad force-entry umbrella for equipment used to break doors, barriers, fortifications, or route-blocking structures.",
      subcategories: ["gear", "kit", "consumable", "weapon"],
      adjacentTags: ["door_breaching", "barrier_breaking", "demolition"],
      compositeOfAnyTags: ["door_breaching", "barrier_breaking", "excavation", "siege_support", "demolition"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("displacement_application", {
    hazard: {
      tag: "displacement_hazard",
      axis: "effect",
      family: "forced_position",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad displacement umbrella for hazards that drop, collapse, shove, or split creatures apart through positional disruption.",
      adjacentTags: ["pitfall", "collapse_hazard", "forced_separation"],
      compositeOfAnyTags: ["pitfall", "collapse_hazard", "forced_movement", "forced_separation"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("environmental_application", {
    hazard: {
      tag: "environmental_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad environmental umbrella for elemental, toxic, contaminating, and terrain-corrupting hazards that threaten a space through recurring exposure.",
      adjacentTags: ["fire_hazard", "poison_hazard", "contamination_hazard"],
      compositeOfAnyTags: [
        "acid_hazard",
        "cold_hazard",
        "fire_hazard",
        "electric_hazard",
        "poison_hazard",
        "respiratory_hazard",
        "sound_hazard",
        "water_hazard",
        "contamination_hazard",
        "blight_hazard",
        "overgrowth_hazard",
        "cursefield_hazard",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("perception_application", {
    hazard: {
      tag: "perception_hazard",
      axis: "effect",
      family: "perception_control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad perception umbrella for hazards that attack through distorted routes, hostile illusion, or deceptive pathing rather than direct force alone.",
      adjacentTags: ["navigation_disruption", "illusion_assault", "false_safe_route"],
      compositeOfAnyTags: ["navigation_disruption", "illusion_assault", "false_safe_route"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("problem_discovery", {
    spell: {
      tag: "revelation",
      axis: "utility",
      family: "revelation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad reveal umbrella for spells that detect magic, uncover deceptions, expose invisible threats, or identify hidden supernatural problems.",
      adjacentTags: ["magic_detection", "truth_reveal", "hazard_revelation"],
      compositeOfAnyTags: [
        "magic_detection",
        "invisibility_reveal",
        "truth_reveal",
        "curse_revelation",
        "hazard_revelation",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("problem_resolution", {
    equipment: {
      tag: "resolution",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad resolution umbrella for equipment used to break curses, sanctify places, contain spread, clean contamination, or solve a problem at its source.",
      subcategories: ["gear", "backpack", "kit", "consumable"],
      adjacentTags: [
        "curse_removal",
        "ritual_appeasement",
        "source_revelation",
        "contamination_cleanup",
        "source_cleanup",
      ],
      compositeOfAnyTags: [
        "curse_removal",
        "sanctification",
        "ritual_appeasement",
        "source_revelation",
        "quarantine_containment",
        "contamination_cleanup",
        "source_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      tag: "resolution",
      axis: "utility",
      family: "resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad resolution umbrella for spells that break curses, expel hostile presences, contain spread, purify contamination, or solve a supernatural problem at its source.",
      adjacentTags: ["curse_removal", "exorcism", "ritual_appeasement", "source_cleanup"],
      compositeOfAnyTags: [
        "curse_removal",
        "exorcism",
        "sanctification",
        "ritual_appeasement",
        "quarantine_containment",
        "contamination_cleanup",
        "source_revelation",
        "source_cleanup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("transformation", {
    spell: {
      axis: "transformation",
      family: "transformation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description: "Spells that alter a creature's body, form, or battle shape.",
      adjacentTags: ["battle_form", "animal_form", "elemental_form"],
      compositeOfAnyTags: ["battle_form", "animal_form", "elemental_form"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

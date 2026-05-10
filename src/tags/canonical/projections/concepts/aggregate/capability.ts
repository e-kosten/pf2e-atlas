import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const AggregateCapabilityProjectionDeclarations = [
  defineConceptProjections("communication", {
    equipment: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad communication umbrella for equipment used to signal allies, relay messages, bridge language barriers, or coordinate silently.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      adjacentTags: ["signaling", "telepathic_communication", "message_delivery"],
      compositeOfAnyTags: ["signaling", "telepathic_communication", "message_delivery", "translation_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "communication",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad communication umbrella for spells used to signal allies, relay messages, bridge language barriers, or coordinate silently.",
      adjacentTags: ["signaling", "telepathic_communication", "message_delivery", "translation_support"],
      compositeOfAnyTags: ["signaling", "telepathic_communication", "message_delivery", "translation_support"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("consultation", {
    spell: {
      axis: "utility",
      family: "consultation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad consultation umbrella for spells used to seek cosmic answers, diagnose mysteries, or gain non-sensory divinatory guidance.",
      adjacentTags: ["lore_consultation", "problem_diagnosis", "omen_guidance"],
      compositeOfAnyTags: ["lore_consultation", "problem_diagnosis", "omen_guidance"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("expedition", {
    equipment: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad expedition umbrella for travel gear, camp support, sustainment, mounts, aquatic operations, and hostile-environment endurance.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable", "armor", "shield", "weapon"],
      adjacentTags: ["survival", "sustenance", "environmental_adaptation"],
      compositeOfAnyTags: [
        "survival",
        "mounted_support",
        "sustenance",
        "aquatic_support",
        "environmental_adaptation",
        "camp_setup",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "expedition",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad expedition umbrella for spells used for routefinding, travel-ready movement, shelter, sustainment, aquatic operations, and hostile-environment survival.",
      adjacentTags: ["navigation", "field_shelter", "environmental_adaptation"],
      compositeOfAnyTags: [
        "navigation",
        "flight",
        "aquatic_support",
        "sustenance",
        "field_shelter",
        "environmental_adaptation",
        "wayfinding",
      ],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("movement_traversal", {
    equipment: {
      axis: "utility",
      family: "movement_traversal",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad movement-and-travel umbrella for equipment that solves climbing, routefinding, repositioning, or transport problems.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      adjacentTags: ["climbing", "navigation", "transport"],
      compositeOfAnyTags: ["climbing", "mobility", "navigation", "transport"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("reconnaissance", {
    equipment: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad recon umbrella for equipment used to scout, illuminate, record evidence, track targets, or frustrate pursuit.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      adjacentTags: ["scouting", "tracking", "anti_tracking"],
      compositeOfAnyTags: ["scouting", "illumination", "surveillance_recording", "tracking", "anti_tracking"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "reconnaissance",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad scouting umbrella for spells that gather remote information, extend senses, or track a target from afar.",
      adjacentTags: ["scouting", "tracking", "scouting_summons"],
      compositeOfAnyTags: ["scouting", "tracking", "scouting_summons"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("security", {
    equipment: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad security umbrella for gear that warns about intrusion, blocks magical spying, or reveals after-the-fact interference.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "consumable"],
      adjacentTags: ["alarm", "scrying_protection", "tamper_evidence"],
      compositeOfAnyTags: ["alarm", "scrying_protection", "tamper_evidence"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "security",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad security umbrella for spells that warn about intrusion, protect private spaces, or harden a target against magical observation and interference.",
      adjacentTags: ["alarm", "scrying_protection", "protective_ward"],
      compositeOfAnyTags: ["alarm", "scrying_protection", "protective_ward", "countermagic"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("wayfinding", {
    spell: {
      axis: "utility",
      family: "wayfinding",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad route-and-destination umbrella for spells that orient travel, locate a target destination, or bypass distance through strategic movement magic.",
      adjacentTags: ["navigation", "tracking", "long_range_teleport"],
      compositeOfAnyTags: ["navigation", "tracking", "long_range_teleport", "planar_travel"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalApplyProjectionDeclarations = [
  defineConceptProjections("acid_application", {
    hazard: {
      tag: "acid_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on acid, corrosive spray, caustic runoff, or similar corrosive exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("blight_application", {
    hazard: {
      tag: "blight_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard centered on ecological ruin, land-sickening corruption, withering growth, or terrain spoiled by supernatural blight.",
      adjacentTags: ["contamination_hazard", "overgrowth_hazard"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("cold_application", {
    hazard: {
      tag: "cold_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on ice, frost, freezing, blizzards, or other cold exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("collapse_application", {
    hazard: {
      tag: "collapse_hazard",
      axis: "effect",
      family: "forced_position",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Hazard built around collapsing structures, cave-ins, rockfalls, or crumbling ground.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("contamination_application", {
    hazard: {
      tag: "contamination_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard centered on tainted residue, corruptive seepage, drifting spores, cursed runoff, or other lingering contamination of a space.",
      adjacentTags: ["poison_hazard", "respiratory_hazard"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("curse_application", {
    creature: {
      tag: "curse_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Threat defined by curses, doom effects, or other lingering supernatural afflictions imposed on victims.",
    },
  }),
  defineConceptProjections("cursefield_application", {
    hazard: {
      tag: "cursefield_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard centered on cursed ground, spiritually poisoned space, or a zone whose danger comes from active supernatural contamination rather than one mechanism.",
      adjacentTags: ["contamination_hazard", "judgment_haunt"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("death_burst", {
    creature: {
      tag: "death_burst_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Threat defined by explosive death effects, cursed aftermath, or punishing consequences when the creature is dropped.",
    },
  }),
  defineConceptProjections("disease_application", {
    creature: {
      tag: "disease_vector",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Threat defined by spreading disease, curse-plague conditions, or infectious aftermath beyond immediate damage.",
    },
  }),
  defineConceptProjections("electric_application", {
    hazard: {
      tag: "electric_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on lightning, shock, static discharge, or electrical exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("fire_application", {
    hazard: {
      tag: "fire_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on open fire, flames, burning spread, or explosive ignition.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("life_drain_application", {
    creature: {
      tag: "life_drain_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by draining blood, vitality, life force, or souls from victims.",
    },
  }),
  defineConceptProjections("overgrowth_application", {
    hazard: {
      tag: "overgrowth_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard centered on choking roots, hostile vines, grasping thorns, or other dangerous living overgrowth that turns terrain against intruders.",
      adjacentTags: ["blight_hazard", "forced_movement"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("petrification_application", {
    creature: {
      tag: "petrification_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by petrifying victims or turning them to stone.",
    },
  }),
  defineConceptProjections("poison_application", {
    creature: {
      tag: "poison_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Threat defined by venom, toxic excretions, poisoned weapons, or other recurring poison delivery.",
    },
    hazard: {
      tag: "poison_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on poison gas, toxic delivery, or other poisonous exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("possession_application", {
    creature: {
      tag: "possession_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Can possess, body-snatch, or take control of a victim from within.",
      appliesWhen: [
        "Use when entering, riding, replacing, or controlling a host body is a major reason to retrieve the creature.",
        "The possession dynamic matters more than ordinary charm, domination, or haunting flavor.",
      ],
      doesNotApplyWhen: [
        "The creature only compels, frightens, or mentally influences targets without true body-occupying takeover.",
        "The stronger fit is curse_threat or reinforcement_threat because possession is not central to encounter prep.",
      ],
      adjacentTags: ["curse_threat", "reinforcement_threat"],
    },
  }),
  defineConceptProjections("respiratory_application", {
    hazard: {
      tag: "respiratory_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on smoke, choking vapor, breathlessness, or impaired breathing.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("sound_application", {
    hazard: {
      tag: "sound_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on sonic force, deafening noise, vibration, or resonant disruption.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("water_application", {
    hazard: {
      tag: "water_hazard",
      axis: "effect",
      family: "environmental_danger",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard centered on floods, geysers, waves, surges, or other dangerous water exposure.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

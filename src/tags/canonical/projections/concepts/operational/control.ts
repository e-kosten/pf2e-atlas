import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalControlProjectionDeclarations = [
  defineConceptProjections("compulsion_control", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Forces scripted behavior, movement, or obedience against a target's normal will.",
      appliesWhen: [
        "The spell explicitly pressures the target into doing something, moving somewhere, or obeying a commanded pattern.",
        "Loss of agency is more important than affection, calm, or broad mood change.",
      ],
      doesNotApplyWhen: [
        "The spell merely charms or emotionally softens the target.",
        "The spell fully dominates the target over sustained actions rather than issuing narrower commands.",
      ],
      adjacentTags: ["charm_influence", "domination"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("emotion_control", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Directly manipulates fear, calm, rage, love, despair, or other emotional states.",
      appliesWhen: [
        "The spell is naturally retrieved for changing a creature's feelings, morale, or emotional volatility.",
        "The emotional state change matters more than explicit obedience or truth extraction.",
      ],
      doesNotApplyWhen: [
        "The spell chiefly compels discrete actions or sustained domination.",
        "The spell only inflicts fear as damage pressure without broader emotional steering.",
      ],
      adjacentTags: ["fear_pressure", "charm_influence"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("line_of_sight_control", {
    equipment: {
      axis: "effect",
      family: "offensive_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Obscures vision, fills an area with smoke, or otherwise denies clear sight lines as the item's main offensive or tactical job.",
      subcategories: ["ammo", "consumable"],
      adjacentTags: ["sensory_impairment", "restraint_capture"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "battlefield",
      family: "control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Blocks vision, obscures sight lines, or denies clear observation across an area.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("prey_control", {
    creature: {
      tag: "prey_control_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Threat defined by holding prey in place through grabs, constriction, webbing, swallowing, or other ongoing body-control pressure.",
      adjacentTags: ["ambush_grabber", "terrain_control_threat"],
    },
  }),
  defineConceptProjections("terrain_control", {
    creature: {
      tag: "terrain_control_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Threat defined by webs, walls, zones, hazards, or other space-shaping control that changes battlefield movement.",
    },
  }),
] satisfies ConceptProjectionDeclaration[];

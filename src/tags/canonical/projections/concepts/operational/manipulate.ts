import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalManipulateProjectionDeclarations = [
  defineConceptProjections("mechanism_manipulation", {
    equipment: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Helps operate levers, latches, panels, pressure surfaces, or similar scene mechanisms from a safer or more advantageous position.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "weapon"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    spell: {
      axis: "utility",
      family: "access_bypass",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Precisely triggers, moves, holds, or operates levers, buttons, switches, pressure plates, locks, or similar scene mechanisms.",
      appliesWhen: [
        "The spell is naturally retrieved to operate a lever, button, latch, control panel, pressure surface, or similar mechanism from a safe or unusual position.",
        "The mechanism interaction itself matters more than broad telekinesis, damage, or ordinary object movement.",
      ],
      doesNotApplyWhen: [
        "The spell only moves creatures or loose objects without a real access-, control-, or mechanism-facing use case.",
        "The spell bypasses the obstacle by teleporting or destroying it instead of operating the mechanism.",
      ],
      adjacentTags: ["lock_bypass", "trap_bypass"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
  defineConceptProjections("memory_manipulation", {
    spell: {
      axis: "influence",
      family: "influence",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Edits, suppresses, restores, or rewrites memories, recollection, and remembered events.",
      appliesWhen: [
        "The spell is naturally retrieved for altering what a target remembers, forgets, or believes it experienced.",
        "Memory editing is more central than charm, emotion, or truth exposure.",
      ],
      doesNotApplyWhen: [
        "The spell only reveals truth or emotions without changing stored recollection.",
        "The spell primarily imposes obedience or domination in the present moment.",
      ],
      adjacentTags: ["truth_reveal", "charm_influence"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const spellEffectProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.SPELL.EFFECT_IMPACT, {
    burst_damage: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Delivers concentrated damage in a spike or splash pattern that users naturally retrieve as a damage-first answer.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    crowd_clearing: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Damages or wipes clusters of weaker foes and is naturally retrieved as an anti-group answer.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    mental_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description:
        "Impairs thought, composure, or agency through fear, confusion, or similarly hostile mental effects.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    persistent_damage: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Directly inflicts persistent damage or grants attacks that reliably impose persistent damage.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    sensory_impairment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Blinds, deafens, or otherwise directly suppresses a creature's senses.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
    single_target_removal: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Drops, banishes, petrifies, or otherwise decisively removes one important creature from the fight.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"spell">[];

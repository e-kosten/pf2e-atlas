import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalRemoveProjectionDeclarations = [
  defineConceptProjections("parasite_removal", {
    affliction: {
      tag: "surgical_extraction_resolution",
      axis: "response",
      family: "resolution_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because removing eggs, larvae, parasites, implants, or invasive growth from the body is a central answer path.",
      adjacentTags: ["source_cleanup_resolution", "infestation_implant"],
    },
  }),
  defineConceptProjections("single_target_removal", {
    spell: {
      axis: "effect",
      family: "impact",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Drops, banishes, petrifies, or otherwise decisively removes one important creature from the fight.",
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

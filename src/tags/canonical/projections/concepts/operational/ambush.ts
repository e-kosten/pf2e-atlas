import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalAmbushProjectionDeclarations = [
  defineConceptProjections("ambush_grab", {
    creature: {
      tag: "ambush_grabber",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Captures prey through grabbing, constriction, swallowing whole, webbing, or drag-off ambush tactics.",
    },
  }),
] satisfies ConceptProjectionDeclaration[];

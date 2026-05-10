import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const AggregateFunctionProjectionDeclarations = [
  defineConceptProjections("guarding_hazard", {
    hazard: {
      axis: "encounter",
      family: "function",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.COMPOSITE,
      description:
        "Broad guarding umbrella for hazards that warn, lock down, or stand watch over a threshold, object, or protected space.",
      adjacentTags: ["alarm", "barrier_lockdown", "sentinel_guardian"],
      compositeOfAnyTags: ["alarm", "barrier_lockdown", "sentinel_guardian"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

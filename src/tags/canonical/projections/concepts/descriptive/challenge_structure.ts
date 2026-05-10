import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveChallengeStructureProjectionDeclarations = [
  defineConceptProjections("endurance_pressure", {
    hazard: {
      axis: "problem",
      family: "problem_shape",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard whose main prep problem is surviving repeated exposure long enough to finish the scene rather than landing one clean solve immediately.",
    },
  }),
  defineConceptProjections("multi_stage_resolution", {
    hazard: {
      tag: "layered_resolution",
      axis: "problem",
      family: "problem_shape",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard that asks the party to solve multiple linked pieces rather than one single disable check or obvious answer.",
    },
  }),
  defineConceptProjections("observation_driven", {
    hazard: {
      tag: "observation_first",
      axis: "problem",
      family: "problem_shape",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard that rewards careful watching, clue gathering, or reading the environment before a safe approach becomes obvious.",
    },
  }),
  defineConceptProjections("timing_window", {
    hazard: {
      axis: "problem",
      family: "problem_shape",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard that is best handled by acting during the right cycle, opening, lull, or repeating timing pattern.",
    },
  }),
] satisfies ConceptProjectionDeclaration[];

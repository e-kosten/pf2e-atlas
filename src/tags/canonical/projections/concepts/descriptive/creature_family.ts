import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const DescriptiveCreatureFamilyProjectionDeclarations = [
  defineConceptProjections("animated_object", {
    creature: {
      axis: "presentation",
      family: "bound_object",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with animated objects, furniture, tools, or other constructed items.",
    },
  }),
  defineConceptProjections("animated_statue", {
    creature: {
      axis: "presentation",
      family: "bound_object",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Strongly associated with animated statues, effigies, idols, or monuments.",
    },
  }),
  defineConceptProjections("possessed_object", {
    creature: {
      axis: "presentation",
      family: "bound_object",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Strongly associated with an inhabiting spirit or curse animating an otherwise mundane object or suit of equipment.",
      appliesWhen: [
        "Use when a spirit, ghost, curse, or other external presence is explicitly what animates the object.",
        "The inhabiting presence matters more than the object's construction, material, or generic animation.",
      ],
      doesNotApplyWhen: [
        "The object is simply animated by magic, clockwork, or sculpted animation with no real possessing force.",
        "The stronger fit is animated_object or animated_statue because possession is not central.",
      ],
      adjacentTags: ["animated_object", "possession_threat"],
    },
  }),
  defineConceptProjections("sinspawn_family", {
    creature: {
      axis: "specialization",
      family: "ontology_cluster",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Groups sinspawn and close runelord-bred sinspawn offshoots into one retrieval bucket.",
      nativeOntologyPolicy: "aggregates_native_signals",
    },
  }),
  defineConceptProjections("undead_family", {
    creature: {
      tag: "undead_adjacent",
      axis: "specialization",
      family: "ontology_cluster",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Groups undead and closely undead-coded native signals into one retrieval bucket.",
      nativeOntologyPolicy: "aggregates_native_signals",
    },
  }),
] satisfies ConceptProjectionDeclaration[];

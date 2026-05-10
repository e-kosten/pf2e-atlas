import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalDisarmProjectionDeclarations = [
  defineConceptProjections("physical_disarm", {
    hazard: {
      axis: "resolution",
      family: "countermeasure_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Hazard meaningfully invites physical mechanism tampering, disassembly, or trigger-blocking as the core answer path.",
      appliesWhen: [
        "The hazard is naturally retrieved because tools, hands-on disable work, or mechanism access are central to solving it.",
        "Mechanical tampering matters more than safe sequencing, anti-magic, or spiritual negotiation.",
      ],
      doesNotApplyWhen: [
        "The hazard is mainly solved by learning the right pattern, dispelling an effect, or meeting a spiritual demand.",
        "Physical interaction exists only as one optional fallback rather than the main resolution mode.",
      ],
      adjacentTags: ["procedural_bypass", "dispel_countermeasure"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

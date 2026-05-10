import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalMisleadProjectionDeclarations = [
  defineConceptProjections("false_safe_route", {
    hazard: {
      axis: "effect",
      family: "perception_control",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Hazard that tempts intruders toward a seemingly safer path or escape line that is itself the trap.",
      appliesWhen: [
        "The hazard actively misdirects intruders toward a route that looks protective, faster, or safer but is actually the danger.",
        "The retrieval hook is deceptive path choice rather than only illusion damage or generic navigation confusion.",
      ],
      doesNotApplyWhen: [
        "The hazard merely scrambles orientation without presenting a tempting fake safe path.",
        "The hazard is solved through procedure, but misleading route presentation is not central to how it works.",
      ],
      adjacentTags: ["navigation_disruption", "procedural_bypass"],
      translationStatus: CANONICAL_VOCABULARY.TRANSLATION.STATUS.PROVISIONAL,
    },
  }),
] satisfies ConceptProjectionDeclaration[];

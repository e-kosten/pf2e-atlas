import { defineConceptProjections, type ConceptProjectionDeclaration } from "../../../builders.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const OperationalCallProjectionDeclarations = [
  defineConceptProjections("reinforcement", {
    creature: {
      tag: "reinforcement_threat",
      axis: "encounter",
      family: "threat_profile",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Threat defined by materially changing encounter structure through added bodies, activated subordinates, or sharply elevated allied creatures.",
      appliesWhen: [
        "Use when the creature's main prep significance is that it adds bodies, activates subordinates, or sharply force-multiplies nearby allies.",
        "The encounter meaningfully changes because of its reinforcement engine rather than just because it personally hits hard.",
      ],
      doesNotApplyWhen: [
        "The creature only has one minor ally-facing buff or an incidental summon without materially changing encounter structure.",
        "The stronger fit is support_combatant, commander_combatant, or spawn_creator because reinforcement is not the real threat hook.",
      ],
      adjacentTags: ["spawn_creator", "commander_combatant"],
    },
  }),
] satisfies ConceptProjectionDeclaration[];

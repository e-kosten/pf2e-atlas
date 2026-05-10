import { defineAssignments, tag } from "../builders.js";

export const PATHFINDER_NPC_CORE_DERIVED_TAG_ASSIGNMENTS = defineAssignments({
  "pathfinder-npc-core:MxcprNbX7hcpAU8p": {
    name: "Departmental Chair",
    applied: [
      tag("civic_npc", {
        source: "human",
        confidence: "high",
        rationale: "Fits the civic and social fabric of an urban institution-driven scene.",
      }),
      tag("profession_npc", {
        source: "human",
        confidence: "high",
        rationale: "Presented as a role-defined academic administrator rather than as a generic monster encounter.",
      })
    ],
  },
  "pathfinder-npc-core:OAxxUyACpMlX3q1X": {
    name: "False Priest",
    applied: [
      tag("enforcer_npc", {
        source: "human",
        confidence: "high",
        rationale: "The encounter framing is adversarial and battle-ready, not only social or civic.",
      }),
      tag("profession_npc", {
        source: "human",
        confidence: "high",
        rationale: "Presented as a role-defined false cleric identity rather than an untyped combatant.",
      })
    ],
    excluded: [
      tag("civic_npc", {
        source: "human",
        confidence: "high",
        rationale: "Not primarily embedded as a helpful or neutral civic scene participant.",
      })
    ],
  },
});

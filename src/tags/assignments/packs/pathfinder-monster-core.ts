import { defineAssignments, tag } from "../builders.js";

export const PATHFINDER_MONSTER_CORE_DERIVED_TAG_ASSIGNMENTS = defineAssignments({
  "pathfinder-monster-core:TGYELuImcTcuX0aH": {
    name: "Conspirator Dragon (Adult)",
    applied: [
      tag("disguised_pretender", {
        source: "human",
        confidence: "high",
        rationale: "Disguise and concealed social identity are central to the creature's concept.",
      }),
      tag("urban_setting", {
        source: "human",
        confidence: "high",
        rationale: "The creature is framed around infiltrating sophisticated social and city-centered environments.",
      })
    ],
  },
});

import type { AuthoredDerivedTagAssignment } from "../runtime/derivation/assignments.js";
import { CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG } from "../canonical/projections/creature.js";

export const CREATURE_DERIVED_TAG_ASSIGNMENTS: AuthoredDerivedTagAssignment[] = [
  {
    name: "Black Whale Guard",
    recordKey: "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
    applied: [
      {
        projectionId: CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG["nautical_setting"]!.id,
        source: "human",
        confidence: "high",
        rationale: "Strongly associated with shipboard and harbor-side encounter spaces.",
      },
    ],
  },
  {
    name: "Conspirator Dragon (Adult)",
    recordKey: "pathfinder-monster-core:TGYELuImcTcuX0aH",
    applied: [
      {
        projectionId: CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG["disguised_pretender"]!.id,
        source: "human",
        confidence: "high",
        rationale: "Disguise and concealed social identity are central to the creature's concept.",
      },
      {
        projectionId: CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG["urban_setting"]!.id,
        source: "human",
        confidence: "high",
        rationale: "The creature is framed around infiltrating sophisticated social and city-centered environments.",
      },
    ],
  },
  {
    name: "Departmental Chair",
    recordKey: "pathfinder-npc-core:MxcprNbX7hcpAU8p",
    applied: [
      {
        projectionId: CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG["civic_npc"]!.id,
        source: "human",
        confidence: "high",
        rationale: "Fits the civic and social fabric of an urban institution-driven scene.",
      },
      {
        projectionId: CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG["profession_npc"]!.id,
        source: "human",
        confidence: "high",
        rationale: "Presented as a role-defined academic administrator rather than as a generic monster encounter.",
      },
    ],
  },
  {
    name: "False Priest",
    recordKey: "pathfinder-npc-core:OAxxUyACpMlX3q1X",
    applied: [
      {
        projectionId: CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG["enforcer_npc"]!.id,
        source: "human",
        confidence: "high",
        rationale: "The encounter framing is adversarial and battle-ready, not only social or civic.",
      },
      {
        projectionId: CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG["profession_npc"]!.id,
        source: "human",
        confidence: "high",
        rationale: "Presented as a role-defined false cleric identity rather than an untyped combatant.",
      },
    ],
    excluded: [
      {
        projectionId: CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG["civic_npc"]!.id,
        source: "human",
        confidence: "high",
        rationale: "Not primarily embedded as a helpful or neutral civic scene participant.",
      },
    ],
  },
  {
    name: "Spiritbound Aluum",
    recordKey: "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
    applied: [
      {
        projectionId: CREATURE_DERIVED_TAG_CANONICAL_PROJECTIONS_BY_TAG["urban_setting"]!.id,
        source: "human",
        confidence: "high",
        rationale: "Its encounter framing is tied to dense built environments rather than wilderness habitats.",
      },
    ],
  },
];

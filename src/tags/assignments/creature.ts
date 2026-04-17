import type { AuthoredDerivedTagAssignment } from "../runtime/assignments.js";

export const CREATURE_DERIVED_TAG_ASSIGNMENTS: AuthoredDerivedTagAssignment[] = [
  {
    name: "Black Whale Guard",
    recordKey: "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
    applied: {
      setting: [
        {
          tag: "nautical_setting",
          source: "human",
          confidence: "high",
          rationale: "Strongly associated with shipboard and harbor-side encounter spaces.",
        },
      ],
    },
  },
  {
    name: "Conspirator Dragon (Adult)",
    recordKey: "pathfinder-monster-core:TGYELuImcTcuX0aH",
    applied: {
      motif: [
        {
          tag: "disguised_pretender",
          source: "human",
          confidence: "high",
          rationale: "Disguise and concealed social identity are central to the creature's concept.",
        },
      ],
      setting: [
        {
          tag: "urban_setting",
          source: "human",
          confidence: "high",
          rationale: "The creature is framed around infiltrating sophisticated social and city-centered environments.",
        },
      ],
    },
  },
  {
    name: "Departmental Chair",
    recordKey: "pathfinder-npc-core:MxcprNbX7hcpAU8p",
    applied: {
      encounter_role: [
        {
          tag: "civic_npc",
          source: "human",
          confidence: "high",
          rationale: "Fits the civic and social fabric of an urban institution-driven scene.",
        },
        {
          tag: "profession_npc",
          source: "human",
          confidence: "high",
          rationale: "Presented as a role-defined academic administrator rather than as a generic monster encounter.",
        },
      ],
    },
  },
  {
    name: "False Priest",
    recordKey: "pathfinder-npc-core:OAxxUyACpMlX3q1X",
    applied: {
      encounter_role: [
        {
          tag: "combatant_npc",
          source: "human",
          confidence: "high",
          rationale: "The encounter framing is adversarial and battle-ready, not only social or civic.",
        },
        {
          tag: "profession_npc",
          source: "human",
          confidence: "high",
          rationale: "Presented as a role-defined false cleric identity rather than an untyped combatant.",
        },
      ],
    },
    excluded: {
      encounter_role: [
        {
          tag: "civic_npc",
          source: "human",
          confidence: "high",
          rationale: "Not primarily embedded as a helpful or neutral civic scene participant.",
        },
      ],
    },
  },
  {
    name: "Spiritbound Aluum",
    recordKey: "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
    applied: {
      setting: [
        {
          tag: "urban_setting",
          source: "human",
          confidence: "high",
          rationale: "Its encounter framing is tied to dense built environments rather than wilderness habitats.",
        },
      ],
    },
  },
];

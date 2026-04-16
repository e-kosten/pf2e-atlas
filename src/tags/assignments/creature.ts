import type { AuthoredDerivedTagAssignment } from "../runtime/assignments.js";

export const CREATURE_DERIVED_TAG_ASSIGNMENTS: AuthoredDerivedTagAssignment[] = [
  {
    name: "Departmental Chair",
    recordKey: "pathfinder-npc-core:MxcprNbX7hcpAU8p",
    applied: {
      encounter_role: ["profession_npc", "civic_npc"],
    },
    review: {
      setting: {
        urban_setting: {
          mode: "include",
          status: "needs_review",
          confidence: "low",
          source: "llm",
          rationale: "Temporary manual CLI test fixture: pending review item for queue and session testing.",
        },
      },
      encounter_role: {
        profession_npc: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Presented as a role-defined academic administrator rather than as a generic monster encounter.",
        },
        civic_npc: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Fits the civic and social fabric of an urban institution-driven scene.",
        },
      },
    },
  },
  {
    name: "False Priest",
    recordKey: "pathfinder-npc-core:OAxxUyACpMlX3q1X",
    applied: {
      encounter_role: ["profession_npc", "combatant_npc"],
    },
    excluded: {
      encounter_role: ["civic_npc"],
    },
    review: {
      setting: {
        urban_setting: {
          mode: "exclude",
          status: "needs_review",
          confidence: "medium",
          source: "llm",
          rationale: "Temporary manual CLI test fixture: pending exclusion review for queue and TUI testing.",
        },
      },
      encounter_role: {
        profession_npc: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Presented as a role-defined false cleric identity rather than an untyped combatant.",
        },
        combatant_npc: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "The encounter framing is adversarial and battle-ready, not only social or civic.",
        },
        civic_npc: {
          mode: "exclude",
          status: "approved",
          confidence: "high",
          rationale: "Not primarily embedded as a helpful or neutral civic scene participant.",
        },
      },
    },
  },
  {
    name: "Spiritbound Aluum",
    recordKey: "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
    applied: {
      setting: ["urban_setting"],
    },
    review: {
      setting: {
        urban_setting: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Its encounter framing is tied to dense built environments rather than wilderness habitats.",
        },
      },
    },
  },
  {
    name: "Black Whale Guard",
    recordKey: "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
    applied: {
      setting: ["nautical_setting"],
    },
    review: {
      setting: {
        coastal_setting: {
          mode: "include",
          status: "needs_review",
          confidence: "medium",
          source: "llm",
          rationale: "Temporary manual CLI test fixture: pending coastal-setting review for CLI testing.",
        },
        nautical_setting: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Strongly associated with shipboard and harbor-side encounter spaces.",
        },
      },
    },
  },
  {
    name: "Conspirator Dragon (Adult)",
    recordKey: "pathfinder-monster-core:TGYELuImcTcuX0aH",
    applied: {
      motif: ["disguised_pretender"],
      setting: ["urban_setting"],
    },
    review: {
      encounter_role: {
        combatant_npc: {
          mode: "include",
          status: "needs_review",
          confidence: "low",
          source: "llm",
          rationale: "Temporary manual CLI test fixture: pending role review to exercise mixed-family queue behavior.",
        },
      },
      motif: {
        disguised_pretender: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Disguise and concealed social identity are central to the creature's concept.",
        },
      },
      setting: {
        urban_setting: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "The creature is framed around infiltrating sophisticated social and city-centered environments.",
        },
      },
    },
  },
];

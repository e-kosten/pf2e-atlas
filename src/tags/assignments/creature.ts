import type { AuthoredDerivedTagAssignment } from "../runtime/assignments.js";

export const CREATURE_DERIVED_TAG_ASSIGNMENTS: AuthoredDerivedTagAssignment[] = [
  {
    name: "Black Whale Guard",
    recordKey: "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
    applied: {
      setting: [
        "nautical_setting"
      ]
    },
    review: {
      setting: {
        coastal_setting: {
          mode: "include",
          status: "rejected",
          rationale: "Temporary manual CLI test fixture: pending coastal-setting review for CLI testing.",
          confidence: "medium",
          source: "llm"
        },
        nautical_setting: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Strongly associated with shipboard and harbor-side encounter spaces."
        }
      }
    }
  },
  {
    name: "Conspirator Dragon (Adult)",
    recordKey: "pathfinder-monster-core:TGYELuImcTcuX0aH",
    applied: {
      motif: [
        "disguised_pretender"
      ],
      setting: [
        "urban_setting"
      ]
    },
    review: {
      encounter_role: {
        combatant_npc: {
          mode: "include",
          status: "rejected",
          rationale: "Temporary manual CLI test fixture: pending role review to exercise mixed-family queue behavior.",
          confidence: "low",
          source: "llm"
        }
      },
      motif: {
        disguised_pretender: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Disguise and concealed social identity are central to the creature's concept."
        }
      },
      setting: {
        urban_setting: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "The creature is framed around infiltrating sophisticated social and city-centered environments."
        }
      }
    }
  },
  {
    name: "Departmental Chair",
    recordKey: "pathfinder-npc-core:MxcprNbX7hcpAU8p",
    applied: {
      encounter_role: [
        "civic_npc",
        "profession_npc"
      ]
    },
    review: {
      encounter_role: {
        civic_npc: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Fits the civic and social fabric of an urban institution-driven scene."
        },
        profession_npc: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Presented as a role-defined academic administrator rather than as a generic monster encounter."
        }
      },
      setting: {
        urban_setting: {
          mode: "include",
          status: "rejected",
          rationale: "Temporary manual CLI test fixture: pending review item for queue and session testing.",
          confidence: "low",
          source: "llm"
        }
      }
    }
  },
  {
    name: "False Priest",
    recordKey: "pathfinder-npc-core:OAxxUyACpMlX3q1X",
    applied: {
      encounter_role: [
        "combatant_npc",
        "profession_npc"
      ]
    },
    excluded: {
      encounter_role: [
        "civic_npc"
      ],
      setting: [
        "urban_setting"
      ]
    },
    review: {
      encounter_role: {
        civic_npc: {
          mode: "exclude",
          status: "approved",
          confidence: "high",
          rationale: "Not primarily embedded as a helpful or neutral civic scene participant."
        },
        combatant_npc: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "The encounter framing is adversarial and battle-ready, not only social or civic."
        },
        profession_npc: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Presented as a role-defined false cleric identity rather than an untyped combatant."
        }
      },
      setting: {
        urban_setting: {
          mode: "exclude",
          status: "approved",
          rationale: "Temporary manual CLI test fixture: pending exclusion review for queue and TUI testing.",
          confidence: "medium",
          source: "llm"
        }
      }
    }
  },
  {
    name: "Spiritbound Aluum",
    recordKey: "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
    applied: {
      setting: [
        "urban_setting"
      ]
    },
    review: {
      setting: {
        urban_setting: {
          mode: "include",
          status: "approved",
          confidence: "high",
          rationale: "Its encounter framing is tied to dense built environments rather than wilderness habitats."
        }
      }
    }
  }
];

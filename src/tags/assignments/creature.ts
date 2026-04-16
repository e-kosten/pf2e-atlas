import type { AuthoredDerivedTagAssignment } from "../assignments.js";

export const CREATURE_DERIVED_TAG_ASSIGNMENTS: AuthoredDerivedTagAssignment[] = [
  {
    recordKey: "pathfinder-npc-core:MxcprNbX7hcpAU8p",
    name: "Departmental Chair",
    byFamily: {
      encounter_role: ["profession_npc", "civic_npc"],
    },
  },
  {
    recordKey: "pathfinder-npc-core:OAxxUyACpMlX3q1X",
    name: "False Priest",
    byFamily: {
      encounter_role: ["profession_npc", "combatant_npc"],
    },
    excludeByFamily: {
      encounter_role: ["civic_npc"],
    },
  },
  {
    recordKey: "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
    name: "Spiritbound Aluum",
    byFamily: {
      setting: ["urban_setting"],
    },
  },
  {
    recordKey: "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
    name: "Black Whale Guard",
    byFamily: {
      setting: ["nautical_setting"],
    },
  },
  {
    recordKey: "pathfinder-monster-core:TGYELuImcTcuX0aH",
    name: "Conspirator Dragon (Adult)",
    byFamily: {
      motif: ["disguised_pretender"],
      setting: ["urban_setting"],
    },
  },
];

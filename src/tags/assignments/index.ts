import type { AuthoredDerivedTagAssignment } from "../runtime/derivation/assignments.js";
import { AGE_OF_ASHES_BESTIARY_DERIVED_TAG_ASSIGNMENTS } from "./packs/age-of-ashes-bestiary.js";
import { AGENTS_OF_EDGEWATCH_BESTIARY_DERIVED_TAG_ASSIGNMENTS } from "./packs/agents-of-edgewatch-bestiary.js";
import { PATHFINDER_MONSTER_CORE_DERIVED_TAG_ASSIGNMENTS } from "./packs/pathfinder-monster-core.js";
import { PATHFINDER_NPC_CORE_DERIVED_TAG_ASSIGNMENTS } from "./packs/pathfinder-npc-core.js";

export const DERIVED_TAG_ASSIGNMENTS: AuthoredDerivedTagAssignment[] = [
  ...AGE_OF_ASHES_BESTIARY_DERIVED_TAG_ASSIGNMENTS,
  ...AGENTS_OF_EDGEWATCH_BESTIARY_DERIVED_TAG_ASSIGNMENTS,
  ...PATHFINDER_MONSTER_CORE_DERIVED_TAG_ASSIGNMENTS,
  ...PATHFINDER_NPC_CORE_DERIVED_TAG_ASSIGNMENTS,
];

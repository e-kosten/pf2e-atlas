import type { DerivedTagManagedCategory } from "../manifest.js";
import type { DerivedTagAssignmentMemoryCategory } from "../runtime/assignments.js";

export const DERIVED_TAG_ASSIGNMENT_MEMORY_BY_CATEGORY: Record<
  DerivedTagManagedCategory,
  DerivedTagAssignmentMemoryCategory
> = {
  affliction: {
    category: "affliction",
    decisions: [],
  },
  creature: {
    category: "creature",
    decisions: [],
  },
  equipment: {
    category: "equipment",
    decisions: [],
  },
  hazard: {
    category: "hazard",
    decisions: [],
  },
  spell: {
    category: "spell",
    decisions: [],
  },
};

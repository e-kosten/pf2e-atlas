import type { DerivedTagManagedCategory } from "../manifest.js";
import type { DerivedTagAssignmentReviewCategory } from "../runtime/assignments.js";

export const DERIVED_TAG_ASSIGNMENT_REVIEWS_BY_CATEGORY: Record<
  DerivedTagManagedCategory,
  DerivedTagAssignmentReviewCategory
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

import type { DerivedTagExemplarReviewCategory } from "../../../domain/derived-tag-types.js";
import type { DerivedTagManagedCategory } from "../../manifest.js";

export const DERIVED_TAG_EXEMPLAR_REVIEWS_BY_CATEGORY: Record<
  DerivedTagManagedCategory,
  DerivedTagExemplarReviewCategory
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

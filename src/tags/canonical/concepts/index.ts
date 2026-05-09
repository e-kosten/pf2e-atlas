import type { DerivedTagCanonicalConcept } from "../../../domain/derived-tag-types.js";
import { DERIVED_TAG_AGGREGATE_CANONICAL_CONCEPTS_BY_ID } from "./aggregate.js";
import { DERIVED_TAG_DESCRIPTIVE_CANONICAL_CONCEPTS_BY_ID } from "./descriptive/index.js";
import { DERIVED_TAG_OPERATIONAL_CANONICAL_CONCEPTS_BY_ID } from "./operational.js";

export const DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID: Record<string, DerivedTagCanonicalConcept> = {
  ...DERIVED_TAG_AGGREGATE_CANONICAL_CONCEPTS_BY_ID,
  ...DERIVED_TAG_DESCRIPTIVE_CANONICAL_CONCEPTS_BY_ID,
  ...DERIVED_TAG_OPERATIONAL_CANONICAL_CONCEPTS_BY_ID,
};

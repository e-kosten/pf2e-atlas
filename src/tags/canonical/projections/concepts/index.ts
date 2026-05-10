import { buildProjectionRecordsByCategory } from "../../builders.js";
import { DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID } from "../../concepts/index.js";
import { aggregateProjectionDeclarations } from "./aggregate.js";
import { descriptiveProjectionDeclarations } from "./descriptive.js";
import { operationalProjectionDeclarations } from "./operational.js";

export const DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY = buildProjectionRecordsByCategory(
  DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID,
  [
    ...aggregateProjectionDeclarations,
    ...descriptiveProjectionDeclarations,
    ...operationalProjectionDeclarations,
  ],
);

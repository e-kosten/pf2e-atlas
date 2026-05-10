import { buildProjectionRecordsByCategory } from "../../builders.js";
import { DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID } from "../../concepts/index.js";
import { aggregateProjectionDeclarations } from "./aggregate/index.js";
import { descriptiveProjectionDeclarations } from "./descriptive/index.js";
import { operationalProjectionDeclarations } from "./operational/index.js";

export const DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY = buildProjectionRecordsByCategory(
  DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID,
  [...aggregateProjectionDeclarations, ...descriptiveProjectionDeclarations, ...operationalProjectionDeclarations],
);

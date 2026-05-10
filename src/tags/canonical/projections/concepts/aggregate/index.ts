import { AggregateCapabilityProjectionDeclarations } from "./capability.js";
import { AggregateFunctionProjectionDeclarations } from "./function.js";
import { AggregateOperationalizedProjectionDeclarations } from "./operationalized.js";
import { AggregateSettingProjectionDeclarations } from "./setting.js";
import type { ConceptProjectionDeclaration } from "../../../builders.js";

export const aggregateProjectionDeclarations = [
  ...AggregateCapabilityProjectionDeclarations,
  ...AggregateFunctionProjectionDeclarations,
  ...AggregateOperationalizedProjectionDeclarations,
  ...AggregateSettingProjectionDeclarations,
] satisfies ConceptProjectionDeclaration[];

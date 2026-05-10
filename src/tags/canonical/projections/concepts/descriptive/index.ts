import { DescriptiveBehaviorOverrideProjectionDeclarations } from "./behavior_override.js";
import { DescriptiveCapabilityProjectionDeclarations } from "./capability.js";
import { DescriptiveChallengeStructureProjectionDeclarations } from "./challenge_structure.js";
import { DescriptiveCreatureFamilyProjectionDeclarations } from "./creature_family.js";
import { DescriptiveDeliveryProjectionDeclarations } from "./delivery.js";
import { DescriptiveEffectProjectionDeclarations } from "./effect.js";
import { DescriptiveFunctionProjectionDeclarations } from "./function.js";
import { DescriptiveMechanismProjectionDeclarations } from "./mechanism.js";
import { DescriptivePathogenesisProjectionDeclarations } from "./pathogenesis.js";
import { DescriptiveProgressionProjectionDeclarations } from "./progression.js";
import { DescriptiveResponseDemandProjectionDeclarations } from "./response_demand.js";
import { DescriptiveRoleProjectionDeclarations } from "./role.js";
import { DescriptiveSettingProjectionDeclarations } from "./setting.js";
import { DescriptiveThemeProjectionDeclarations } from "./theme.js";
import type { ConceptProjectionDeclaration } from "../../../builders.js";

export const descriptiveProjectionDeclarations = [
  ...DescriptiveBehaviorOverrideProjectionDeclarations,
  ...DescriptiveCapabilityProjectionDeclarations,
  ...DescriptiveChallengeStructureProjectionDeclarations,
  ...DescriptiveCreatureFamilyProjectionDeclarations,
  ...DescriptiveDeliveryProjectionDeclarations,
  ...DescriptiveEffectProjectionDeclarations,
  ...DescriptiveFunctionProjectionDeclarations,
  ...DescriptiveMechanismProjectionDeclarations,
  ...DescriptivePathogenesisProjectionDeclarations,
  ...DescriptiveProgressionProjectionDeclarations,
  ...DescriptiveResponseDemandProjectionDeclarations,
  ...DescriptiveRoleProjectionDeclarations,
  ...DescriptiveSettingProjectionDeclarations,
  ...DescriptiveThemeProjectionDeclarations,
] satisfies ConceptProjectionDeclaration[];

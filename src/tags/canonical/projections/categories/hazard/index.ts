import { hazardEffectProjectionFamilies } from "./effect.js";
import { hazardEncounterProjectionFamilies } from "./encounter.js";
import { hazardHauntProjectionFamilies } from "./haunt.js";
import { hazardMechanismProjectionFamilies } from "./mechanism.js";
import { hazardProblemProjectionFamilies } from "./problem.js";
import { hazardResolutionProjectionFamilies } from "./resolution.js";
import { hazardSettingProjectionFamilies } from "./setting.js";
import { defineCategoryProjections, type CategoryProjectionDeclaration } from "../../../builders.js";

export const hazardProjectionDeclarations = defineCategoryProjections("hazard", [
  ...hazardEffectProjectionFamilies,
  ...hazardEncounterProjectionFamilies,
  ...hazardHauntProjectionFamilies,
  ...hazardMechanismProjectionFamilies,
  ...hazardProblemProjectionFamilies,
  ...hazardResolutionProjectionFamilies,
  ...hazardSettingProjectionFamilies,
]) satisfies CategoryProjectionDeclaration<"hazard">;

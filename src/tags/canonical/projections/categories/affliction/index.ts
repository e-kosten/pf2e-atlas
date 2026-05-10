import { afflictionBehaviorProjectionFamilies } from "./behavior.js";
import { afflictionDiseaseModelProjectionFamilies } from "./disease_model.js";
import { afflictionEffectProjectionFamilies } from "./effect.js";
import { afflictionMetaphysicalProjectionFamilies } from "./metaphysical.js";
import { afflictionResponseProjectionFamilies } from "./response.js";
import { defineCategoryProjections, type CategoryProjectionDeclaration } from "../../../builders.js";

export const afflictionProjectionDeclarations = defineCategoryProjections("affliction", [
  ...afflictionBehaviorProjectionFamilies,
  ...afflictionDiseaseModelProjectionFamilies,
  ...afflictionEffectProjectionFamilies,
  ...afflictionMetaphysicalProjectionFamilies,
  ...afflictionResponseProjectionFamilies,
]) satisfies CategoryProjectionDeclaration<"affliction">;

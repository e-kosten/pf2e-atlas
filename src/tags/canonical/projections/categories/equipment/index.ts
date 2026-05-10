import { equipmentEffectProjectionFamilies } from "./effect.js";
import { equipmentItemMechanicalProjectionFamilies } from "./item_mechanical.js";
import { equipmentPartyRoleProjectionFamilies } from "./party_role.js";
import { equipmentUtilityProjectionFamilies } from "./utility.js";
import { defineCategoryProjections, type CategoryProjectionDeclaration } from "../../../builders.js";

export const equipmentProjectionDeclarations = defineCategoryProjections("equipment", [
  ...equipmentEffectProjectionFamilies,
  ...equipmentItemMechanicalProjectionFamilies,
  ...equipmentPartyRoleProjectionFamilies,
  ...equipmentUtilityProjectionFamilies,
]) satisfies CategoryProjectionDeclaration<"equipment">;

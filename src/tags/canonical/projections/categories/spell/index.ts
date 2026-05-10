import { spellBattlefieldProjectionFamilies } from "./battlefield.js";
import { spellEffectProjectionFamilies } from "./effect.js";
import { spellInfluenceProjectionFamilies } from "./influence.js";
import { spellSummoningProjectionFamilies } from "./summoning.js";
import { spellSupportProjectionFamilies } from "./support.js";
import { spellTransformationProjectionFamilies } from "./transformation.js";
import { spellUtilityProjectionFamilies } from "./utility.js";
import { defineCategoryProjections, type CategoryProjectionDeclaration } from "../../../builders.js";

export const spellProjectionDeclarations = defineCategoryProjections("spell", [
  ...spellBattlefieldProjectionFamilies,
  ...spellEffectProjectionFamilies,
  ...spellInfluenceProjectionFamilies,
  ...spellSummoningProjectionFamilies,
  ...spellSupportProjectionFamilies,
  ...spellTransformationProjectionFamilies,
  ...spellUtilityProjectionFamilies,
]) satisfies CategoryProjectionDeclaration<"spell">;

import { creatureEncounterProjectionFamilies } from "./encounter.js";
import { creatureNpcRoleProjectionFamilies } from "./npc_role.js";
import { creaturePresentationProjectionFamilies } from "./presentation.js";
import { creatureSettingProjectionFamilies } from "./setting.js";
import { creatureSpecializationProjectionFamilies } from "./specialization.js";
import { defineCategoryProjections, type CategoryProjectionDeclaration } from "../../../builders.js";

export const creatureProjectionDeclarations = defineCategoryProjections("creature", [
  ...creatureEncounterProjectionFamilies,
  ...creatureNpcRoleProjectionFamilies,
  ...creaturePresentationProjectionFamilies,
  ...creatureSettingProjectionFamilies,
  ...creatureSpecializationProjectionFamilies,
]) satisfies CategoryProjectionDeclaration<"creature">;

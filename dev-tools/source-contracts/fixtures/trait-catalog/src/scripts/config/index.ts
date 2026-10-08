import { equipmentTraits, weaponTraits, actionTraits, effectTraits, otherArmorTags, traitDescriptions } from "./traits.ts";
const PF2E = {
    equipmentTraits,
    weaponTraits,
    actionTraits,
    effectTraits,
    otherArmorTags,
    rarityTraits: { common: "PF2E.Common" },
    traitsDescriptions: traitDescriptions,
    unrelatedRuntime: unknownFoundryStartup(),
};
export { PF2E };

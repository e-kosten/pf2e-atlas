import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";

export const equipmentItemMechanicalProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.ITEM_MECHANICAL_ACCESS_SYSTEM, {
    ammo_management: {
      description: "Magazines or related gear that manage repeating-weapon ammunition or reload workflow.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "ammo", "armor", "weapon"],
    },
    extradimensional_storage: {
      description: "Provides bag-of-holding-style storage through extradimensional or magically expanded space.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "ammo", "armor", "weapon"],
    },
    weapon_staging: {
      description: "Holsters, sheaths, scabbards, or bandoliers that stage weapons for quick draw or organized carry.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "ammo", "armor", "weapon"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.ITEM_MECHANICAL_AMMUNITION_PAYLOAD, {
    creature_bane: {
      description: "Tailored ammunition for a selected creature type or trait.",
      subcategories: ["ammo"],
    },
    elemental_payload: {
      description: "Ammunition that delivers an elemental or reagent-based payload on impact.",
      subcategories: ["ammo"],
    },
    explosive_payload: {
      description: "Ammunition that detonates or scatters area damage on impact.",
      subcategories: ["ammo"],
    },
    spell_payload: {
      description: "Ammunition that delivers, casts, or imposes a spell effect on hit.",
      subcategories: ["ammo"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.ITEM_MECHANICAL_DEFENSE_PROFILE, {
    ally_cover: {
      description: "Provides cover or upgraded cover to nearby allies.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "shield"],
    },
    fall_protection: {
      description:
        "Reduces falling harm, cushions impact, or protects against vertical movement accidents and collapse.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "shield"],
    },
    hazard_shielding: {
      description: "Protects against environmental hazards, area effects, or other damaging exposures.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "shield"],
      appliesWhen: [
        "The item is naturally retrieved for surviving traps, breath weapons, alchemical blasts, or other hazardous exposure.",
        "Protection against scenes and effects matters more than only deflecting direct weapon attacks.",
      ],
      doesNotApplyWhen: [
        "The item's main value is cover against arrows or weapon strikes rather than wider hazard exposure.",
        "The item mainly protects against spells specifically, making magic_protection the stronger hook.",
      ],
      adjacentTags: ["projectile_defense", "magic_protection"],
    },
    projectile_defense: {
      description: "Intercepts, redirects, or absorbs ranged attacks and projectiles.",
      subcategories: ["gear", "backpack", "kit", "vehicle", "armor", "shield"],
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"equipment">[];

import type { SearchCategory } from "../../types.js";
import { normalizeDerivedTag } from "./shared.js";

const RAW_LEGACY_FAMILY_ALIASES: Partial<Record<SearchCategory, Record<string, string[]>>> = {
  creature: {
    setting: ["habitat_setting", "site_setting", "regional_setting", "named_locale_setting", "planar_setting"],
    encounter_role: ["scene_role", "social_role"],
    world_role: ["social_role"],
    motif: ["visual_motif", "genre_motif", "story_motif"],
  },
  equipment: {
    purpose: ["movement_traversal", "scouting_surveillance", "access_bypass", "logistics_restraint"],
  },
  hazard: {
    encounter_role: ["function"],
  },
};

export function getLegacyDerivedTagFamilyAliases(
  category: SearchCategory,
  family: string,
): string[] {
  const normalizedFamily = normalizeDerivedTag(family);
  return (RAW_LEGACY_FAMILY_ALIASES[category]?.[normalizedFamily] ?? [])
    .map((candidate) => normalizeDerivedTag(candidate));
}

export function listLegacyDerivedTagFamilyAliases(
  category: SearchCategory,
): Array<{ legacyFamily: string; targetFamilies: string[] }> {
  return Object.entries(RAW_LEGACY_FAMILY_ALIASES[category] ?? {})
    .map(([legacyFamily, targetFamilies]) => ({
      legacyFamily: normalizeDerivedTag(legacyFamily),
      targetFamilies: targetFamilies.map((candidate) => normalizeDerivedTag(candidate)),
    }));
}

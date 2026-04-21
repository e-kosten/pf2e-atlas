import type { SearchCategory, SearchSubcategory } from "../domain/search-types.js";
import { normalizeSearchSubcategory } from "../domain/categories.js";
import { normalizeText, uniqueSorted } from "../shared/utils.js";
import { firstString, getNested, toStringArray } from "./raw-utils.js";

const FEAT_SYSTEM_SUBCATEGORIES = new Set<SearchSubcategory>(["class", "ancestry", "skill", "general"]);

function isExcludedPublicCategoryRecord(documentType: string, recordType: string): boolean {
  const normalizedDocumentType = normalizeText(documentType);
  const normalizedRecordType = normalizeText(recordType);
  return (
    normalizedDocumentType === "macro" || normalizedDocumentType === "rolltable" || normalizedRecordType === "script"
  );
}

function inferHazardSubcategory(traits: string[]): SearchSubcategory | null {
  const normalizedTraits = new Set(traits.map((trait) => normalizeText(trait)).filter(Boolean));
  if (normalizedTraits.has("haunt")) {
    return "haunt";
  }
  if (normalizedTraits.has("trap")) {
    return "trap";
  }
  return null;
}

function inferAfflictionSubcategory(traits: string[]): SearchSubcategory | null {
  const normalizedTraits = new Set(traits.map((trait) => normalizeText(trait)).filter(Boolean));
  if (normalizedTraits.has("curse")) {
    return "curse";
  }
  if (normalizedTraits.has("disease")) {
    return "disease";
  }
  if (normalizedTraits.has("poison")) {
    return "poison";
  }
  return null;
}

function inferFeatSubcategories(
  packName: string,
  sourcePath: string,
  traits: string[],
  raw: Record<string, unknown>,
): SearchSubcategory | null {
  const normalizedPackName = normalizeText(packName);
  const lowerSourcePath = sourcePath.replace(/\\/g, "/").toLowerCase();
  const normalizedTraits = new Set(traits.map((trait) => normalizeText(trait)).filter(Boolean));
  const systemCategory = normalizeText(firstString(getNested(raw, ["system", "category"])) ?? "");

  if (normalizedPackName === "boons and curses" || lowerSourcePath.includes("/boons-and-curses/")) {
    return "boonCurse";
  }

  if (normalizedTraits.has("archetype") || lowerSourcePath.includes("/feats/archetype/")) {
    return "archetype";
  }

  const featSystemSubcategory = normalizeSearchSubcategory(systemCategory);
  if (featSystemSubcategory && FEAT_SYSTEM_SUBCATEGORIES.has(featSystemSubcategory)) {
    return featSystemSubcategory;
  }

  if (systemCategory === "deityboon" || systemCategory === "curse") {
    return "boonCurse";
  }

  return null;
}

export function classifyRecordCategory(input: {
  documentType: string;
  recordType: string;
  packName: string;
  sourcePath: string;
  traits: string[];
  traditions: string[];
  raw: Record<string, unknown>;
}): { category: SearchCategory; subcategory: SearchSubcategory | null } | null {
  const documentType = normalizeText(input.documentType);
  const recordType = normalizeText(input.recordType);

  if (isExcludedPublicCategoryRecord(documentType, recordType)) {
    return null;
  }

  switch (recordType) {
    case "npc":
      return { category: "creature", subcategory: null };
    case "character":
      return { category: "creature", subcategory: "character" };
    case "familiar":
      return { category: "creature", subcategory: "familiar" };
    case "hazard":
      return { category: "hazard", subcategory: inferHazardSubcategory(input.traits) };
    case "affliction":
      return { category: "affliction", subcategory: inferAfflictionSubcategory(input.traits) };
    case "curse":
      return { category: "affliction", subcategory: "curse" };
    case "disease":
      return { category: "affliction", subcategory: "disease" };
    case "poison":
      return { category: "affliction", subcategory: "poison" };
    case "spell":
      return { category: "spell", subcategory: null };
    case "feat":
      return {
        category: "feat",
        subcategory: inferFeatSubcategories(input.packName, input.sourcePath, input.traits, input.raw),
      };
    case "action":
      return { category: "rule", subcategory: "action" };
    case "condition":
      return { category: "rule", subcategory: "condition" };
    case "effect":
      return { category: "rule", subcategory: "effect" };
    case "campaignfeature":
      return { category: "rule", subcategory: "campaignFeature" };
    case "deity":
      return { category: "lore", subcategory: "deity" };
    case "ancestry":
      return { category: "characterCreation", subcategory: "ancestry" };
    case "heritage":
      return { category: "characterCreation", subcategory: "heritage" };
    case "background":
      return { category: "characterCreation", subcategory: "background" };
    case "class":
      return { category: "characterCreation", subcategory: "class" };
    case "consumable":
      return { category: "equipment", subcategory: "consumable" };
    case "equipment":
      return { category: "equipment", subcategory: "gear" };
    case "weapon":
      return { category: "equipment", subcategory: "weapon" };
    case "armor":
      return { category: "equipment", subcategory: "armor" };
    case "shield":
      return { category: "equipment", subcategory: "shield" };
    case "ammo":
      return { category: "equipment", subcategory: "ammo" };
    case "backpack":
      return { category: "equipment", subcategory: "backpack" };
    case "treasure":
      return { category: "equipment", subcategory: "treasure" };
    case "kit":
      return { category: "equipment", subcategory: "kit" };
    case "vehicle":
      return { category: "equipment", subcategory: "vehicle" };
    case "unknown":
      if (documentType === "journalentry") {
        return { category: "lore", subcategory: "journal" };
      }
      return null;
    default:
      return null;
  }
}

export function extractSpellTraditions(raw: Record<string, unknown>): string[] {
  return uniqueSorted(
    toStringArray(getNested(raw, ["system", "traits", "traditions"]))
      .map((value) => normalizeText(value))
      .filter(Boolean),
  );
}

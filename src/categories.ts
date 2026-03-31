import { SearchCategory } from "./types.js";
import { firstString, getNested, normalizeText, toStringArray, uniqueSorted } from "./utils.js";

export const SEARCH_CATEGORIES = [
  "equipment",
  "feats",
  "creatures",
  "hazards",
  "afflictions",
  "rules",
  "spells",
  "characterCreation",
  "lore",
] as const satisfies readonly SearchCategory[];

export const CATEGORY_SUBCATEGORY_MAP: Record<SearchCategory, string[]> = {
  equipment: ["consumable", "gear", "weapon", "armor", "shield", "ammo", "backpack", "treasure", "kit", "vehicle"],
  feats: ["class", "ancestry", "skill", "general", "archetype", "boonCurse"],
  creatures: ["character", "familiar"],
  hazards: ["haunt", "trap"],
  afflictions: ["curse", "disease", "poison"],
  rules: ["action", "condition", "effect", "campaign"],
  spells: ["arcane", "divine", "occult", "primal", "focus", "ritual", "cantrip"],
  characterCreation: ["ancestry", "heritage", "background", "class"],
  lore: ["deity", "journal"],
};

const SUBCATEGORY_TO_CATEGORY = new Map<string, SearchCategory>(
  Object.entries(CATEGORY_SUBCATEGORY_MAP)
    .flatMap(([category, subcategories]) => subcategories.map((subcategory) => [normalizeText(subcategory), category as SearchCategory] as const)),
);

export function getCategoryForSubcategory(subcategory: string | null | undefined): SearchCategory | null {
  const normalized = normalizeText(subcategory ?? "");
  return SUBCATEGORY_TO_CATEGORY.get(normalized) ?? null;
}

function isExcludedPublicCategoryRecord(documentType: string, recordType: string): boolean {
  const normalizedDocumentType = normalizeText(documentType);
  const normalizedRecordType = normalizeText(recordType);
  return normalizedDocumentType === "macro" ||
    normalizedDocumentType === "rolltable" ||
    normalizedRecordType === "script";
}

function inferHazardSubcategory(traits: string[]): string[] {
  const normalizedTraits = new Set(traits.map((trait) => normalizeText(trait)).filter(Boolean));
  if (normalizedTraits.has("haunt")) {
    return ["haunt"];
  }
  if (normalizedTraits.has("trap")) {
    return ["trap"];
  }
  return [];
}

function inferSpellSubcategories(traits: string[], traditions: string[]): string[] {
  const normalizedTraits = new Set(traits.map((trait) => normalizeText(trait)).filter(Boolean));
  const normalizedTraditions = traditions.map((tradition) => normalizeText(tradition)).filter(Boolean);
  const subcategories = [...normalizedTraditions];

  for (const candidate of ["focus", "ritual", "cantrip"]) {
    if (normalizedTraits.has(candidate)) {
      subcategories.push(candidate);
    }
  }

  return uniqueSorted(subcategories);
}

function inferFeatSubcategories(
  packName: string,
  sourcePath: string,
  traits: string[],
  raw: Record<string, unknown>,
): string[] {
  const normalizedPackName = normalizeText(packName);
  const lowerSourcePath = sourcePath.replace(/\\/g, "/").toLowerCase();
  const normalizedTraits = new Set(traits.map((trait) => normalizeText(trait)).filter(Boolean));
  const systemCategory = normalizeText(firstString(getNested(raw, ["system", "category"])) ?? "");

  if (normalizedPackName === "boons and curses" || lowerSourcePath.includes("/boons-and-curses/")) {
    return ["boonCurse"];
  }

  if (normalizedTraits.has("archetype") || lowerSourcePath.includes("/feats/archetype/")) {
    return ["archetype"];
  }

  if (["class", "ancestry", "skill", "general"].includes(systemCategory)) {
    return [systemCategory];
  }

  if (systemCategory === "deityboon" || systemCategory === "curse") {
    return ["boonCurse"];
  }

  return [];
}

export function classifyRecordCategory(input: {
  documentType: string;
  recordType: string;
  packName: string;
  sourcePath: string;
  traits: string[];
  traditions: string[];
  raw: Record<string, unknown>;
}): { category: SearchCategory; subcategories: string[] } | null {
  const documentType = normalizeText(input.documentType);
  const recordType = normalizeText(input.recordType);

  if (isExcludedPublicCategoryRecord(documentType, recordType)) {
    return null;
  }

  switch (recordType) {
    case "npc":
      return { category: "creatures", subcategories: [] };
    case "character":
      return { category: "creatures", subcategories: ["character"] };
    case "familiar":
      return { category: "creatures", subcategories: ["familiar"] };
    case "hazard":
      return { category: "hazards", subcategories: inferHazardSubcategory(input.traits) };
    case "spell":
      return { category: "spells", subcategories: inferSpellSubcategories(input.traits, input.traditions) };
    case "feat":
      return { category: "feats", subcategories: inferFeatSubcategories(input.packName, input.sourcePath, input.traits, input.raw) };
    case "action":
      return { category: "rules", subcategories: ["action"] };
    case "condition":
      return { category: "rules", subcategories: ["condition"] };
    case "effect":
      return { category: "rules", subcategories: ["effect"] };
    case "campaignfeature":
      return { category: "rules", subcategories: ["campaign"] };
    case "deity":
      return { category: "lore", subcategories: ["deity"] };
    case "ancestry":
      return { category: "characterCreation", subcategories: ["ancestry"] };
    case "heritage":
      return { category: "characterCreation", subcategories: ["heritage"] };
    case "background":
      return { category: "characterCreation", subcategories: ["background"] };
    case "class":
      return { category: "characterCreation", subcategories: ["class"] };
    case "consumable":
      return { category: "equipment", subcategories: ["consumable"] };
    case "equipment":
      return { category: "equipment", subcategories: ["gear"] };
    case "weapon":
      return { category: "equipment", subcategories: ["weapon"] };
    case "armor":
      return { category: "equipment", subcategories: ["armor"] };
    case "shield":
      return { category: "equipment", subcategories: ["shield"] };
    case "ammo":
      return { category: "equipment", subcategories: ["ammo"] };
    case "backpack":
      return { category: "equipment", subcategories: ["backpack"] };
    case "treasure":
      return { category: "equipment", subcategories: ["treasure"] };
    case "kit":
      return { category: "equipment", subcategories: ["kit"] };
    case "vehicle":
      return { category: "equipment", subcategories: ["vehicle"] };
    case "unknown":
      if (documentType === "journalentry") {
        return { category: "lore", subcategories: ["journal"] };
      }
      return null;
    default:
      return null;
  }
}

export function getCategoryKeywordAnchors(): Record<SearchCategory, string[]> {
  return {
    creatures: ["creature", "creatures", "monster", "monsters", "beast", "undead", "swarm", "dragon", "fiend", "enemy"],
    hazards: ["hazard", "hazards", "haunt", "haunted", "trap", "traps", "snare", "snares"],
    spells: ["spell", "spells", "cantrip", "cantrips", "ritual", "rituals", "focus", "arcane", "divine", "occult", "primal", "magic", "magical"],
    equipment: ["gear", "equipment", "item", "items", "weapon", "weapons", "armor", "shield", "consumable", "consumables", "potion", "potions", "elixir", "elixirs", "bomb", "bombs", "gadget", "gadgets", "ammo", "treasure"],
    feats: ["feat", "feats", "dedication", "archetype", "general feat", "skill feat", "class feat", "ancestry feat", "boon", "boons", "curse feat"],
    afflictions: ["affliction", "afflictions", "disease", "diseases", "poison", "poisons", "venom", "venoms", "toxin", "toxins", "plague", "plagues", "curse", "curses"],
    rules: ["rule", "rules", "action", "actions", "condition", "conditions", "effect", "effects", "trait", "traits", "kingdom", "warfare", "army", "subsystem"],
    characterCreation: ["character creation", "ancestry", "heritage", "background", "backgrounds", "class", "classes"],
    lore: ["lore", "setting", "deity", "deities", "god", "gods", "pantheon", "faith", "religion", "journal"],
  };
}

export function getSubcategoryKeywordAnchors(): Array<{ subcategory: string; category: SearchCategory; keywords: string[]; weight?: number }> {
  return [
    { category: "hazards", subcategory: "haunt", keywords: ["haunt", "haunted", "manifestation", "poltergeist"] },
    { category: "hazards", subcategory: "trap", keywords: ["trap", "traps", "snare", "snares", "tripwire", "pitfall"] },
    { category: "equipment", subcategory: "consumable", keywords: ["consumable", "consumables", "potion", "potions", "elixir", "elixirs", "bomb", "bombs", "fulu", "mutagen", "mutagens"] },
    { category: "equipment", subcategory: "weapon", keywords: ["weapon", "weapons"] },
    { category: "equipment", subcategory: "armor", keywords: ["armor"] },
    { category: "equipment", subcategory: "shield", keywords: ["shield", "shields"] },
    { category: "equipment", subcategory: "ammo", keywords: ["ammo", "ammunition", "arrow", "arrows", "bolt", "bolts"] },
    { category: "equipment", subcategory: "treasure", keywords: ["treasure", "loot", "hoard"] },
    { category: "spells", subcategory: "arcane", keywords: ["arcane"] },
    { category: "spells", subcategory: "divine", keywords: ["divine"] },
    { category: "spells", subcategory: "occult", keywords: ["occult"] },
    { category: "spells", subcategory: "primal", keywords: ["primal"] },
    { category: "spells", subcategory: "focus", keywords: ["focus"] },
    { category: "spells", subcategory: "ritual", keywords: ["ritual", "rituals"] },
    { category: "spells", subcategory: "cantrip", keywords: ["cantrip", "cantrips"] },
    { category: "feats", subcategory: "class", keywords: ["class feat", "class feats"], weight: 2 },
    { category: "feats", subcategory: "ancestry", keywords: ["ancestry feat", "ancestry feats"], weight: 2 },
    { category: "feats", subcategory: "skill", keywords: ["skill feat", "skill feats"], weight: 2 },
    { category: "feats", subcategory: "general", keywords: ["general feat", "general feats"], weight: 2 },
    { category: "feats", subcategory: "archetype", keywords: ["archetype", "dedication", "archetype feat", "archetype feats"] },
    { category: "feats", subcategory: "boonCurse", keywords: ["boon", "boons", "curse feat", "curse feats"] },
    { category: "rules", subcategory: "action", keywords: ["action", "actions"] },
    { category: "rules", subcategory: "condition", keywords: ["condition", "conditions"] },
    { category: "rules", subcategory: "effect", keywords: ["effect", "effects"] },
    { category: "rules", subcategory: "campaign", keywords: ["kingdom", "warfare", "army", "settlement", "hex", "campaign"], weight: 3 },
    { category: "characterCreation", subcategory: "ancestry", keywords: ["ancestry"] },
    { category: "characterCreation", subcategory: "heritage", keywords: ["heritage"] },
    { category: "characterCreation", subcategory: "background", keywords: ["background", "backgrounds"] },
    { category: "characterCreation", subcategory: "class", keywords: ["class", "classes"] },
    { category: "lore", subcategory: "deity", keywords: ["deity", "deities", "god", "gods", "pantheon", "faith", "religion"] },
    { category: "lore", subcategory: "journal", keywords: ["journal", "history", "setting", "lore"] },
    { category: "afflictions", subcategory: "disease", keywords: ["disease", "diseases", "plague", "plagues"] },
    { category: "afflictions", subcategory: "poison", keywords: ["poison", "poisons", "venom", "venoms", "toxin", "toxins"] },
    { category: "afflictions", subcategory: "curse", keywords: ["curse", "curses"] },
  ];
}

export function extractSpellTraditions(raw: Record<string, unknown>): string[] {
  return uniqueSorted(toStringArray(getNested(raw, ["system", "traits", "traditions"])).map((value) => normalizeText(value)).filter(Boolean));
}

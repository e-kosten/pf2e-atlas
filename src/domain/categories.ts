import type { SearchCategory, SearchSubcategory } from "./search-types.js";
import { normalizeText, uniqueSorted } from "../shared/utils.js";

export const SEARCH_CATEGORIES = [
  "equipment",
  "feat",
  "creature",
  "hazard",
  "affliction",
  "rule",
  "spell",
  "characterCreation",
  "lore",
] as const satisfies readonly SearchCategory[];

export const CATEGORY_SUBCATEGORY_MAP: Record<SearchCategory, SearchSubcategory[]> = {
  equipment: ["consumable", "gear", "weapon", "armor", "shield", "ammo", "backpack", "treasure", "kit", "vehicle"],
  feat: ["class", "ancestry", "skill", "general", "archetype", "boonCurse"],
  creature: ["character", "familiar"],
  hazard: ["haunt", "trap"],
  affliction: ["curse", "disease", "poison"],
  rule: ["action", "condition", "effect", "campaignFeature"],
  spell: [],
  characterCreation: ["ancestry", "heritage", "background", "class"],
  lore: ["deity", "journal"],
};

export const SEARCH_SUBCATEGORIES = uniqueSorted(Object.values(CATEGORY_SUBCATEGORY_MAP).flat());

const CATEGORY_INPUT_ALIASES = new Map<string, SearchCategory>([
  ...SEARCH_CATEGORIES.map((category) => [normalizeText(category), category] as const),
  ["feats", "feat"],
  ["creatures", "creature"],
  ["hazards", "hazard"],
  ["afflictions", "affliction"],
  ["rules", "rule"],
  ["spells", "spell"],
  ["character creation", "characterCreation"],
]);

const SUBCATEGORY_INPUT_ALIASES = new Map<string, SearchSubcategory>([
  ...SEARCH_SUBCATEGORIES.map((subcategory) => [normalizeText(subcategory), subcategory] as const),
  ["actions", "action"],
  ["conditions", "condition"],
  ["effects", "effect"],
  ["campaign", "campaignFeature"],
  ["campaigns", "campaignFeature"],
  ["campaign feature", "campaignFeature"],
  ["campaign features", "campaignFeature"],
  ["consumables", "consumable"],
  ["weapons", "weapon"],
  ["shields", "shield"],
  ["backpacks", "backpack"],
  ["treasures", "treasure"],
  ["kits", "kit"],
  ["vehicles", "vehicle"],
  ["archetypes", "archetype"],
  ["haunts", "haunt"],
  ["traps", "trap"],
  ["curses", "curse"],
  ["diseases", "disease"],
  ["poisons", "poison"],
  ["classes", "class"],
  ["ancestries", "ancestry"],
  ["heritages", "heritage"],
  ["backgrounds", "background"],
  ["deities", "deity"],
  ["journals", "journal"],
]);

const SUBCATEGORY_TO_CATEGORIES = new Map<string, SearchCategory[]>(
  SEARCH_SUBCATEGORIES.map((subcategory) => {
    const categories = SEARCH_CATEGORIES.filter((category) => CATEGORY_SUBCATEGORY_MAP[category].includes(subcategory));
    return [normalizeText(subcategory), categories] as const;
  }),
);

export const VALID_SEARCH_CATEGORY_LIST = SEARCH_CATEGORIES.join(", ");
export const VALID_SEARCH_SUBCATEGORY_LIST = SEARCH_SUBCATEGORIES.join(", ");

export function normalizeSearchCategory(value: string | null | undefined): SearchCategory | null {
  const normalized = normalizeText(value ?? "");
  return CATEGORY_INPUT_ALIASES.get(normalized) ?? null;
}

export function normalizeSearchSubcategory(value: string | null | undefined): SearchSubcategory | null {
  const normalized = normalizeText(value ?? "");
  return SUBCATEGORY_INPUT_ALIASES.get(normalized) ?? null;
}

export function getSearchCategoryErrorMessage(value: string): string {
  return `Unknown top-level category "${value}". Valid categories: ${VALID_SEARCH_CATEGORY_LIST}.`;
}

export function getSearchSubcategoryErrorMessage(value: string): string {
  return `Unknown subcategory "${value}". Valid subcategories: ${VALID_SEARCH_SUBCATEGORY_LIST}.`;
}

export function categorySupportsSubcategory(category: SearchCategory, subcategory: SearchSubcategory): boolean {
  return CATEGORY_SUBCATEGORY_MAP[category].includes(subcategory);
}

export function getCategoryForSubcategory(subcategory: string | null | undefined): SearchCategory | null {
  const canonicalSubcategory = normalizeSearchSubcategory(subcategory);
  if (!canonicalSubcategory) {
    return null;
  }

  const categories = SUBCATEGORY_TO_CATEGORIES.get(normalizeText(canonicalSubcategory)) ?? [];
  return categories.length === 1 ? categories[0]! : null;
}

export function getCategoryKeywordAnchors(): Record<SearchCategory, string[]> {
  return {
    creature: ["creature", "creatures", "monster", "monsters", "beast", "undead", "swarm", "dragon", "fiend", "enemy"],
    hazard: ["hazard", "hazards", "haunt", "haunted", "trap", "traps", "snare", "snares"],
    spell: [
      "spell",
      "spells",
      "cantrip",
      "cantrips",
      "ritual",
      "rituals",
      "focus",
      "arcane",
      "divine",
      "occult",
      "primal",
      "magic",
      "magical",
    ],
    equipment: [
      "gear",
      "equipment",
      "item",
      "items",
      "weapon",
      "weapons",
      "armor",
      "shield",
      "consumable",
      "consumables",
      "potion",
      "potions",
      "elixir",
      "elixirs",
      "bomb",
      "bombs",
      "gadget",
      "gadgets",
      "ammo",
      "treasure",
    ],
    feat: [
      "feat",
      "feats",
      "dedication",
      "archetype",
      "general feat",
      "skill feat",
      "class feat",
      "ancestry feat",
      "boon",
      "boons",
      "curse feat",
    ],
    affliction: [
      "affliction",
      "afflictions",
      "disease",
      "diseases",
      "poison",
      "poisons",
      "venom",
      "venoms",
      "toxin",
      "toxins",
      "plague",
      "plagues",
      "curse",
      "curses",
    ],
    rule: [
      "rule",
      "rules",
      "action",
      "actions",
      "condition",
      "conditions",
      "effect",
      "effects",
      "trait",
      "traits",
      "kingdom",
      "warfare",
      "army",
      "subsystem",
    ],
    characterCreation: ["character creation", "ancestry", "heritage", "background", "backgrounds", "class", "classes"],
    lore: ["lore", "setting", "deity", "deities", "god", "gods", "pantheon", "faith", "religion", "journal"],
  };
}

export function getSubcategoryKeywordAnchors(): Array<{
  subcategory: string;
  category: SearchCategory;
  keywords: string[];
  weight?: number;
}> {
  return [
    { category: "hazard", subcategory: "haunt", keywords: ["haunt", "haunted", "manifestation", "poltergeist"] },
    { category: "hazard", subcategory: "trap", keywords: ["trap", "traps", "snare", "snares", "tripwire", "pitfall"] },
    {
      category: "equipment",
      subcategory: "consumable",
      keywords: [
        "consumable",
        "consumables",
        "potion",
        "potions",
        "elixir",
        "elixirs",
        "bomb",
        "bombs",
        "fulu",
        "mutagen",
        "mutagens",
      ],
    },
    { category: "equipment", subcategory: "weapon", keywords: ["weapon", "weapons"] },
    { category: "equipment", subcategory: "armor", keywords: ["armor"] },
    { category: "equipment", subcategory: "shield", keywords: ["shield", "shields"] },
    {
      category: "equipment",
      subcategory: "ammo",
      keywords: ["ammo", "ammunition", "arrow", "arrows", "bolt", "bolts"],
    },
    { category: "equipment", subcategory: "treasure", keywords: ["treasure", "loot", "hoard"] },
    { category: "feat", subcategory: "class", keywords: ["class feat", "class feats"], weight: 2 },
    { category: "feat", subcategory: "ancestry", keywords: ["ancestry feat", "ancestry feats"], weight: 2 },
    { category: "feat", subcategory: "skill", keywords: ["skill feat", "skill feats"], weight: 2 },
    { category: "feat", subcategory: "general", keywords: ["general feat", "general feats"], weight: 2 },
    {
      category: "feat",
      subcategory: "archetype",
      keywords: ["archetype", "dedication", "archetype feat", "archetype feats"],
    },
    { category: "feat", subcategory: "boonCurse", keywords: ["boon", "boons", "curse feat", "curse feats"] },
    { category: "rule", subcategory: "action", keywords: ["action", "actions"] },
    { category: "rule", subcategory: "condition", keywords: ["condition", "conditions"] },
    { category: "rule", subcategory: "effect", keywords: ["effect", "effects"] },
    {
      category: "rule",
      subcategory: "campaignFeature",
      keywords: [
        "kingdom",
        "warfare",
        "army",
        "settlement",
        "hex",
        "campaign",
        "campaign feature",
        "campaign features",
      ],
      weight: 3,
    },
    { category: "characterCreation", subcategory: "ancestry", keywords: ["ancestry"] },
    { category: "characterCreation", subcategory: "heritage", keywords: ["heritage"] },
    { category: "characterCreation", subcategory: "background", keywords: ["background", "backgrounds"] },
    { category: "characterCreation", subcategory: "class", keywords: ["class", "classes"] },
    {
      category: "lore",
      subcategory: "deity",
      keywords: ["deity", "deities", "god", "gods", "pantheon", "faith", "religion"],
    },
    { category: "lore", subcategory: "journal", keywords: ["journal", "history", "setting", "lore"] },
    { category: "affliction", subcategory: "disease", keywords: ["disease", "diseases", "plague", "plagues"] },
    {
      category: "affliction",
      subcategory: "poison",
      keywords: ["poison", "poisons", "venom", "venoms", "toxin", "toxins"],
    },
    { category: "affliction", subcategory: "curse", keywords: ["curse", "curses"] },
  ];
}

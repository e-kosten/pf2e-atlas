import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import { uniqueSorted } from "../../shared/utils.js";

const AON_SEARCH_URL = "https://2e.aonprd.com/Search.aspx";

export type AonSearchRecordLike = {
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  rarity: string | null;
  traits: string[];
  traditions: string[];
  actionCost: number | null;
};

export type AonSearchLink = {
  url: string;
  label: string;
  plainTextFallback: string;
  appliedFilters: {
    query: string;
    includeTypes: string[];
    includeTraits: string[];
    includeRarities: string[];
    includeTraditions: string[];
    includeActions: string[];
  };
};

function normalizeFacetValues(values: readonly string[]): string[] {
  return uniqueSorted(
    values.map((value) => value.trim().toLowerCase()).filter((value): value is string => value.length > 0),
  );
}

const AON_INCLUDE_TYPES_BY_CATEGORY: Record<SearchCategory, string[]> = {
  creature: ["creature"],
  spell: ["spell", "cantrip", "ritual"],
  feat: ["feat"],
  hazard: ["hazard"],
  equipment: ["item"],
  affliction: ["disease", "curse", "poison"],
  characterCreation: [],
  lore: [],
  rule: [],
};
const AON_RULE_INCLUDE_TYPES_BY_SUBCATEGORY: Record<string, string[]> = {
  action: ["action"],
  condition: ["condition"],
};
const AON_CHARACTER_CREATION_SUBCATEGORIES = new Set(["ancestry", "background", "class", "heritage", "deity"]);

function getAonIncludeTypes(record: Pick<AonSearchRecordLike, "category" | "subcategory">): string[] {
  if (record.category === "rule" && record.subcategory) {
    return AON_RULE_INCLUDE_TYPES_BY_SUBCATEGORY[record.subcategory] ?? [];
  }

  if (record.category === "characterCreation" && record.subcategory) {
    return AON_CHARACTER_CREATION_SUBCATEGORIES.has(record.subcategory) ? [record.subcategory] : [];
  }

  return AON_INCLUDE_TYPES_BY_CATEGORY[record.category];
}

function getAonIncludeRarities(rarity: string | null): string[] {
  const normalized = rarity?.trim().toLowerCase();
  return normalized === "uncommon" || normalized === "rare" || normalized === "unique" ? [normalized] : [];
}

function getAonIncludeTraditions(record: Pick<AonSearchRecordLike, "category" | "traditions">): string[] {
  return record.category === "spell" ? normalizeFacetValues(record.traditions) : [];
}

function getAonIncludeActions(record: Pick<AonSearchRecordLike, "category" | "subcategory" | "actionCost">): string[] {
  if (
    record.actionCost == null ||
    (record.category !== "spell" && !(record.category === "rule" && record.subcategory === "action"))
  ) {
    return [];
  }

  return record.actionCost >= 1 && record.actionCost <= 3 ? [String(record.actionCost)] : [];
}

function setJoinedParam(params: URLSearchParams, key: string, values: string[]): void {
  if (values.length > 0) {
    params.set(key, values.join(" "));
  }
}

export function buildAonSearchLink(record: AonSearchRecordLike): AonSearchLink | null {
  const query = record.name.trim();
  if (query.length === 0) {
    return null;
  }

  const appliedFilters = {
    query,
    includeTypes: getAonIncludeTypes(record),
    includeTraits: normalizeFacetValues(record.traits),
    includeRarities: getAonIncludeRarities(record.rarity),
    includeTraditions: getAonIncludeTraditions(record),
    includeActions: getAonIncludeActions(record),
  };

  const url = new URL(AON_SEARCH_URL);
  url.searchParams.set("display", "short");
  url.searchParams.set("type", "eqs");
  url.searchParams.set("q", appliedFilters.query);
  setJoinedParam(url.searchParams, "include-types", appliedFilters.includeTypes);
  setJoinedParam(url.searchParams, "include-traits", appliedFilters.includeTraits);
  setJoinedParam(url.searchParams, "include-rarities", appliedFilters.includeRarities);
  setJoinedParam(url.searchParams, "include-traditions", appliedFilters.includeTraditions);
  setJoinedParam(url.searchParams, "include-actions", appliedFilters.includeActions);
  const urlString = url.toString();
  const label = "Open in Archives of Nethys";

  return {
    url: urlString,
    label,
    plainTextFallback: `${label}: ${urlString}`,
    appliedFilters,
  };
}

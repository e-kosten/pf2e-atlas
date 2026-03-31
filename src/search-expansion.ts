import {
  SearchCategory,
  NormalizedRecord,
  SearchAppliedBoost,
  SearchExpansionScope,
  SearchFilters,
  SearchMatchedRule,
  SearchQueryAnalysis,
  SearchSkippedRule,
  SourceCategory,
} from "./types.js";
import { getCategoryForSubcategory, getCategoryKeywordAnchors, getSubcategoryKeywordAnchors, SEARCH_CATEGORIES } from "./categories.js";
import { normalizeText, uniqueSorted } from "./utils.js";

type BoostTarget = "traits" | "nameTokens" | "metadataTokens";

type WeightedBoostInput = string | { token: string; weight?: number };

export interface SearchExpansionRule {
  id: string;
  label: string;
  triggers: string[];
  scope?: SearchExpansionScope;
  boosts: {
    traits?: WeightedBoostInput[];
    nameTokens?: WeightedBoostInput[];
    metadataTokens?: WeightedBoostInput[];
  };
}

type MatchedRuleState = {
  rule: SearchExpansionRule;
  matchedTriggers: string[];
  appliedBoosts: SearchMatchedRule["appliedBoosts"];
};

export type SearchQueryAnalysisState = SearchQueryAnalysis & {
  baseTraitWeights: Map<string, number>;
  baseNameWeights: Map<string, number>;
  baseMetadataWeights: Map<string, number>;
  ruleStates: MatchedRuleState[];
};

export type CandidateQueryWeights = {
  traitWeights: Map<string, number>;
  nameWeights: Map<string, number>;
  metadataWeights: Map<string, number>;
  matchedRuleIds: string[];
};

const DEFAULT_BOOST_WEIGHTS: Record<BoostTarget, number> = {
  traits: 0.9,
  nameTokens: 0.9,
  metadataTokens: 0.8,
};

export const DEFAULT_SEARCH_EXPANSION_RULES: SearchExpansionRule[] = [
  {
    id: "spectral-undead",
    label: "Spectral and undead encounters",
    triggers: ["ghost", "haunted", "possession", "possessed", "spectral", "restless spirit"],
    boosts: {
      traits: ["ghost", "spirit", "incorporeal", "undead"],
      metadataTokens: ["cursed", "curse"],
    },
  },
  {
    id: "swarm-infestation",
    label: "Swarm and infestation themes",
    triggers: ["swarm", "infestation", "vermin", "hive"],
    scope: {
      categories: ["creatures", "hazards"],
    },
    boosts: {
      traits: ["swarm"],
      nameTokens: ["crawling"],
      metadataTokens: ["infested", "verminous"],
    },
  },
  {
    id: "maritime-depths",
    label: "Maritime and shipwreck themes",
    triggers: ["ship", "voyage", "hold", "harbor", "sea", "ocean", "drowned", "shipwreck"],
    scope: {
      categories: ["creatures", "hazards"],
    },
    boosts: {
      traits: ["water", "aquatic"],
      metadataTokens: ["sailor", "wreck", "drowned"],
    },
  },
  {
    id: "body-horror",
    label: "Body horror and dismemberment",
    triggers: ["body horror", "severed limbs", "severed limb", "limbs", "limb", "gore"],
    scope: {
      categories: ["creatures", "hazards"],
    },
    boosts: {
      nameTokens: ["crawling", "hand", "limb"],
      metadataTokens: ["stitched", "flesh"],
    },
  },
  {
    id: "poison-plague",
    label: "Poison and plague",
    triggers: ["poison", "venom", "toxin", "plague", "pestilence", "disease"],
    boosts: {
      traits: ["poison", "disease"],
      metadataTokens: ["venom", "toxin", "plague"],
    },
  },
  {
    id: "fire-destruction",
    label: "Fire and ash",
    triggers: ["fire", "inferno", "lava", "ash", "burning", "flame"],
    boosts: {
      traits: ["fire"],
      metadataTokens: ["flame", "burning", "ember"],
    },
  },
  {
    id: "cold-winter",
    label: "Cold and winter",
    triggers: ["cold", "frost", "ice", "blizzard", "winter", "frozen"],
    boosts: {
      traits: ["cold"],
      metadataTokens: ["frozen", "rime", "glacial"],
    },
  },
  {
    id: "dream-mind",
    label: "Dreams, psychic horror, and illusion",
    triggers: ["dream", "nightmare", "madness", "psychic", "illusion"],
    boosts: {
      traits: ["mental", "illusion"],
      metadataTokens: ["nightmare", "dream", "psychic"],
    },
  },
  {
    id: "shadow-darkness",
    label: "Shadow and darkness",
    triggers: ["shadow", "darkness", "gloom", "umbral", "night"],
    boosts: {
      traits: ["shadow", "darkness"],
      metadataTokens: ["gloom", "night", "umbral"],
    },
  },
  {
    id: "void-death",
    label: "Void and death magic",
    triggers: ["void", "negative", "death", "necrotic", "grave"],
    boosts: {
      traits: ["void", "death", "negative"],
      metadataTokens: ["grave", "deathly", "necrotic"],
    },
  },
  {
    id: "occult-aberration",
    label: "Occult, aberrant, and cosmic horror",
    triggers: ["occult", "aberration", "eldritch", "cosmic", "alien"],
    boosts: {
      traits: ["occult", "aberration"],
      metadataTokens: ["eldritch", "cosmic", "alien"],
    },
  },
  {
    id: "fungus-rot",
    label: "Fungus, spores, and rot",
    triggers: ["fungus", "fungal", "mold", "mould", "spore", "rot", "decay"],
    boosts: {
      traits: ["fungus", "plant", "disease"],
      metadataTokens: ["spore", "rot", "decay"],
    },
  },
  {
    id: "blighted-wilds",
    label: "Blight, tainted growth, and corrupted wilds",
    triggers: ["blight", "blighted", "tainted", "tainted growth", "corrupted woodland", "corrupted grove", "rotting grove"],
    scope: {
      categories: ["creatures", "hazards"],
    },
    boosts: {
      traits: ["plant", "fungus", "ooze", "disease"],
      metadataTokens: ["blight", "blighted", "tainted", "rot", "spore"],
    },
  },
  {
    id: "wilderness-primal",
    label: "Wilderness and primal ecology",
    triggers: ["wilderness", "forest", "woodland", "jungle", "beast", "primal", "wild"],
    scope: {
      categories: ["creatures", "hazards", "spells"],
    },
    boosts: {
      traits: ["animal", "beast", "plant", "primal"],
      metadataTokens: ["wild", "forest", "woodland", "jungle"],
    },
  },
  {
    id: "fey-tricksters",
    label: "Fey, sprites, and trickster creatures",
    triggers: ["fey", "sprite", "pixie", "gremlin", "trickster", "prankster"],
    scope: {
      categories: ["creatures", "hazards"],
    },
    boosts: {
      traits: ["fey"],
      metadataTokens: ["trickster", "sprite", "pixie", "gremlin"],
    },
  },
  {
    id: "construct-clockwork",
    label: "Constructs and clockwork",
    triggers: ["construct", "clockwork", "automaton", "mechanical"],
    boosts: {
      traits: ["construct", "clockwork", "mechanical"],
      metadataTokens: ["gearwork", "automaton", "mechanical"],
    },
  },
  {
    id: "dragon-elemental",
    label: "Dragon and elemental power",
    triggers: ["dragon", "draconic", "elemental"],
    boosts: {
      traits: ["dragon", "elemental"],
      metadataTokens: ["draconic", "wyrm", "elemental"],
    },
  },
  {
    id: "giant-brutal",
    label: "Giants and brutal might",
    triggers: ["giant", "titanic", "massive", "brutal"],
    scope: {
      categories: ["creatures", "hazards"],
    },
    boosts: {
      traits: ["giant"],
      metadataTokens: ["massive", "titanic", "brutal"],
    },
  },
  {
    id: "storm-spells",
    label: "Storm and tempest magic",
    triggers: ["storm", "lightning", "thunder", "tempest"],
    scope: {
      categories: ["spells"],
    },
    boosts: {
      traits: ["air", "electricity", "sonic"],
      metadataTokens: ["tempest", "thunder", "lightning"],
    },
  },
  {
    id: "healing-spells",
    label: "Healing and restorative magic",
    triggers: ["healing", "restoration", "recovery", "cure", "mend"],
    scope: {
      categories: ["spells"],
    },
    boosts: {
      traits: ["healing", "vitality"],
      metadataTokens: ["heal", "restoration", "recovery"],
    },
  },
  {
    id: "warding-spells",
    label: "Protection, wards, and abjurations",
    triggers: ["ward", "shielding", "barrier", "protection", "abjuration"],
    scope: {
      categories: ["spells"],
    },
    boosts: {
      traits: ["abjuration"],
      metadataTokens: ["ward", "barrier", "protection"],
    },
  },
  {
    id: "curse-spells",
    label: "Curses, hexes, and maledictions",
    triggers: ["curse", "cursed", "hex", "malediction"],
    scope: {
      categories: ["spells"],
    },
    boosts: {
      traits: ["curse", "occult"],
      metadataTokens: ["curse", "hex", "malediction"],
    },
  },
  {
    id: "trap-hazards",
    label: "Traps and ambush hazards",
    triggers: ["trap", "snare", "pressure plate", "pitfall", "tripwire", "deadfall", "booby trap"],
    scope: {
      categories: ["hazards"],
      subcategories: ["trap"],
    },
    boosts: {
      traits: ["trap", "mechanical"],
      metadataTokens: ["snare", "trap", "tripwire"],
    },
  },
  {
    id: "haunt-hazards",
    label: "Haunts and supernatural hazards",
    triggers: ["haunt", "haunted", "poltergeist", "manifestation"],
    scope: {
      categories: ["hazards"],
      subcategories: ["haunt"],
    },
    boosts: {
      traits: ["haunt", "magical"],
      metadataTokens: ["haunting", "manifestation", "spirit"],
    },
  },
  {
    id: "encounter-ecology",
    label: "Encounter ecology and lair language",
    triggers: ["lair", "brood", "nest", "den", "territory", "stalk", "ambush", "ambusher", "ambush predator"],
    scope: {
      categories: ["creatures", "hazards"],
    },
    boosts: {
      metadataTokens: ["lair", "nest", "den", "territory", "ambush", "stalk"],
    },
  },
  {
    id: "stealth-assailants",
    label: "Assassins, skulkers, and ambush predators",
    triggers: ["assassin", "ambush", "skulker", "stalker", "sneak attack", "hit and run"],
    scope: {
      categories: ["creatures"],
    },
    boosts: {
      metadataTokens: ["assassin", "ambush", "stalker", "stealth", "skulking"],
    },
  },
  {
    id: "stealth-gear",
    label: "Stealth and infiltration gear",
    triggers: ["stealth gear", "infiltration tools", "lockpick", "lockpicks", "camouflage", "disguise kit", "smoke bomb", "concealed weapon"],
    scope: {
      categories: ["equipment"],
      subcategories: ["gear", "consumable", "armor", "weapon"],
    },
    boosts: {
      metadataTokens: ["stealth", "silent", "concealed", "subtle"],
    },
  },
  {
    id: "healing-gear",
    label: "Healing and restorative gear",
    triggers: ["healing", "restoration", "medicine", "recovery", "antidote"],
    scope: {
      categories: ["equipment"],
      subcategories: ["gear", "consumable"],
    },
    boosts: {
      metadataTokens: ["healing", "restorative", "recovery", "antidote"],
    },
  },
  {
    id: "defense-gear",
    label: "Defensive and protective gear",
    triggers: ["defense", "defensive", "shielding", "protection", "warding"],
    scope: {
      categories: ["equipment"],
      subcategories: ["gear", "armor", "shield", "weapon"],
    },
    boosts: {
      metadataTokens: ["defense", "protective", "shield", "ward"],
    },
  },
  {
    id: "alchemical-gear",
    label: "Alchemical and bomb gear",
    triggers: ["alchemical", "bomb", "elixir", "mutagen", "tincture"],
    scope: {
      categories: ["equipment"],
      subcategories: ["consumable", "gear"],
    },
    boosts: {
      traits: ["alchemical"],
      metadataTokens: ["bomb", "elixir", "mutagen"],
    },
  },
  {
    id: "holy-radiance",
    label: "Holy and radiant themes",
    triggers: ["holy", "radiant", "celestial", "angelic"],
    boosts: {
      traits: ["holy", "divine"],
      metadataTokens: ["radiance", "celestial"],
    },
  },
  {
    id: "fiendish-corruption",
    label: "Fiendish and corruptive themes",
    triggers: ["demonic", "infernal", "abyssal", "fiendish", "corruption"],
    boosts: {
      traits: ["fiend", "demon", "devil", "unholy"],
      metadataTokens: ["corruption", "infernal"],
    },
  },
];

function tokenize(value: string): string[] {
  const normalized = normalizeText(value);
  return normalized ? normalized.split(" ").filter(Boolean) : [];
}

function normalizeScopeValues(values: string[] | undefined): string[] {
  return values?.map((value) => normalizeText(value)).filter(Boolean) ?? [];
}

function normalizeBoostInput(target: BoostTarget, boost: WeightedBoostInput): SearchAppliedBoost | null {
  if (typeof boost === "string") {
    const token = normalizeText(boost);
    return token
      ? {
          token,
          weight: DEFAULT_BOOST_WEIGHTS[target],
        }
      : null;
  }

  const token = normalizeText(boost.token);
  if (!token) {
    return null;
  }

  return {
    token,
    weight: boost.weight ?? DEFAULT_BOOST_WEIGHTS[target],
  };
}

function normalizeBoosts(target: BoostTarget, boosts: WeightedBoostInput[] | undefined): SearchAppliedBoost[] {
  return (boosts ?? [])
    .map((boost) => normalizeBoostInput(target, boost))
    .filter((boost): boost is SearchAppliedBoost => Boolean(boost));
}

function matchesPattern(normalizedQuery: string, queryTokens: Set<string>, trigger: string): boolean {
  const normalizedTrigger = normalizeText(trigger);
  if (!normalizedTrigger) {
    return false;
  }

  return normalizedTrigger.includes(" ")
    ? normalizedQuery.includes(normalizedTrigger)
    : queryTokens.has(normalizedTrigger);
}

function scopeConflictsWithFilters(scope: SearchExpansionScope | undefined, filters: SearchFilters): boolean {
  if (!scope) {
    return false;
  }

  const categories = scope.categories?.map((category) => normalizeText(category)).filter(Boolean) ?? [];
  if (categories.length > 0 && filters.category && !categories.includes(normalizeText(filters.category))) {
    return true;
  }

  const subcategories = normalizeScopeValues(scope.subcategories);
  if (subcategories.length > 0 && filters.subcategory && !subcategories.includes(normalizeText(filters.subcategory))) {
    return true;
  }

  const packNames = normalizeScopeValues(scope.packNames);
  if (packNames.length > 0 && filters.pack && !packNames.includes(normalizeText(filters.pack))) {
    return true;
  }

  if (scope.sourceCategories && scope.sourceCategories.length > 0) {
    if (filters.coreOnly && !scope.sourceCategories.includes("core")) {
      return true;
    }

    if (filters.excludeAdventureContent && scope.sourceCategories.every((category) => category === "adventure")) {
      return true;
    }
  }

  return false;
}

function scopeMatchesRecord(scope: SearchExpansionScope | undefined, record: NormalizedRecord): boolean {
  if (!scope) {
    return true;
  }

  const categories = scope.categories?.map((category) => normalizeText(category)).filter(Boolean) ?? [];
  if (categories.length > 0 && !categories.includes(normalizeText(record.category))) {
    return false;
  }

  const subcategories = normalizeScopeValues(scope.subcategories);
  if (subcategories.length > 0) {
    const recordSubcategories = new Set(record.subcategories.map((subcategory) => normalizeText(subcategory)).filter(Boolean));
    if (!subcategories.some((subcategory) => recordSubcategories.has(subcategory))) {
      return false;
    }
  }

  const packNames = normalizeScopeValues(scope.packNames);
  if (packNames.length > 0 && !packNames.includes(normalizeText(record.packName))) {
    return false;
  }

  if (scope.sourceCategories && scope.sourceCategories.length > 0 && !scope.sourceCategories.includes(record.sourceCategory)) {
    return false;
  }

  return true;
}

function addWeight(weights: Map<string, number>, token: string, weight: number): void {
  const normalized = normalizeText(token);
  if (!normalized) {
    return;
  }

  weights.set(normalized, Math.max(weight, weights.get(normalized) ?? 0));
}

function applyBoosts(weights: Map<string, number>, boosts: SearchAppliedBoost[]): void {
  for (const boost of boosts) {
    addWeight(weights, boost.token, boost.weight);
  }
}

function collectBoostedTokens(weights: Map<string, number>, queryTokenSet: Set<string>): string[] {
  return [...weights.keys()]
    .filter((token) => !queryTokenSet.has(token))
    .sort((left, right) => left.localeCompare(right));
}

function createScopeCopy(scope: SearchExpansionScope | undefined): SearchExpansionScope | undefined {
  if (!scope) {
    return undefined;
  }

  return {
    categories: scope.categories ? [...scope.categories] : undefined,
    subcategories: scope.subcategories ? [...scope.subcategories] : undefined,
    packNames: scope.packNames ? [...scope.packNames] : undefined,
    sourceCategories: scope.sourceCategories ? [...scope.sourceCategories] : undefined,
  };
}

function addCategoryScore(scores: Map<SearchCategory, number>, category: SearchCategory, value: number): void {
  scores.set(category, (scores.get(category) ?? 0) + value);
}

function addSubcategoryScore(scores: Map<string, number>, subcategory: string, value: number): void {
  const normalized = normalizeText(subcategory);
  if (!normalized) {
    return;
  }
  scores.set(normalized, (scores.get(normalized) ?? 0) + value);
}

function scoreKeywordAnchors(
  normalizedQuery: string,
  queryTokenSet: Set<string>,
  categoryScores: Map<SearchCategory, number>,
  subcategoryScores: Map<string, number>,
): void {
  const categoryAnchors = getCategoryKeywordAnchors();
  for (const category of SEARCH_CATEGORIES) {
    for (const keyword of categoryAnchors[category] ?? []) {
      if (matchesPattern(normalizedQuery, queryTokenSet, keyword)) {
        addCategoryScore(categoryScores, category, keyword.includes(" ") ? 2 : 1.5);
      }
    }
  }

  for (const entry of getSubcategoryKeywordAnchors()) {
    for (const keyword of entry.keywords) {
      if (matchesPattern(normalizedQuery, queryTokenSet, keyword)) {
        addCategoryScore(categoryScores, entry.category, entry.weight ?? 2);
        addSubcategoryScore(subcategoryScores, entry.subcategory, entry.weight ?? 2);
      }
    }
  }
}

function inferCategorySelection(
  normalizedQuery: string,
  queryTokenSet: Set<string>,
  filters: SearchFilters,
  matchedRules: SearchMatchedRule[],
): { category: SearchCategory | null; subcategory: string | null } {
  const categoryScores = new Map<SearchCategory, number>();
  const subcategoryScores = new Map<string, number>();

  if (filters.category) {
    addCategoryScore(categoryScores, filters.category, 100);
  }
  if (filters.subcategory) {
    addSubcategoryScore(subcategoryScores, filters.subcategory, 100);
    const parent = getCategoryForSubcategory(filters.subcategory);
    if (parent) {
      addCategoryScore(categoryScores, parent, 100);
    }
  }

  for (const rule of matchedRules) {
    for (const category of rule.scope?.categories ?? []) {
      addCategoryScore(categoryScores, category, 2);
    }

    for (const subcategory of rule.scope?.subcategories ?? []) {
      addSubcategoryScore(subcategoryScores, subcategory, 1.5);
      const parent = getCategoryForSubcategory(subcategory);
      if (parent) {
        addCategoryScore(categoryScores, parent, 1);
      }
    }
  }

  scoreKeywordAnchors(normalizedQuery, queryTokenSet, categoryScores, subcategoryScores);

  const rankedCategories = [...categoryScores.entries()]
    .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]));
  const topCategory = rankedCategories[0] ?? null;
  const runnerUpCategory = rankedCategories[1] ?? null;
  const inferredCategory = topCategory && topCategory[1] >= 2 && (
    !runnerUpCategory || topCategory[1] >= runnerUpCategory[1] + 1.5
  )
    ? topCategory[0]
    : null;

  let inferredSubcategory: string | null = null;
  if (filters.subcategory) {
    inferredSubcategory = normalizeText(filters.subcategory) || null;
  } else if (inferredCategory) {
    const rankedSubcategories = [...subcategoryScores.entries()]
      .filter(([subcategory]) => getCategoryForSubcategory(subcategory) === inferredCategory)
      .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]));
    const topSubcategory = rankedSubcategories[0] ?? null;
    const runnerUpSubcategory = rankedSubcategories[1] ?? null;
    if (topSubcategory && topSubcategory[1] >= 2 && (!runnerUpSubcategory || topSubcategory[1] >= runnerUpSubcategory[1] + 1)) {
      inferredSubcategory = topSubcategory[0];
    }
  }

  return {
    category: filters.category ?? inferredCategory,
    subcategory: filters.subcategory ?? inferredSubcategory,
  };
}

export function buildSearchQueryAnalysis(
  query: string,
  filters: SearchFilters,
  options: {
    expandQuery?: boolean;
    rules?: SearchExpansionRule[];
  } = {},
): SearchQueryAnalysisState | null {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) {
    return null;
  }

  const queryTokens = normalizedQuery.split(" ").filter(Boolean);
  const queryTokenSet = new Set(queryTokens);
  const baseTraitWeights = new Map<string, number>();
  const baseNameWeights = new Map<string, number>();
  const baseMetadataWeights = new Map<string, number>();
  for (const token of queryTokens) {
    addWeight(baseTraitWeights, token, 1);
    addWeight(baseNameWeights, token, 0.65);
    addWeight(baseMetadataWeights, token, 0.75);
  }

  const traitWeights = new Map(baseTraitWeights);
  const nameWeights = new Map(baseNameWeights);
  const metadataWeights = new Map(baseMetadataWeights);
  const matchedRules: SearchMatchedRule[] = [];
  const skippedRules: SearchSkippedRule[] = [];
  const ruleStates: MatchedRuleState[] = [];
  const expansionEnabled = options.expandQuery ?? true;
  const rules = options.rules ?? DEFAULT_SEARCH_EXPANSION_RULES;

  for (const rule of rules) {
    const matchedTriggers = rule.triggers
      .map((trigger) => normalizeText(trigger))
      .filter((trigger): trigger is string => Boolean(trigger))
      .filter((trigger) => matchesPattern(normalizedQuery, queryTokenSet, trigger));
    if (matchedTriggers.length === 0) {
      continue;
    }

    if (!expansionEnabled) {
      skippedRules.push({
        id: rule.id,
        label: rule.label,
        matchedTriggers,
        scope: createScopeCopy(rule.scope),
        reason: "expansion_disabled",
      });
      continue;
    }

    if (scopeConflictsWithFilters(rule.scope, filters)) {
      skippedRules.push({
        id: rule.id,
        label: rule.label,
        matchedTriggers,
        scope: createScopeCopy(rule.scope),
        reason: "scope_mismatch",
      });
      continue;
    }

    const appliedBoosts = {
      traits: normalizeBoosts("traits", rule.boosts.traits),
      nameTokens: normalizeBoosts("nameTokens", rule.boosts.nameTokens),
      metadataTokens: normalizeBoosts("metadataTokens", rule.boosts.metadataTokens),
    };

    applyBoosts(traitWeights, appliedBoosts.traits);
    applyBoosts(nameWeights, appliedBoosts.nameTokens);
    applyBoosts(metadataWeights, appliedBoosts.metadataTokens);

    matchedRules.push({
      id: rule.id,
      label: rule.label,
      matchedTriggers,
      scope: createScopeCopy(rule.scope),
      appliedBoosts,
    });
    ruleStates.push({
      rule,
      matchedTriggers,
      appliedBoosts,
    });
  }

  const boostedTraits = collectBoostedTokens(traitWeights, queryTokenSet);
  const boostedNameTokens = collectBoostedTokens(nameWeights, queryTokenSet);
  const boostedMetadataTokens = collectBoostedTokens(metadataWeights, queryTokenSet);
  const expandedQuery = uniqueSorted([
    ...queryTokens,
    ...boostedTraits,
    ...boostedNameTokens,
    ...boostedMetadataTokens,
  ]).join(" ");
  const inference = inferCategorySelection(normalizedQuery, queryTokenSet, filters, matchedRules);

  return {
    rawQuery: query,
    normalizedQuery,
    queryTokens,
    expandedQuery,
    inferredCategory: inference.category,
    inferredSubcategory: inference.subcategory,
    boostedTraits,
    boostedNameTokens,
    boostedMetadataTokens,
    matchedRules,
    skippedRules,
    baseTraitWeights,
    baseNameWeights,
    baseMetadataWeights,
    ruleStates,
  };
}

export function inferSearchCategorySelection(
  query: string,
  filters: SearchFilters,
  analysis?: SearchQueryAnalysis | null,
): { category: SearchCategory | null; subcategory: string | null } {
  if (analysis) {
    return {
      category: analysis.inferredCategory,
      subcategory: analysis.inferredSubcategory,
    };
  }

  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) {
    return {
      category: filters.category ?? null,
      subcategory: filters.subcategory ? normalizeText(filters.subcategory) : null,
    };
  }

  const queryTokenSet = new Set(normalizedQuery.split(" ").filter(Boolean));
  return inferCategorySelection(normalizedQuery, queryTokenSet, filters, []);
}

export function buildCandidateQueryWeights(record: NormalizedRecord, analysis: SearchQueryAnalysisState): CandidateQueryWeights {
  const traitWeights = new Map(analysis.baseTraitWeights);
  const nameWeights = new Map(analysis.baseNameWeights);
  const metadataWeights = new Map(analysis.baseMetadataWeights);
  const matchedRuleIds: string[] = [];

  for (const ruleState of analysis.ruleStates) {
    if (!scopeMatchesRecord(ruleState.rule.scope, record)) {
      continue;
    }

    matchedRuleIds.push(ruleState.rule.id);
    applyBoosts(traitWeights, ruleState.appliedBoosts.traits);
    applyBoosts(nameWeights, ruleState.appliedBoosts.nameTokens);
    applyBoosts(metadataWeights, ruleState.appliedBoosts.metadataTokens);
  }

  return {
    traitWeights,
    nameWeights,
    metadataWeights,
    matchedRuleIds,
  };
}

import {
  NormalizedRecord,
  SearchAppliedBoost,
  SearchExpansionScope,
  SearchFilters,
  SearchMatchedRule,
  SearchQueryAnalysis,
  SearchSkippedRule,
  SourceCategory,
} from "./types.js";
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
      recordTypes: ["npc", "hazard"],
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
      recordTypes: ["npc", "hazard"],
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
      recordTypes: ["npc", "hazard"],
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
      recordTypes: ["npc", "hazard"],
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
      recordTypes: ["npc", "hazard", "spell"],
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
      recordTypes: ["npc", "hazard"],
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
      recordTypes: ["npc", "hazard"],
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
      recordTypes: ["spell"],
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
      recordTypes: ["spell"],
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
      recordTypes: ["spell"],
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
      recordTypes: ["spell"],
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
      recordTypes: ["hazard"],
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
      recordTypes: ["hazard"],
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
      recordTypes: ["npc", "hazard"],
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
      recordTypes: ["npc"],
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
      documentTypes: ["Item"],
      itemCategories: ["equipment", "consumable", "armor", "weapon"],
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
      documentTypes: ["Item"],
      itemCategories: ["equipment", "consumable"],
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
      documentTypes: ["Item"],
      itemCategories: ["equipment", "armor", "shield", "weapon"],
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
      documentTypes: ["Item"],
      itemCategories: ["consumable", "equipment"],
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

function inferDocumentTypeFromFilters(filters: SearchFilters): string | null {
  if (filters.documentType) {
    return normalizeText(filters.documentType);
  }

  const normalizedRecordType = normalizeText(filters.recordType ?? "");
  if (normalizedRecordType === "npc" || normalizedRecordType === "hazard") {
    return "actor";
  }

  if (filters.itemCategory) {
    return "item";
  }

  const itemLikeRecordTypes = new Set([
    "action",
    "feat",
    "condition",
    "spell",
    "equipment",
    "consumable",
    "armor",
    "weapon",
    "shield",
    "backpack",
    "treasure",
    "deityboon",
    "ancestry",
    "class",
    "background",
  ]);
  if (itemLikeRecordTypes.has(normalizedRecordType)) {
    return "item";
  }

  return null;
}

function scopeConflictsWithFilters(scope: SearchExpansionScope | undefined, filters: SearchFilters): boolean {
  if (!scope) {
    return false;
  }

  const documentTypes = normalizeScopeValues(scope.documentTypes);
  const inferredDocumentType = inferDocumentTypeFromFilters(filters);
  if (documentTypes.length > 0 && inferredDocumentType && !documentTypes.includes(inferredDocumentType)) {
    return true;
  }

  const recordTypes = normalizeScopeValues(scope.recordTypes);
  if (recordTypes.length > 0 && filters.recordType && !recordTypes.includes(normalizeText(filters.recordType))) {
    return true;
  }

  const itemCategories = normalizeScopeValues(scope.itemCategories);
  if (itemCategories.length > 0 && filters.itemCategory && !itemCategories.includes(normalizeText(filters.itemCategory))) {
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

  const documentTypes = normalizeScopeValues(scope.documentTypes);
  if (documentTypes.length > 0 && !documentTypes.includes(normalizeText(record.documentType))) {
    return false;
  }

  const recordTypes = normalizeScopeValues(scope.recordTypes);
  if (recordTypes.length > 0 && !recordTypes.includes(normalizeText(record.type))) {
    return false;
  }

  const itemCategories = normalizeScopeValues(scope.itemCategories);
  if (itemCategories.length > 0 && !itemCategories.includes(normalizeText(record.itemCategory ?? ""))) {
    return false;
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
    documentTypes: scope.documentTypes ? [...scope.documentTypes] : undefined,
    recordTypes: scope.recordTypes ? [...scope.recordTypes] : undefined,
    itemCategories: scope.itemCategories ? [...scope.itemCategories] : undefined,
    packNames: scope.packNames ? [...scope.packNames] : undefined,
    sourceCategories: scope.sourceCategories ? [...scope.sourceCategories] : undefined,
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

  return {
    rawQuery: query,
    normalizedQuery,
    queryTokens,
    expandedQuery,
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

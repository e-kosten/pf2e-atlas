import { SearchFilters, SearchQueryAnalysis } from "./types.js";
import { buildSearchQueryAnalysis, DEFAULT_SEARCH_EXPANSION_RULES, SearchExpansionRule } from "./search-expansion.js";
import { normalizeText } from "./utils.js";

type SearchPlanQuery = {
  label: string;
  purpose: string;
  arguments: SearchFilters;
};

type SearchPlanResult = {
  intent: string;
  inputFilters: SearchFilters;
  query: SearchQueryAnalysis | null;
  recognizedSemantics: Array<{ id: string; label: string }>;
  uncoveredTokens: string[];
  suggestedFilters: {
    preferredRecordTypes: string[];
    preferredDocumentTypes: string[];
    preferredItemCategories: string[];
    traitsAny: string[];
  };
  recommendedQueries: SearchPlanQuery[];
  notes: string[];
};

function uniqueSorted(values: string[]): string[] {
  return [...new Set(values.filter(Boolean))].sort((left, right) => left.localeCompare(right));
}

function rankValues(values: string[]): string[] {
  const counts = new Map<string, number>();
  for (const value of values) {
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }

  return [...counts.entries()]
    .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))
    .map(([value]) => value);
}

function collectRuleScopeValues(
  analysis: SearchQueryAnalysis,
  selector: (rule: SearchQueryAnalysis["matchedRules"][number]) => string[] | undefined,
): string[] {
  return rankValues(
    analysis.matchedRules.flatMap((rule) => selector(rule) ?? []).map((value) => normalizeText(value)).filter(Boolean),
  );
}

function collectBoostedTraits(analysis: SearchQueryAnalysis): string[] {
  const weights = new Map<string, number>();
  for (const rule of analysis.matchedRules) {
    for (const boost of rule.appliedBoosts.traits) {
      weights.set(boost.token, Math.max(boost.weight, weights.get(boost.token) ?? 0));
    }
  }

  return [...weights.entries()]
    .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))
    .map(([token]) => token);
}

function collectCoveredTokens(analysis: SearchQueryAnalysis): Set<string> {
  const covered = new Set<string>();
  for (const rule of analysis.matchedRules) {
    for (const trigger of rule.matchedTriggers) {
      for (const token of normalizeText(trigger).split(" ").filter(Boolean)) {
        covered.add(token);
      }
    }
  }

  for (const token of [
    ...analysis.boostedTraits,
    ...analysis.boostedNameTokens,
    ...analysis.boostedMetadataTokens,
  ]) {
    covered.add(token);
  }

  return covered;
}

function recommendQueries(intent: string, filters: SearchFilters, analysis: SearchQueryAnalysis | null, suggestedTraitsAny: string[]): SearchPlanQuery[] {
  const base: SearchFilters = {
    ...filters,
    mode: filters.mode ?? "hybrid",
    themeQuery: filters.themeQuery ?? intent,
    expandQuery: filters.expandQuery ?? true,
  };

  const queries: SearchPlanQuery[] = [
    {
      label: "broad_hybrid",
      purpose: "Bounded semantic search with current filters and ontology expansion.",
      arguments: base,
    },
  ];

  if (suggestedTraitsAny.length > 0) {
    queries.push({
      label: "trait_hinted_hybrid",
      purpose: "Hybrid search narrowed by server-recognized taxonomy terms.",
      arguments: {
        ...base,
        traitsAny: uniqueSorted([...(filters.traitsAny ?? []), ...suggestedTraitsAny]).slice(0, 8),
      },
    });

    queries.push({
      label: "structured_backstop",
      purpose: "Deterministic structured search using recognized taxonomy traits.",
      arguments: {
        ...filters,
        mode: "structured",
        traitsAny: uniqueSorted([...(filters.traitsAny ?? []), ...suggestedTraitsAny]).slice(0, 8),
      },
    });
  }

  if (analysis && analysis.matchedRules.some((rule) => (rule.scope?.recordTypes ?? []).includes("hazard"))) {
    queries.push({
      label: "hazard_probe",
      purpose: "Separate hazard-focused retrieval when the ontology suggests traps, haunts, or environmental threats.",
      arguments: {
        ...base,
        recordType: "hazard",
      },
    });
  }

  return queries;
}

function buildNotes(analysis: SearchQueryAnalysis | null, filters: SearchFilters, suggestedTraitsAny: string[]): string[] {
  const notes: string[] = [];
  if (!analysis || analysis.matchedRules.length === 0) {
    notes.push("The intent query did not strongly match any built-in semantic domains; structured filters may matter more than vibe text.");
    return notes;
  }

  if (analysis.skippedRules.some((rule) => rule.reason === "scope_mismatch")) {
    notes.push("Some ontology rules matched the wording but were skipped because the current filters point at a different record family.");
  }

  if (!filters.recordType && analysis.matchedRules.some((rule) => (rule.scope?.recordTypes ?? []).length > 0)) {
    notes.push("The server recognized scoped domains; setting recordType explicitly may improve precision.");
  }

  if (suggestedTraitsAny.length > 0) {
    notes.push("A trait-hinted rerun is recommended because the server recognized taxonomy terms that map cleanly to structured filters.");
  }

  return notes;
}

export function buildSearchPlan(intent: string, filters: SearchFilters = {}): SearchPlanResult {
  const normalizedIntent = intent.trim();
  const analysis = normalizedIntent
    ? buildSearchQueryAnalysis(normalizedIntent, filters, { expandQuery: filters.expandQuery ?? true, rules: DEFAULT_SEARCH_EXPANSION_RULES })
    : null;

  const preferredRecordTypes = analysis
    ? collectRuleScopeValues(analysis, (rule) => rule.scope?.recordTypes)
    : [];
  const preferredDocumentTypes = analysis
    ? collectRuleScopeValues(analysis, (rule) => rule.scope?.documentTypes)
    : [];
  const preferredItemCategories = analysis
    ? collectRuleScopeValues(analysis, (rule) => rule.scope?.itemCategories)
    : [];
  const traitsAny = analysis ? collectBoostedTraits(analysis).slice(0, 8) : [];
  const coveredTokens = analysis ? collectCoveredTokens(analysis) : new Set<string>();
  const uncoveredTokens = analysis
    ? analysis.queryTokens.filter((token) => !coveredTokens.has(token))
    : [];

  return {
    intent,
    inputFilters: filters,
    query: analysis
      ? {
          rawQuery: analysis.rawQuery,
          normalizedQuery: analysis.normalizedQuery,
          queryTokens: analysis.queryTokens,
          expandedQuery: analysis.expandedQuery,
          boostedTraits: analysis.boostedTraits,
          boostedNameTokens: analysis.boostedNameTokens,
          boostedMetadataTokens: analysis.boostedMetadataTokens,
          matchedRules: analysis.matchedRules,
          skippedRules: analysis.skippedRules,
        }
      : null,
    recognizedSemantics: analysis?.matchedRules.map((rule) => ({ id: rule.id, label: rule.label })) ?? [],
    uncoveredTokens,
    suggestedFilters: {
      preferredRecordTypes,
      preferredDocumentTypes,
      preferredItemCategories,
      traitsAny,
    },
    recommendedQueries: recommendQueries(intent, filters, analysis, traitsAny),
    notes: buildNotes(analysis, filters, traitsAny),
  };
}

export function summarizeExpansionRules(rules: SearchExpansionRule[] = DEFAULT_SEARCH_EXPANSION_RULES): Array<{
  id: string;
  label: string;
  triggers: string[];
  scope?: SearchExpansionRule["scope"];
  boosts: {
    traits: string[];
    nameTokens: string[];
    metadataTokens: string[];
  };
}> {
  return rules.map((rule) => ({
    id: rule.id,
    label: rule.label,
    triggers: [...rule.triggers],
    scope: rule.scope
      ? {
          documentTypes: rule.scope.documentTypes ? [...rule.scope.documentTypes] : undefined,
          recordTypes: rule.scope.recordTypes ? [...rule.scope.recordTypes] : undefined,
          itemCategories: rule.scope.itemCategories ? [...rule.scope.itemCategories] : undefined,
          packNames: rule.scope.packNames ? [...rule.scope.packNames] : undefined,
          sourceCategories: rule.scope.sourceCategories ? [...rule.scope.sourceCategories] : undefined,
        }
      : undefined,
    boosts: {
      traits: uniqueSorted((rule.boosts.traits ?? []).map((boost) => typeof boost === "string" ? normalizeText(boost) : normalizeText(boost.token))),
      nameTokens: uniqueSorted((rule.boosts.nameTokens ?? []).map((boost) => typeof boost === "string" ? normalizeText(boost) : normalizeText(boost.token))),
      metadataTokens: uniqueSorted((rule.boosts.metadataTokens ?? []).map((boost) => typeof boost === "string" ? normalizeText(boost) : normalizeText(boost.token))),
    },
  }));
}

import { useQuery } from "@tanstack/react-query";
import { assertSearchIdentity } from "../search/searchIdentity";
import {
  discoverFilterCounts,
  discoverFilterEditor,
  discoverFilterValues,
} from "../../api/atlasApi";
import type {
  FilterDiscoveryContext,
  FilterEditorView,
  FilterValueListView,
  FilterVocabularyScope,
  QueryLiteral,
  QueryPredicate,
} from "../../generated/atlas";

/** Clause location is editor addressing, not a contextual-facet eligibility check. */
export function findClauseIds(
  predicate: QueryPredicate | null,
  field: string,
  scope: string | null = null,
  currentScope: string | null = null,
): (string | null)[] {
  if (!predicate || typeof predicate !== "object") return [];
  if ("field" in predicate && predicate.field === field && currentScope === scope)
    return [predicate.clause_id || null];
  if (predicate.kind === "all_of" || predicate.kind === "any_of")
    return Array.isArray(predicate.children)
      ? predicate.children.flatMap((child) =>
          findClauseIds(child, field, scope, currentScope),
        )
      : [];
  if (predicate.kind === "exists")
    return findClauseIds(predicate.predicate, field, scope, predicate.collection);
  if (predicate.kind === "not")
    return findClauseIds(predicate.predicate, field, scope, currentScope);
  return [];
}
export function useFilterDiscovery({
  context,
  enabled = true,
  queryKeyPrefix,
  catalogIdentity = [],
  vocabularyScope = { kind: "artifact" },
}: {
  context: FilterDiscoveryContext;
  enabled?: boolean;
  queryKeyPrefix: readonly unknown[];
  catalogIdentity?: readonly unknown[];
  vocabularyScope?: FilterVocabularyScope;
}) {
  const editor = useQuery<FilterEditorView>({
    queryKey: ["filter-discovery", "editor", ...catalogIdentity],
    enabled,
    staleTime: Infinity,
    queryFn: ({ signal }) => discoverFilterEditor({ selected_field_ids: [] }, signal),
  });
  return {
    filterEditor: editor.data,
    filterCountsEnabled: enabled,
    filterCountQueryKey: queryKeyPrefix,
    filterDiscoveryContext: context,
    vocabularyScope,
    loading: editor.isLoading || editor.isFetching,
    errorMessage: editor.error?.message ?? null,
  };
}

/** Optional counts never own vocabulary or control whether a value is selectable. */
export function useFilterOptionCounts(
  context: FilterDiscoveryContext | undefined,
  fieldId: string,
  clauseId: string | null,
  values: QueryLiteral[],
  enabled: boolean,
  literalLimit: number,
  queryKeyPrefix: readonly unknown[] = [],
) {
  const literals = [...new Set(values)].slice(0, literalLimit);
  const query = useQuery({
    queryKey: [
      ...(queryKeyPrefix.length ? queryKeyPrefix : ["filter-discovery"]),
      "option-counts",
      context,
      fieldId,
      clauseId,
      literals,
      enabled,
    ],
    enabled: enabled && !!context && !context.text?.trim() && literals.length > 0,
    retry: false,
    queryFn: async ({ signal }) => {
      if (!context) throw new Error("Missing filter count context");
      const response = await discoverFilterCounts(
        { context, field_id: fieldId, clause_id: clauseId, values: literals },
        signal,
      );
      assertSearchIdentity(
        response,
        context.search_profile,
        context.expected_artifact_generation,
      );
      return response;
    },
  });
  const counts = query.data?.counts;
  return counts?.status === "available" && !query.error && enabled
    ? new Map(
        counts.value.options.map((option) => [
          String(option.value),
          option.distinct_roots,
        ]),
      )
    : new Map<string, bigint>();
}

export const VOCABULARY_PAGE_SIZE = 40;
export function useFilterVocabulary(
  scope: FilterVocabularyScope,
  fieldId: string,
  text: string,
  offset: number,
  selectedValues: QueryLiteral[],
  enabled = true,
) {
  return useQuery<Extract<FilterValueListView, { kind: "vocabulary" }>>({
    queryKey: ["filter-vocabulary", scope, fieldId, text, offset, selectedValues],
    enabled,
    // A selection edit must not temporarily remove or reorder the menu's rows.
    // Never carry suggestions across a different scope, field or page/search.
    placeholderData: (previous, previousQuery) =>
      JSON.stringify(previousQuery?.queryKey.slice(0, 5)) ===
      JSON.stringify(["filter-vocabulary", scope, fieldId, text, offset])
        ? previous
        : undefined,
    queryFn: async ({ signal }) => {
      const result = await discoverFilterValues(
        {
          kind: "vocabulary",
          scope,
          field_id: fieldId,
          text: text.trim() || null,
          offset,
          limit: VOCABULARY_PAGE_SIZE,
          selected_values: selectedValues,
        },
        signal,
      );
      if (result.kind !== "vocabulary")
        throw new Error("Unexpected contextual response to vocabulary request");
      return result;
    },
  });
}

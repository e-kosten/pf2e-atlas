import { keepPreviousData, useQueries, useQuery } from "@tanstack/react-query";
import {
  discoverFilterCounts,
  discoverFilterEditor,
  discoverFilterValues,
} from "../../api/atlasApi";
import type {
  FilterDiscoveryContext,
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
  countsEnabled = true,
  queryKeyPrefix,
  vocabularyScope = { kind: "artifact" },
}: {
  context: FilterDiscoveryContext;
  enabled?: boolean;
  countsEnabled?: boolean;
  queryKeyPrefix: readonly unknown[];
  vocabularyScope?: FilterVocabularyScope;
}) {
  const editor = useQuery({
    queryKey: [...queryKeyPrefix, "editor"],
    enabled,
    placeholderData: keepPreviousData,
    queryFn: () => discoverFilterEditor({ context, selected_field_ids: [] }),
  });
  const definitions = (editor.data?.groups ?? []).flatMap((group) =>
    group.fields.map((field) => field.definition),
  );
  const requests = (countsEnabled ? definitions : []).flatMap((field) =>
    findClauseIds(context.filter, field.path, field.scope).map((clauseId) => ({
      fieldId: field.id,
      clauseId,
    })),
  );
  const counts = useQueries({
    queries: requests.map(({ fieldId, clauseId }) => ({
      queryKey: [...queryKeyPrefix, "counts", fieldId, clauseId],
      enabled: enabled && !editor.isPlaceholderData,
      queryFn: () =>
        discoverFilterCounts({ context, field_id: fieldId, clause_id: clauseId }),
    })),
  });
  return {
    filterEditor: editor.data,
    filterCountsByField: Object.fromEntries(
      requests.map(({ fieldId, clauseId }, index) => [
        clauseId ? `${fieldId}:${clauseId}` : fieldId,
        counts[index]?.data,
      ]),
    ),
    filterDiscoveryContext: context,
    vocabularyScope,
    loading:
      editor.isLoading || editor.isFetching || counts.some((query) => query.isFetching),
    errorMessage:
      editor.error?.message ??
      counts.find((query) => query.error)?.error?.message ??
      null,
  };
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
  return useQuery({
    queryKey: ["filter-vocabulary", scope, fieldId, text, offset, selectedValues],
    enabled,
    // A selection edit must not temporarily remove or reorder the menu's rows.
    // Never carry suggestions across a different scope, field or page/search.
    placeholderData: (previous, previousQuery) =>
      JSON.stringify(previousQuery?.queryKey.slice(0, 5)) ===
      JSON.stringify(["filter-vocabulary", scope, fieldId, text, offset])
        ? previous
        : undefined,
    queryFn: async () => {
      const result = await discoverFilterValues({
        kind: "vocabulary",
        scope,
        field_id: fieldId,
        text: text.trim() || null,
        offset,
        limit: VOCABULARY_PAGE_SIZE,
        selected_values: selectedValues,
      });
      if (result.kind !== "vocabulary")
        throw new Error("Unexpected contextual response to vocabulary request");
      return result;
    },
  });
}

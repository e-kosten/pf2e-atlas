import { referenceQueryMeta } from "../queries/referenceQueries";
import { userErrorMessage } from "../feedback/errorMessages";
import { isSharedFailure } from "../feedback/errorMessages";
import { useQuery } from "@tanstack/react-query";
import { assertSearchIdentity } from "../search/searchIdentity";
import {
  discoverFilterCounts,
  discoverFilterEditor,
  discoverFilterValues,
} from "../../api/atlasApi";
import type {
  FilterDiscoveryContext,
  FilterCountsView,
  FilterEditorView,
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
    queryKey: [
      "filter-discovery",
      "editor",
      ...catalogIdentity,
      context.expected_artifact_generation,
      context.filter,
    ],
    enabled,
    staleTime: Infinity,
    meta: { ...referenceQueryMeta, catalogIdentity },
    placeholderData: (previous, previousQuery) =>
      JSON.stringify(previousQuery?.meta?.catalogIdentity) ===
      JSON.stringify(catalogIdentity)
        ? context.expected_artifact_generation &&
          previous?.artifact_generation !== context.expected_artifact_generation
          ? undefined
          : previous
            ? { ...previous, relationship_clauses: [] }
            : previous
        : undefined,
    queryFn: async ({ signal }) => {
      const response = await discoverFilterEditor(
        {
          selected_field_ids: [],
          filter: context.filter ?? undefined,
          expected_artifact_generation: context.expected_artifact_generation,
        },
        signal,
      );
      assertSearchIdentity(response, undefined, context.expected_artifact_generation);
      return response;
    },
  });
  return {
    filterEditor: editor.data,
    filterCountsEnabled: enabled,
    filterCountQueryKey: queryKeyPrefix,
    filterDiscoveryContext: context,
    vocabularyScope,
    loading: editor.isLoading,
    errorMessage:
      editor.error && !isSharedFailure(editor.error)
        ? userErrorMessage(editor.error, "Retry loading the available filters.")
        : null,
  };
}

/** Optional counts never own vocabulary or control whether a value is selectable. */
export function existingClauseId(
  context: FilterDiscoveryContext | undefined,
  clauseId: string | null,
): string | null {
  if (!clauseId) return null;
  const visit = (predicate: QueryPredicate | null): boolean => {
    if (!predicate) return false;
    if (predicate.clause_id === clauseId) return true;
    if (predicate.kind === "all_of" || predicate.kind === "any_of")
      return predicate.children.some(visit);
    if (predicate.kind === "exists" || predicate.kind === "not")
      return visit(predicate.predicate);
    return false;
  };
  return visit(context?.filter ?? null) ? clauseId : null;
}
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
  const addressedClause = existingClauseId(context, clauseId);
  const query = useQuery<FilterCountsView>({
    queryKey: [
      ...(queryKeyPrefix.length ? queryKeyPrefix : ["filter-discovery"]),
      "option-counts",
      context,
      fieldId,
      clauseId,
      literals,
      enabled,
    ],
    enabled:
      enabled &&
      !!context &&
      !context.text?.trim() &&
      literals.length > 0 &&
      (!fieldId.includes("[]") || !!addressedClause),
    meta: {
      ...referenceQueryMeta,
      fieldId,
      discoveryIdentity: context && discoveryIdentity({ context, clauseId: null }),
    },
    placeholderData: (previous, previousQuery) =>
      previousQuery?.meta?.fieldId === fieldId &&
      JSON.stringify(previousQuery.meta.discoveryIdentity) ===
        JSON.stringify(context && discoveryIdentity({ context, clauseId: null }))
        ? previous
        : undefined,
    retry: false,
    queryFn: async ({ signal }) => {
      if (!context) throw new Error("Missing filter count context");
      const response = await discoverFilterCounts(
        { context, field_id: fieldId, clause_id: addressedClause, values: literals },
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

function discoveryIdentity(value?: {
  context: FilterDiscoveryContext;
  clauseId: string | null;
}) {
  if (!value) return null;
  const { filter: _filter, ...identity } = value.context;
  void _filter;
  function containsLink(node: QueryPredicate | null): boolean {
    if (!node) return false;
    if (node.kind === "relationship") return true;
    if (node.kind === "all_of" || node.kind === "any_of")
      return node.children.some(containsLink);
    if (node.kind === "not" || node.kind === "exists")
      return containsLink(node.predicate);
    return false;
  }
  // Ordinary selection edits keep menu rows stable. Link eligibility includes
  // its Boolean placement, so it cannot reuse an earlier universe's counts.
  return containsLink(_filter) ? { ...identity, filter: _filter } : identity;
}

export const VOCABULARY_PAGE_SIZE = 40;
type OptionPage = {
  values: Pick<
    import("../../generated/atlas").QueryVocabulary,
    "field" | "options" | "selected_values" | "total_values"
  >;
  contextual: boolean;
};
export function useFilterVocabulary(
  scope: FilterVocabularyScope,
  fieldId: string,
  text: string,
  offset: number,
  selectedValues: QueryLiteral[],
  enabled = true,
  applicable?: { context: FilterDiscoveryContext; clauseId: string | null },
) {
  return useQuery<OptionPage>({
    meta: referenceQueryMeta,
    queryKey: [
      "filter-vocabulary",
      scope,
      fieldId,
      text,
      offset,
      selectedValues,
      applicable,
    ],
    enabled,
    // A selection edit must not temporarily remove or reorder the menu's rows.
    // Never carry suggestions across a different scope, field or page/search.
    placeholderData: (previous, previousQuery) =>
      JSON.stringify(previousQuery?.queryKey.slice(0, 5)) ===
        JSON.stringify(["filter-vocabulary", scope, fieldId, text, offset]) &&
      JSON.stringify(
        discoveryIdentity(previousQuery?.queryKey[6] as typeof applicable),
      ) === JSON.stringify(discoveryIdentity(applicable))
        ? previous
        : undefined,
    queryFn: async ({ signal }) => {
      const addressedClause = existingClauseId(
        applicable?.context,
        applicable?.clauseId ?? null,
      );
      if (
        applicable &&
        !applicable.context.text?.trim() &&
        (!fieldId.includes("[]") || addressedClause)
      ) {
        const result = await discoverFilterValues(
          {
            kind: "contextual",
            context: applicable.context,
            field_id: fieldId,
            clause_id: addressedClause,
            text: text.trim() || null,
            offset,
            limit: VOCABULARY_PAGE_SIZE,
            only_matching: true,
          },
          signal,
        );
        if (result.kind !== "contextual")
          throw new Error("Unexpected vocabulary response");
        assertSearchIdentity(
          result,
          applicable.context.search_profile,
          applicable.context.expected_artifact_generation,
        );
        if (result.outcome.status === "available")
          return {
            contextual: true,
            values: { ...result.outcome.value, selected_values: selectedValues },
          };
      }
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
      return { values: result.values, contextual: false };
    },
  });
}

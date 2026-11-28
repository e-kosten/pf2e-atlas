import { useState } from "react";
import { referenceQueryMeta } from "../queries/referenceQueries";
import { userErrorMessage } from "../feedback/errorMessages";
import { isSharedFailure } from "../feedback/errorMessages";
import {
  queryOptions,
  useQueries,
  useQuery,
  type UseQueryResult,
} from "@tanstack/react-query";
import { assertSearchIdentity } from "../search/searchIdentity";
import { encodeFilterUrl } from "./filterUrl";
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
// The service owns Kind membership. The browser only compares raw committed
// Kind clauses for focus continuity. Exact editor identity still gates commits.
function toolbarKindKey(filter: QueryPredicate | null): string {
  function containsKind(node: QueryPredicate): boolean {
    if ("field" in node) return node.field === "kind";
    if (node.kind === "all_of" || node.kind === "any_of")
      return node.children.some(containsKind);
    return node.kind === "not" && containsKind(node.predicate);
  }
  function terms(node: QueryPredicate): string[] {
    if (node.kind === "all_of") return node.children.flatMap(terms);
    return containsKind(node) ? [encodeFilterUrl(node)] : [];
  }
  return JSON.stringify(filter ? terms(filter) : []);
}

export function useFilterDiscovery({
  context,
  enabled = true,
  queryKeyPrefix,
  catalogIdentity = [],
  includeSearchToolbar = false,
  vocabularyScope = { kind: "artifact" },
}: {
  context: FilterDiscoveryContext;
  enabled?: boolean;
  queryKeyPrefix: readonly unknown[];
  catalogIdentity?: readonly unknown[];
  includeSearchToolbar?: boolean;
  vocabularyScope?: FilterVocabularyScope;
}) {
  const toolbarKey = includeSearchToolbar
    ? toolbarKindKey(context.filter ?? null)
    : null;
  const editor = useQuery<{ view: FilterEditorView; toolbarKey: string | null }>({
    queryKey: [
      "filter-discovery",
      "editor",
      includeSearchToolbar,
      ...catalogIdentity,
      context.expected_artifact_generation,
      context.filter,
    ],
    enabled,
    staleTime: Infinity,
    meta: { ...referenceQueryMeta, catalogIdentity, includeSearchToolbar },
    placeholderData: (previous, previousQuery) =>
      previousQuery?.meta?.includeSearchToolbar === includeSearchToolbar &&
      JSON.stringify(previousQuery?.meta?.catalogIdentity) ===
        JSON.stringify(catalogIdentity)
        ? context.expected_artifact_generation &&
          previous?.view.artifact_generation !== context.expected_artifact_generation
          ? undefined
          : previous
            ? { ...previous, view: { ...previous.view, relationship_clauses: [] } }
            : previous
        : undefined,
    queryFn: async ({ signal }) => {
      const response = await discoverFilterEditor(
        {
          selected_field_ids: [],
          include_search_toolbar: includeSearchToolbar || undefined,
          filter: context.filter ?? undefined,
          expected_artifact_generation: context.expected_artifact_generation,
        },
        signal,
      );
      assertSearchIdentity(response, undefined, context.expected_artifact_generation);
      return { view: response, toolbarKey };
    },
  });
  return {
    filterEditor: editor.data?.view,
    filterToolbarKindMatches:
      enabled &&
      !!editor.data?.view.search_toolbar &&
      editor.data.toolbarKey === toolbarKey,
    refreshFilterInformation: async () => {
      await editor.refetch({ throwOnError: true });
    },
    filterApplicabilityCurrent:
      enabled && editor.isSuccess && !editor.isPlaceholderData,
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
function optionCountQuery(
  context: FilterDiscoveryContext | undefined,
  fieldId: string,
  clauseId: string | null,
  literals: QueryLiteral[],
  enabled: boolean,
  queryKeyPrefix: readonly unknown[],
) {
  const addressedClause = existingClauseId(context, clauseId);
  return queryOptions({
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
}

function optionCountResult(
  query: UseQueryResult<FilterCountsView>,
  literals: QueryLiteral[],
  enabled: boolean,
) {
  const result = query.data?.counts;
  const counts =
    result?.status === "available" && !query.error && enabled
      ? new Map(
          result.value.options.map((option) => [
            String(option.value),
            option.distinct_roots,
          ]),
        )
      : new Map<string, number>();
  // The previous response can still annotate its rows while a new batch arrives.
  const pendingValues =
    enabled && query.isFetching && (query.isPending || query.isPlaceholderData)
      ? new Set(literals.map(String).filter((value) => !counts.has(value)))
      : undefined;
  return { counts, pendingValues };
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
  const query = useQuery({
    ...optionCountQuery(context, fieldId, clauseId, literals, enabled, queryKeyPrefix),
    placeholderData: (previous, previousQuery) =>
      previousQuery?.meta?.fieldId === fieldId &&
      JSON.stringify(previousQuery.meta.discoveryIdentity) ===
        JSON.stringify(context && discoveryIdentity({ context, clauseId: null }))
        ? previous
        : undefined,
  });
  return optionCountResult(query, literals, enabled);
}

/** Fixed catalog batches preserve known zeros when more groups are opened. */
export function useKindOverflowCounts(
  context: FilterDiscoveryContext | undefined,
  clauseId: string | null,
  catalogValues: string[],
  displayedValues: string[],
  literalLimit: number,
  queryKeyPrefix: readonly unknown[] = [],
) {
  const batches: string[][] = [];
  if (literalLimit > 0 && displayedValues.length) {
    const catalog = [...new Set(catalogValues)];
    const displayed = new Set(displayedValues);
    for (let offset = 0; offset < catalog.length; offset += literalLimit) {
      const batch = catalog.slice(offset, offset + literalLimit);
      if (batch.some((value) => displayed.has(value))) batches.push(batch);
    }
  }
  // A saved list's request key includes the edited filter. Its discovery
  // identity already includes the list, without tying placeholders to that edit.
  const identity = batches.length
    ? JSON.stringify([
        context?.kind === "saved_list" ? [] : queryKeyPrefix,
        context && discoveryIdentity({ context, clauseId: null }),
      ])
    : null;
  const [retained, setRetained] = useState<{
    identity: string | null;
    batches: ReadonlyMap<string, FilterCountsView>;
  }>();
  const previous = retained?.identity === identity ? retained.batches : undefined;
  const batchKeys = batches.map((batch) => JSON.stringify(batch));
  const replies = useQueries({
    queries: batches.map((batch, index) => ({
      ...optionCountQuery(context, "kind", clauseId, batch, true, queryKeyPrefix),
      placeholderData: previous?.get(batchKeys[index]),
    })),
  });
  const counts = new Map<string, number>();
  const pendingValues = new Set<string>();
  let settled: Map<string, FilterCountsView> | undefined;
  for (const [index, reply] of replies.entries()) {
    const result = optionCountResult(reply, batches[index], true);
    for (const [value, count] of result.counts) counts.set(value, count);
    for (const value of result.pendingValues ?? []) pendingValues.add(value);
    if (reply.isPending || reply.isPlaceholderData) continue;
    const key = batchKeys[index];
    const data = !reply.error ? reply.data : undefined;
    if (data !== previous?.get(key)) {
      settled ??= new Map(previous);
      if (data) settled.set(key, data);
      else settled.delete(key);
    }
  }
  // useQueries has no previous-query placeholder callback. Only retain settled
  // data within the same discovery scope, as the single-batch picker does.
  if (settled) setRetained({ identity, batches: settled });
  return { counts, pendingValues };
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
  return containsLink(_filter ?? null) ? { ...identity, filter: _filter } : identity;
}

export const VOCABULARY_PAGE_SIZE = 40;
type OptionPage = {
  values: Pick<
    import("../../generated/atlas").FilterVocabularyView,
    "field" | "options" | "selected_values" | "total_values"
  >;
  generation: string;
  contextual: boolean;
  pageKey: string;
  offset: number;
  counts?: ReadonlyMap<string, number>;
};
function searchPageKey(text: string, offset: number) {
  return JSON.stringify([text, offset]);
}
export function useFilterVocabulary(
  scope: FilterVocabularyScope,
  fieldId: string,
  text: string,
  offset: number,
  selectedValues: QueryLiteral[],
  enabled = true,
  applicable?: { context: FilterDiscoveryContext; clauseId: string | null },
  pageGeneration?: string | null,
  restart = 0,
) {
  const expected =
    pageGeneration === undefined
      ? applicable?.context.expected_artifact_generation
      : pageGeneration;
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
      expected,
      restart,
    ],
    enabled,
    retry: false,
    // A selection edit must not temporarily remove or reorder the menu's rows.
    // Keep the list stable while a replacement page loads.
    // Never carry suggestions across a different scope, field or search.
    placeholderData: (previous, previousQuery) =>
      previousQuery?.queryKey[8] === restart &&
      (!expected || previous?.generation === expected) &&
      JSON.stringify(previousQuery?.queryKey.slice(0, 4)) ===
        JSON.stringify(["filter-vocabulary", scope, fieldId, text]) &&
      JSON.stringify(
        discoveryIdentity(previousQuery?.queryKey[6] as typeof applicable),
      ) === JSON.stringify(discoveryIdentity(applicable))
        ? previous
        : undefined,
    queryFn: async ({ signal }) => {
      let generation = expected;
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
            context: { ...applicable.context, expected_artifact_generation: expected },
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
        assertSearchIdentity(result, applicable.context.search_profile, expected);
        generation = result.artifact_generation;
        if (result.outcome.status === "available")
          return {
            generation: result.artifact_generation,
            contextual: true,
            pageKey: searchPageKey(text, offset),
            offset,
            counts: new Map(
              result.outcome.value.options.map((option) => [
                String(option.value),
                option.distinct_roots,
              ]),
            ),
            values: { ...result.outcome.value, selected_values: selectedValues },
          };
      }
      const result = await discoverFilterValues(
        {
          kind: "vocabulary",
          expected_artifact_generation: generation ?? undefined,
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
      assertSearchIdentity(result, undefined, generation);
      return {
        generation: result.artifact_generation,
        values: result.values,
        contextual: false,
        pageKey: searchPageKey(text, offset),
        offset,
      };
    },
  });
}

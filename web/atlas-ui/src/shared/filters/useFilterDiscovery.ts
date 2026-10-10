import { useMemo } from "react";
import {
  keepPreviousData,
  useQueries,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import {
  discoverFilterEditor,
  discoverFilterValues,
  discoverFilterCounts,
} from "../../api/atlasApi";
import type {
  FilterDiscoveryContext,
  FilterEditorView,
  FilterValueListView,
} from "../../generated/atlas";

export type UseFilterDiscoveryInput = {
  context: FilterDiscoveryContext;
  enabled?: boolean;
  hiddenFieldIds: string[];
  queryKeyPrefix: readonly unknown[];
  retainedValueQueryKeyPrefix?: readonly unknown[];
  selectedFieldIds: string[];
  visibleFieldIds: string[];
};

export type UseFilterDiscoveryResult = {
  filterEditor: FilterEditorView | undefined;
  filterValuesByField: Record<string, FilterValueListView | undefined>;
  filterCountsByField: Record<
    string,
    import("../../generated/atlas").FilterCountsView | undefined
  >;
  loading: boolean;
  errorMessage: string | null;
};

export function useFilterDiscovery({
  context,
  enabled = true,
  hiddenFieldIds,
  queryKeyPrefix,
  retainedValueQueryKeyPrefix = queryKeyPrefix,
  selectedFieldIds,
  visibleFieldIds,
}: UseFilterDiscoveryInput): UseFilterDiscoveryResult {
  const queryClient = useQueryClient();
  const filterEditorQuery = useQuery({
    queryKey: [...queryKeyPrefix, "editor", selectedFieldIds],
    enabled,
    placeholderData: keepPreviousData,
    queryFn: () =>
      discoverFilterEditor({
        context,
        selected_field_ids: selectedFieldIds,
      }),
  });

  const valueFieldIds = useMemo(() => {
    const fields = (filterEditorQuery.data?.groups ?? []).flatMap(
      (group) => group.fields,
    );
    const visibleFields = new Set(visibleFieldIds);
    const hiddenFields = new Set(hiddenFieldIds);
    return fields
      .filter(
        (field) =>
          (field.definition.value_discovery === "open_values" ||
            field.definition.value_discovery === "closed_choices" ||
            field.definition.value_discovery === "boolean_counts") &&
          (visibleFields.has(field.definition.id) ||
            (field.placement === "initially_visible" &&
              !hiddenFields.has(field.definition.id))),
      )
      .map((field) => field.definition.id);
  }, [filterEditorQuery.data, hiddenFieldIds, visibleFieldIds]);

  const definitions = (filterEditorQuery.data?.groups ?? []).flatMap((g) =>
    g.fields.map((f) => f.definition),
  );
  const requestsFor = (fieldId: string) => {
    const field = definitions.find((f) => f.id === fieldId);
    const clauses = field ? findClauseIds(context.filter, field.path, field.scope) : [];
    return (clauses.length ? clauses : [null]).map((clauseId) => ({
      fieldId,
      clauseId,
    }));
  };
  const countFields = definitions.filter(
    (f) => f.value_discovery === "numeric_statistics" && visibleFieldIds.includes(f.id),
  );
  const countRequests = countFields.flatMap((field) => requestsFor(field.id));
  const valueRequests = valueFieldIds.flatMap(requestsFor);
  const countQueries = useQueries({
    queries: countRequests.map(({ fieldId, clauseId }) => ({
      queryKey: [...queryKeyPrefix, "counts", fieldId, clauseId],
      enabled: enabled && !filterEditorQuery.isPlaceholderData,
      queryFn: () =>
        discoverFilterCounts({
          context,
          field_id: fieldId,
          clause_id: clauseId,
        }),
    })),
  });
  const filterValueQueries = useQueries({
    queries: valueRequests.map(({ fieldId, clauseId }) => ({
      queryKey: [...queryKeyPrefix, "values", fieldId, clauseId],
      enabled: enabled && !filterEditorQuery.isPlaceholderData,
      placeholderData: () =>
        retainedFilterValue(
          queryClient,
          retainedValueQueryKeyPrefix,
          fieldId,
          clauseId,
        ),
      queryFn: () =>
        discoverFilterValues({
          context,
          field_id: fieldId,
          clause_id: clauseId,
          text: null,
          offset: 0,
          limit: 100,
        }),
    })),
  });

  const filterValuesByField = Object.fromEntries(
    valueRequests.map(({ fieldId, clauseId }, index) => [
      clauseId ? `${fieldId}:${clauseId}` : fieldId,
      filterValueQueries[index]?.data,
    ]),
  );

  const errorMessage =
    countQueries.find((q) => q.error)?.error?.message ??
    filterEditorQuery.error?.message ??
    filterValueQueries.find((query) => query.error)?.error?.message ??
    null;

  return {
    filterEditor: filterEditorQuery.data,
    filterValuesByField,
    filterCountsByField: Object.fromEntries(
      countRequests.map(({ fieldId, clauseId }, i) => [
        clauseId ? `${fieldId}:${clauseId}` : fieldId,
        countQueries[i]?.data,
      ]),
    ),
    loading:
      filterEditorQuery.isLoading ||
      filterEditorQuery.isFetching ||
      filterValueQueries.some((query) => query.isLoading || query.isFetching),
    errorMessage,
  };
}

function retainedFilterValue(
  queryClient: ReturnType<typeof useQueryClient>,
  queryKeyPrefix: readonly unknown[],
  fieldId: string,
  clauseId: string | null,
): FilterValueListView | undefined {
  const candidates = queryClient.getQueriesData<FilterValueListView>({
    queryKey: [...queryKeyPrefix],
  });
  return (
    candidates.find(
      ([key, data]) =>
        key[key.length - 1] === clauseId && data?.values?.field === fieldId,
    )?.[1] ??
    candidates.find(
      ([key, data]) => key[key.length - 1] === null && data?.values?.field === fieldId,
    )?.[1]
  );
}

export function findClauseIds(
  predicate: import("../../generated/atlas").QueryPredicate | null,
  field: string,
  scope: string | null = null,
  currentScope: string | null = null,
): (string | null)[] {
  if (!predicate) return [];
  if ("field" in predicate && predicate.field === field && currentScope === scope)
    return [predicate.clause_id || null];
  if (predicate.kind === "all_of" || predicate.kind === "any_of") {
    return predicate.children.flatMap((child) =>
      findClauseIds(child, field, scope, currentScope),
    );
  }
  if (predicate.kind === "exists")
    return findClauseIds(predicate.predicate, field, scope, predicate.collection);
  if (predicate.kind === "not")
    return findClauseIds(predicate.predicate, field, scope, currentScope);
  return [];
}

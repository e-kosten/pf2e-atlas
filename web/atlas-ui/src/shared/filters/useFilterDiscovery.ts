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
          field.applicability === "applicable" &&
          field.supports_counts &&
          (field.placement === "always_visible" ||
            visibleFields.has(field.id) ||
            (field.placement === "initially_visible" && !hiddenFields.has(field.id))),
      )
      .map((field) => field.id);
  }, [filterEditorQuery.data, hiddenFieldIds, visibleFieldIds]);

  const filterValueQueries = useQueries({
    queries: valueFieldIds.map((fieldId) => ({
      queryKey: [...queryKeyPrefix, "values", fieldId],
      enabled: enabled && !filterEditorQuery.isPlaceholderData,
      placeholderData: () =>
        retainedFilterValue(queryClient, retainedValueQueryKeyPrefix, fieldId),
      queryFn: () =>
        discoverFilterValues({
          context,
          field_id: fieldId,
        }),
    })),
  });

  const filterValuesByField = useMemo(() => {
    const pairs = valueFieldIds.map((fieldId, index) => [
      fieldId,
      filterValueQueries[index]?.data,
    ]);
    return Object.fromEntries(pairs);
  }, [filterValueQueries, valueFieldIds]);

  const errorMessage =
    filterEditorQuery.error?.message ??
    filterValueQueries.find((query) => query.error)?.error?.message ??
    null;

  return {
    filterEditor: filterEditorQuery.data,
    filterValuesByField,
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
): FilterValueListView | undefined {
  return queryClient
    .getQueriesData<FilterValueListView>({ queryKey: [...queryKeyPrefix] })
    .map(([, data]) => data)
    .find((data) => data?.field_id === fieldId);
}

import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { ExternalLink } from "lucide-react";
import { useMemo, useState } from "react";
import { filterSavedList, removeSavedListItem } from "../../api/atlasApi";
import {
  recordPath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
  type AtlasRoute,
} from "../../app/routes";
import {
  buildBasicFilter,
  DEFAULT_SEARCH_STATE,
  encodeSearchExecutionState,
  type SearchFormState,
} from "../../shared/filters/searchState";
import { WorkspaceLayout } from "../../shared/layout/WorkspaceLayout";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { RecordPreviewScope } from "../../shared/records/RecordPreviewScope";
import { useRecordDetail } from "../../shared/records/useRecordDetail";
import { PaneIconLink } from "../../shared/ui/actions/PaneAction";
import { ListInfoPane } from "./ListInfoPane";
import { ListItemsPane } from "./ListItemsPane";
import { useSavedLists } from "./savedListQueries";
import {
  listSearchQuery,
  useDebouncedSearchFilters,
  useSavedListFilterDiscovery,
} from "./useSavedListFilters";

type ListDetailViewProps = {
  route: Extract<AtlasRoute, { kind: "list" }>;
};

const LIST_WORKSPACE_WIDTH_SPECS = {
  filter: { defaultWidth: 280, minWidth: 240 },
  results: { defaultWidth: 420, minWidth: 280 },
  detail: { defaultWidth: 640, minWidth: 360 },
};

export function ListDetailView({ route }: ListDetailViewProps) {
  const queryClient = useQueryClient();
  const lists = useSavedLists();
  const [filters, setFilters] = useState<SearchFormState>(DEFAULT_SEARCH_STATE);
  const activeFilters = useDebouncedSearchFilters(filters);
  const filterToken = useMemo(
    () => encodeSearchExecutionState(activeFilters),
    [activeFilters],
  );
  const filterDiscovery = useSavedListFilterDiscovery(route.slug, filters);
  const list = useQuery({
    queryKey: ["saved-list", route.slug, filterToken],
    placeholderData: keepPreviousData,
    queryFn: () => {
      const query = listSearchQuery(activeFilters);
      return filterSavedList({
        list_ref: route.slug,
        ...(query ? { query } : {}),
        filter: buildBasicFilter(activeFilters),
      });
    },
  });
  const selectedItem = list.data?.items.find(
    (item) => item.record_key === route.selectedRecordKey,
  );
  const detail = useRecordDetail(
    selectedItem?.status === "unresolved" ? null : route.selectedRecordKey,
  );
  const removeItem = useMutation({
    mutationFn: (recordKey: string) =>
      removeSavedListItem({ list_ref: route.slug, record_ref: recordKey }),
    onSuccess: async (_view, recordKey) => {
      await queryClient.invalidateQueries({ queryKey: ["saved-list", route.slug] });
      await queryClient.invalidateQueries({ queryKey: ["saved-lists"] });
      if (recordKey === route.selectedRecordKey) {
        navigateToAtlasRoute({
          kind: "list",
          slug: route.slug,
          selectedRecordKey: null,
        });
      }
    },
  });

  return (
    <>
      <WorkspaceLayout
        filter={
          <ListInfoPane
            currentSlug={route.slug}
            filterState={{
              search: filters,
              setSearch: setFilters,
              filterEditor: filterDiscovery.filterEditor,
              filterValuesByField: filterDiscovery.filterValuesByField,
              filterDiscoveryLoading: filterDiscovery.loading,
              errorMessage: filterDiscovery.errorMessage,
            }}
            list={list.data?.list}
            lists={lists.data?.lists ?? []}
            loading={list.isLoading}
            listsLoading={lists.isLoading || lists.isFetching}
            onSelectList={(slug) =>
              navigateToAtlasRoute({ kind: "list", slug, selectedRecordKey: null })
            }
          />
        }
        results={
          <ListItemsPane
            items={list.data?.items ?? []}
            loading={list.isLoading || list.isFetching}
            removingKey={
              removeItem.isPending && typeof removeItem.variables === "string"
                ? removeItem.variables
                : null
            }
            selectedRecordKey={route.selectedRecordKey}
            onRemove={(recordKey) => removeItem.mutate(recordKey)}
            onSelect={(recordKey) =>
              navigateToAtlasRoute({
                kind: "list",
                slug: route.slug,
                selectedRecordKey: recordKey,
              })
            }
          />
        }
        selectedRecordKey={route.selectedRecordKey}
        labels={{ filter: "List", results: "Items", detail: "Detail" }}
        sizing="detail-focus"
        widthSpecs={LIST_WORKSPACE_WIDTH_SPECS}
        detailHeaderActions={
          route.selectedRecordKey ? (
            <PaneIconLink
              href={recordPath(route.selectedRecordKey)}
              icon={<ExternalLink size={16} />}
              label="Open full page"
              onClick={(event) => {
                if (!shouldHandleAtlasRouteClick(event)) {
                  return;
                }
                event.preventDefault();
                navigateToAtlasRoute({
                  kind: "record",
                  recordKey: route.selectedRecordKey!,
                });
              }}
            />
          ) : null
        }
        detail={
          <RecordPreviewScope
            onOpenFullPage={(recordKey, childLocator) =>
              navigateToAtlasRoute({ kind: "record", recordKey, childLocator })
            }
          >
            <RecordDetailPane
              detail={selectedItem?.status === "unresolved" ? undefined : detail.data}
              emptyMessage={
                selectedItem?.status === "unresolved"
                  ? "This saved record is unresolved."
                  : undefined
              }
              errors={[list.error, detail.error, removeItem.error]}
              loading={
                selectedItem?.status === "unresolved"
                  ? false
                  : detail.isLoading || detail.isFetching
              }
              onReference={(recordKey, childLocator) =>
                navigateToAtlasRoute({ kind: "record", recordKey, childLocator })
              }
            />
          </RecordPreviewScope>
        }
      />
    </>
  );
}

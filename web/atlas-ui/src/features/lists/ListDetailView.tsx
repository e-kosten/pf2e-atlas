import type { ListPageInput } from "../../app/destinations";
import { navigateDestination, destinationHref } from "../../app/navigation";
import { listDestination } from "../../app/destinations";
import { atlasRouter } from "../../app/router";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ListEntryNote } from "./ListEntryNote";
import { ListRecordPicker } from "./ListRecordPicker";
import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import {
  filterSavedList,
  removeSavedListItem,
  reorderSavedListItem,
} from "../../api/atlasApi";
import {
  buildFilter,
  decodeSearchStateFromParams,
  effectiveResultSort,
  encodeSearchExecutionState,
  searchStateQueryString,
  type SearchFormState,
} from "../../shared/filters/searchState";
import { BrowseReader } from "../../shared/records/BrowseReader";
import { useFilterUrlRecovery } from "../../shared/filters/useFilterUrlRecovery";
import {
  rootNavigation,
  useReferenceReading,
  type ReadingRequest,
} from "../../shared/records/useReferenceReading";
import { ListBrowseHeader } from "./ListBrowseHeader";
import { ListItemsPane } from "./ListItemsPane";
import { invalidateSavedListQueries } from "./savedListQueries";
import { sameResultSort } from "../../shared/search/searchIdentity";
import { ResultSortControl } from "../../shared/search/ResultSortControl";
import {
  listSearchQuery,
  useDebouncedSearchFilters,
  useSavedListFilterDiscovery,
} from "./useSavedListFilters";
type ListDetailViewProps = {
  route: ListPageInput;
};
export function ListDetailView({ route }: ListDetailViewProps) {
  const queryClient = useQueryClient();
  const [addOpen, setAddOpen] = useState(false);
  const [filters, setFilters] = useState<SearchFormState>(() =>
    decodeSearchStateFromParams(new URLSearchParams(route.query), "saved_list"),
  );
  const previousQuery = useRef(route.query);
  useEffect(() => {
    if (previousQuery.current === route.query) return;
    previousQuery.current = route.query;
    setFilters(
      decodeSearchStateFromParams(new URLSearchParams(route.query), "saved_list"),
    );
  }, [route.query]);
  useFilterUrlRecovery(filters);
  function changeFilters(next: SearchFormState) {
    const query = searchStateQueryString(next);
    previousQuery.current = query || undefined;
    setFilters(next);
    navigateDestination(listDestination({ ...route, query }));
  }
  const listQuery = searchStateQueryString(filters);
  const reading = useReferenceReading(null, !filters.filterWasReset);
  const readingNavigation = useRef(false);
  const activeFilters = useDebouncedSearchFilters(filters);
  const filterToken = useMemo(
    () => encodeSearchExecutionState(activeFilters),
    [activeFilters],
  );
  const filterDiscovery = useSavedListFilterDiscovery(route.slug, filters);
  const list = useQuery({
    queryKey: ["saved-list", route.slug, filterToken],
    placeholderData: keepPreviousData,
    enabled:
      !filters.filterWasReset &&
      !activeFilters.filterWasReset &&
      !activeFilters.filterError,
    queryFn: async () => {
      const query = listSearchQuery(activeFilters);
      const view = await filterSavedList({
        list_ref: route.slug,
        ...(activeFilters.sort ? { sort: activeFilters.sort } : {}),
        ...(query ? { query } : {}),
        ...(buildFilter(activeFilters) ? { filter: buildFilter(activeFilters)! } : {}),
      });
      if (!sameResultSort(view.sort, effectiveResultSort(activeFilters, "saved_list")))
        throw new Error("The saved records used a different order. Retry the list.");
      return view;
    },
  });
  const selectedItem = list.data?.items.find(
    (item) => item.record_key === route.selectedRecordKey,
  );
  usePageTitle(list.data?.list.name ?? "List");
  const inspectSelection = reading.select;
  const selectedRoute = useRef<
    | {
        member: string;
        address: string;
        blocked: boolean;
      }
    | undefined
  >(undefined);
  useEffect(() => {
    const member = JSON.stringify([route.slug, route.selectedRecordKey]);
    const address = JSON.stringify([route.selection, route.landing]);
    const previous = selectedRoute.current;
    const changed = previous?.member !== member;
    const addressChanged = previous?.address !== address;
    const unavailable = selectedItem?.status === "unresolved";
    const recovered = !changed && previous.blocked && selectedItem?.status === "active";
    const blocked = unavailable || (!changed && previous.blocked && !recovered);
    selectedRoute.current = {
      member,
      address,
      // An omitted filtered member supplies no new availability information.
      blocked,
    };
    if (!changed && !addressChanged && !unavailable && !recovered) return;
    const request =
      !filters.filterWasReset && route.selectedRecordKey && !blocked
        ? {
            navigation: route.selection ?? rootNavigation(route.selectedRecordKey),
            landing: route.landing,
          }
        : null;
    if (JSON.stringify(reading.request) !== JSON.stringify(request))
      inspectSelection(request);
  }, [
    filters.filterWasReset,
    route.slug,
    route.selectedRecordKey,
    route.selection,
    route.landing,
    selectedItem?.status,
    inspectSelection,
    reading.request,
  ]);
  useEffect(() => {
    if (!readingNavigation.current) return;
    readingNavigation.current = false;
    void atlasRouter.navigate({
      ...listDestination({
        ...route,
        selectedRecordKey: reading.request ? route.selectedRecordKey : null,
        query: listQuery,
        selection: reading.request?.navigation,
        landing: reading.request?.landing,
      }),
      replace: true,
      resetScroll: false,
      state: true,
    });
  }, [reading.request, route, listQuery]);
  const addressableReading = {
    ...reading,
    select: (request: ReadingRequest | null) => {
      readingNavigation.current = true;
      reading.select(request);
    },
    inspect: (request: ReadingRequest | null) => {
      readingNavigation.current = true;
      reading.inspect(request);
    },
    back: () => {
      readingNavigation.current = true;
      reading.back();
    },
    changeRank: (navigation: ReadingRequest["navigation"]) => {
      readingNavigation.current = true;
      reading.changeRank(navigation);
    },
  };
  const removeItem = useMutation({
    mutationFn: (recordKey: string) =>
      removeSavedListItem({ list_ref: route.slug, record_ref: recordKey }),
    onSuccess: async (view, recordKey) => {
      await invalidateSavedListQueries(queryClient, [
        route.slug,
        view.slug,
        view.list_key,
      ]);
      if (recordKey === route.selectedRecordKey) {
        navigateDestination(
          listDestination({
            slug: route.slug,
            selectedRecordKey: null,
            query: listQuery,
          }),
        );
      }
    },
  });
  const reorderItem = useMutation({
    mutationFn: (input: {
      recordKey: string;
      targetRecordKey: string;
      placement: "before" | "after";
    }) =>
      reorderSavedListItem({
        list_ref: route.slug,
        record_ref: input.recordKey,
        target_record_ref: input.targetRecordKey,
        placement: input.placement,
      }),
    onSuccess: async (view) =>
      invalidateSavedListQueries(queryClient, [route.slug, view.slug, view.list_key]),
  });
  const canReorder =
    effectiveResultSort(filters, "saved_list").kind === "saved_order" &&
    effectiveResultSort(activeFilters, "saved_list").kind === "saved_order" &&
    list.data?.sort.kind === "saved_order" &&
    !filters.filterError &&
    !activeFilters.filterError &&
    !filters.query.trim() &&
    !activeFilters.query.trim() &&
    !buildFilter(filters) &&
    !buildFilter(activeFilters);
  return (
    <main className="list-workspace">
      <ListBrowseHeader
        error={list.error}
        onRetry={() => void list.refetch()}
        filterState={{
          search: filters,
          setSearch: changeFilters,
          filterEditor: filterDiscovery.filterEditor,
          filterCountsEnabled: filterDiscovery.filterCountsEnabled,
          filterCountQueryKey: filterDiscovery.filterCountQueryKey,
          filterDiscoveryContext: filterDiscovery.filterDiscoveryContext,
          vocabularyScope: filterDiscovery.vocabularyScope,
          filterDiscoveryLoading: filterDiscovery.loading,
          filterDiscoveryError: filterDiscovery.errorMessage,
        }}
        list={list.data?.list}
        loading={list.isLoading}
        onAddRecords={() => setAddOpen(true)}
      />
      <ListRecordPicker
        key={route.slug}
        list={list.data?.list}
        open={addOpen}
        onClose={() => setAddOpen(false)}
      />
      <BrowseReader
        reading={addressableReading}
        selectedRecordKey={route.selectedRecordKey}
        readerFooter={
          route.selectedRecordKey &&
          list.data &&
          (list.data.list.slug === route.slug ||
            list.data.list.list_key === route.slug) ? (
            <ListEntryNote
              key={JSON.stringify([list.data.list.list_key, route.selectedRecordKey])}
              listRef={list.data.list.list_key}
              recordKey={route.selectedRecordKey}
              initialNote={
                selectedItem
                  ? {
                      list_key: list.data.list.list_key,
                      slug: list.data.list.slug,
                      record_key: selectedItem.record_key,
                      title: selectedItem.snapshot.title,
                      note: selectedItem.note ?? null,
                    }
                  : undefined
              }
              initialNoteUpdatedAt={list.dataUpdatedAt}
              hidden={Boolean(
                route.selection &&
                route.selection.record_key !== route.selectedRecordKey,
              )}
            />
          ) : undefined
        }
        resultsLabel="Saved records"
        resultsIdentity={list.data?.items}
        resultsLoading={list.isLoading || list.isFetching}
        resultsHeader={
          <div className="result-pane-header">
            <span>
              {list.data
                ? `${list.data.items.length.toLocaleString()} saved records`
                : "Saved records"}
            </span>
            <ResultSortControl
              search={filters}
              surface="saved_list"
              setSearch={changeFilters}
            />
          </div>
        }
      >
        <ListItemsPane
          error={removeItem.error}
          reorderError={reorderItem.error}
          mutating={
            reorderItem.isPending ||
            removeItem.isPending ||
            list.isPlaceholderData ||
            Boolean(list.error)
          }
          items={list.data?.items ?? []}
          loading={list.isLoading || list.isFetching}
          unavailable={Boolean(list.error && !list.data)}
          filtered={Boolean(filters.query.trim() || buildFilter(filters))}
          removingKey={removeItem.isPending ? removeItem.variables : null}
          selectedRecordKey={route.selectedRecordKey}
          itemHref={(recordKey) =>
            destinationHref(
              listDestination({
                slug: route.slug,
                selectedRecordKey: recordKey,
                query: listQuery,
              }),
            )
          }
          onRemove={(key) => removeItem.mutateAsync(key)}
          onReorder={
            canReorder
              ? async (recordKey, targetRecordKey, placement) => {
                  await reorderItem.mutateAsync({
                    recordKey,
                    targetRecordKey,
                    placement,
                  });
                  return true;
                }
              : undefined
          }
          onSelect={(recordKey) => {
            reading.select({ navigation: rootNavigation(recordKey) });
            navigateDestination(
              listDestination({
                slug: route.slug,
                selectedRecordKey: recordKey,
                query: listQuery,
              }),
            );
          }}
        />
      </BrowseReader>
    </main>
  );
}

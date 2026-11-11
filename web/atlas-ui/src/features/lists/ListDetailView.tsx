import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AddToRecordButton } from "../record-actions/AddToRecordButton";
import { FilterControls } from "../../shared/filters/FilterPanel";
import type { ReorderPlacement } from "../../generated/atlas";
import { ReferenceHelpHost } from "../../shared/records/ReferenceHelpHost";
import { Alert, Grid } from "antd";
import type { ListPageInput } from "../../app/destinations";
import { navigateDestination, destinationHref } from "../../app/navigation";
import { listDestination, listsDestination } from "../../app/destinations";
import { atlasRouter } from "../../app/router";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { usePreparedReferenceSelection } from "../../shared/records/usePreparedReferenceSelection";
import {
  ReferenceSelectionStatus,
  hasReferenceSelectionStatus,
} from "../../shared/records/ReferenceSelectionStatus";
import { ListEntryNoteEditor, useListEntryNote } from "./ListEntryNote";
import { EditListModal } from "./EditListModal";
import { useListSettingsOpening } from "./useListSettingsOpening";
import { SettingsOpeningError } from "../../shared/ui/overlays/SettingsOpeningError";
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
  onCloseDirectEditor?: (slug: string) => void;
};
export function ListDetailView({ route, onCloseDirectEditor }: ListDetailViewProps) {
  const queryClient = useQueryClient();
  const narrow = !Grid.useBreakpoint().lg;
  const [addOpen, setAddOpen] = useState(false);
  const paused = useSavedWritesPaused();
  const editing = useListSettingsOpening(onCloseDirectEditor ? route.slug : undefined);
  const [resolvedListKey, setResolvedListKey] = useState<string>();
  const listRef = resolvedListKey ?? route.slug;
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
  const selection = usePreparedReferenceSelection(
    reading,
    !filters.filterWasReset,
    JSON.stringify([
      route.slug,
      route.selectedRecordKey,
      route.selection,
      route.landing,
      listQuery,
    ]),
    null,
  );
  const readingNavigation = useRef(false);
  const activeFilters = useDebouncedSearchFilters(filters);
  const filterToken = useMemo(
    () => encodeSearchExecutionState(activeFilters),
    [activeFilters],
  );
  const filterDiscovery = useSavedListFilterDiscovery(
    listRef,
    filters,
    !!resolvedListKey,
  );
  const list = useQuery({
    queryKey: ["saved-list", listRef, filterToken],
    placeholderData: keepPreviousData,
    refetchOnMount: resolvedListKey ? undefined : "always",
    enabled:
      !filters.filterWasReset &&
      !activeFilters.filterWasReset &&
      !activeFilters.filterError,
    queryFn: async ({ signal }) => {
      const query = listSearchQuery(activeFilters);
      const view = await filterSavedList(
        listRef,
        {
          ...(activeFilters.sort ? { sort: activeFilters.sort } : {}),
          ...(query ? { query } : {}),
          ...(buildFilter(activeFilters)
            ? { filter: buildFilter(activeFilters)! }
            : {}),
        },
        signal,
      );
      if (!sameResultSort(view.sort, effectiveResultSort(activeFilters, "saved_list")))
        throw new Error("The saved records used a different order. Retry the list.");
      return view;
    },
  });
  if (
    !resolvedListKey &&
    list.isFetchedAfterMount &&
    list.isSuccess &&
    !list.isPlaceholderData
  ) {
    setResolvedListKey(list.data.list.list_key);
  }
  const listView = resolvedListKey ? list.data : undefined;
  const listIdentity = listView?.list;
  const selectedItem = listView?.items.find(
    (item) => item.record_key === route.selectedRecordKey,
  );
  const listLoading = list.isLoading || (!resolvedListKey && list.isFetching);
  const selectedRecordKey = route.selectedRecordKey;
  const noteHidden =
    !!route.selection && route.selection.record_key !== selectedRecordKey;
  const entryNote = useListEntryNote({
    selected:
      selectedRecordKey && listIdentity && listIdentity.list_key === resolvedListKey
        ? {
            listRef: listIdentity.list_key,
            recordKey: selectedRecordKey,
            initialNote: selectedItem
              ? {
                  list_key: listIdentity.list_key,
                  slug: listIdentity.slug,
                  record_key: selectedItem.record_key,
                  title: selectedItem.snapshot.title,
                  note: selectedItem.note ?? null,
                }
              : undefined,
            initialNoteUpdatedAt: list.dataUpdatedAt,
          }
        : undefined,
    selectedRecordKey,
    listRouteRef: route.slug,
  });
  usePageTitle(listIdentity?.name ?? "List");
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
    const unavailable =
      selectedItem?.status === "unresolved" || selectedItem?.status === "unchecked";
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
      removeSavedListItem(listRef, { record_ref: recordKey }),
    onSuccess: async (view, recordKey) => {
      if (recordKey === route.selectedRecordKey) {
        entryNote.retainAfterRemoval(recordKey);
        selection.cancel();
        reading.select(null);
        navigateDestination(
          listDestination({
            slug: route.slug,
            selectedRecordKey: null,
            query: listQuery,
          }),
        );
      }
      await invalidateSavedListQueries(queryClient, [
        route.slug,
        view.slug,
        view.list_key,
      ]);
    },
  });
  const reorderItem = useMutation({
    mutationFn: (input: {
      recordKey: string;
      targetRecordKey: string;
      placement: ReorderPlacement;
    }) =>
      reorderSavedListItem(listRef, {
        record_ref: input.recordKey,
        target_record_ref: input.targetRecordKey,
        placement: input.placement,
      }),
    onSuccess: async (view) =>
      invalidateSavedListQueries(queryClient, [route.slug, view.slug, view.list_key]),
  });
  const narrowed = Boolean(listSearchQuery(filters) || buildFilter(filters));
  const pendingCount =
    filterToken !== encodeSearchExecutionState(filters) ||
    list.isFetching ||
    list.isPlaceholderData;
  const resultCount =
    filters.filterError || list.error || listView?.reference_error
      ? "Count unavailable"
      : pendingCount || !listView
        ? "Updating records…"
        : narrowed
          ? `${listView.items.length.toLocaleString()} of ${listView.list.item_count.toLocaleString()} records`
          : listView.list.item_count === 1
            ? "1 record"
            : `${listView.list.item_count.toLocaleString()} records`;
  const canReorder =
    effectiveResultSort(filters, "saved_list").kind === "saved_order" &&
    effectiveResultSort(activeFilters, "saved_list").kind === "saved_order" &&
    listView?.sort.kind === "saved_order" &&
    !filters.filterError &&
    !activeFilters.filterError &&
    !filters.query.trim() &&
    !activeFilters.query.trim() &&
    !buildFilter(filters) &&
    !buildFilter(activeFilters);
  return (
    <ReferenceHelpHost generation={reading.detail?.artifact_generation}>
      <main className="list-workspace">
        <ListBrowseHeader
          error={list.error}
          onRetry={() => void list.refetch()}
          list={listView?.list}
          loading={listLoading}
          onAddRecords={() => setAddOpen(true)}
          editPreparing={editing.preparing}
          onEdit={(opener) => {
            if (listIdentity) editing.begin(listIdentity.list_key, opener);
          }}
        />
        {listView?.reference_error && (
          <Alert type="warning" title="Reference details could not be checked" />
        )}
        <SettingsOpeningError
          error={editing.error}
          title="List settings could not load"
          onRetry={() => {
            editing.opener?.focus({ preventScroll: true });
            editing.retry();
          }}
        />
        {editing.data && editing.ref && (
          <EditListModal
            listRef={editing.ref}
            initialDetail={editing.data}
            open={editing.open}
            onClose={(slug) => {
              editing.close();
              onCloseDirectEditor?.(slug ?? route.slug);
            }}
            afterClose={editing.clear}
            onDeleted={() => navigateDestination(listsDestination())}
          />
        )}
        <ListRecordPicker
          key={route.slug}
          list={listView?.list}
          open={addOpen}
          onClose={() => setAddOpen(false)}
        />
        {narrow && entryNote.detached && <ListEntryNoteEditor state={entryNote} />}
        <BrowseReader
          reading={addressableReading}
          recordActions={(record) =>
            record.record_key === route.selectedRecordKey &&
            record.kind === "creature" ? (
              <AddToRecordButton
                key={record.record_key}
                recordKey={record.record_key}
                record={record}
                encounterOnly
              />
            ) : null
          }
          onClearSelection={() => {
            entryNote.finish();
            selection.cancel();
            reading.select(null);
            void atlasRouter.navigate({
              ...listDestination({
                slug: route.slug,
                selectedRecordKey: null,
                query: listQuery,
              }),
              replace: true,
              resetScroll: false,
              state: true,
            });
          }}
          readerStatus={
            !narrow && hasReferenceSelectionStatus(selection.current) ? (
              <ReferenceSelectionStatus selection={selection} />
            ) : undefined
          }
          selectedRecordKey={route.selectedRecordKey}
          readerFooter={
            entryNote.entry && !(narrow && entryNote.detached) ? (
              <ListEntryNoteEditor state={entryNote} hidden={noteHidden} />
            ) : undefined
          }
          resultsLabel="Saved records"
          resultsIdentity={listView?.items}
          resultsLoading={listLoading || list.isFetching}
          renderResultsPaneHeader={(actions) =>
            listView && (
              <FilterControls
                filterState={{
                  search: filters,
                  setSearch: changeFilters,
                  filterEditor: filterDiscovery.filterEditor,
                  refreshFilterInformation: filterDiscovery.refreshFilterInformation,
                  filterApplicabilityCurrent:
                    filterDiscovery.filterApplicabilityCurrent,
                  filterCountsEnabled: filterDiscovery.filterCountsEnabled,
                  filterCountQueryKey: filterDiscovery.filterCountQueryKey,
                  filterDiscoveryContext: filterDiscovery.filterDiscoveryContext,
                  vocabularyScope: filterDiscovery.vocabularyScope,
                  filterDiscoveryLoading: filterDiscovery.loading,
                  filterDiscoveryError: filterDiscovery.errorMessage,
                }}
                pane={{
                  narrow,
                  summary: (
                    <div className="list-results-summary">
                      {narrow && hasReferenceSelectionStatus(selection.current) ? (
                        <ReferenceSelectionStatus selection={selection} />
                      ) : (
                        <span className="list-results-count" role="status">
                          {resultCount}
                        </span>
                      )}
                      <ResultSortControl
                        compact
                        search={filters}
                        surface="saved_list"
                        setSearch={changeFilters}
                      />
                      {actions}
                    </div>
                  ),
                }}
              />
            )
          }
        >
          <ListItemsPane
            error={removeItem.error}
            reorderError={reorderItem.error}
            mutating={
              paused ||
              reorderItem.isPending ||
              removeItem.isPending ||
              list.isPlaceholderData ||
              Boolean(list.error)
            }
            items={listView?.items ?? []}
            loading={listLoading || list.isFetching}
            unavailable={Boolean(list.error && !listView)}
            filtered={narrowed}
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
              const item = listView?.items.find(
                (item) => item.record_key === recordKey,
              );
              void selection.select({
                request: { navigation: rootNavigation(recordKey) },
                name: item?.record?.title ?? item?.snapshot.title ?? "record",
                generation: null,
                commit: () =>
                  navigateDestination(
                    listDestination({
                      slug: route.slug,
                      selectedRecordKey: recordKey,
                      query: listQuery,
                    }),
                  ),
              });
            }}
          />
        </BrowseReader>
      </main>
    </ReferenceHelpHost>
  );
}

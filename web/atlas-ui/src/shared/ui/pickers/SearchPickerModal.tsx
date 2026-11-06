import { LoadingStatus } from "../LoadingStatus";
import { useDismissOnReadingNavigation } from "../overlays/useDismissOnReadingNavigation";
import { AtlasModal as Modal } from "../overlays/AtlasModal";
import { ErrorNotice } from "../../feedback/ErrorNotice";
import { useQuery } from "@tanstack/react-query";
import {
  Button,
  ConfigProvider,
  Form,
  Grid,
  Pagination,
  Space,
  theme,
  Typography,
} from "antd";
import type { FormInstance, InputRef, ModalProps } from "antd";
import { useId, useLayoutEffect, useRef, useState } from "react";
import { searchPage } from "../../../api/atlasApi";
import { useSearchProfile } from "../../search/SearchProfileContext";
import {
  useReferenceGeneration,
  useReferenceGenerationAdoption,
} from "../../search/useReferenceGeneration";
import { useReferenceQueryRetirement } from "../../queries/useReferenceQueryRetirement";
import { SearchSettings } from "../../search/SearchSettings";
import { similarityFromPreferences } from "../../search/similarityState";
import { SearchProfileNotice } from "../../search/SearchProfileNotice";
import {
  assertSearchIdentity,
  sameSearchPageContext,
  assertSimilarityIdentity,
} from "../../search/searchIdentity";
import type {
  RecordSummaryView,
  RecordNavigationView,
  RecordDetailView,
  SearchResultsPage,
  SearchResultsRequest,
} from "../../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  buildFilterDiscoveryContext,
  effectiveResultSort,
  normalizeSortTransition,
  type SearchFormState,
} from "../../filters/searchState";
import { FilterControls } from "../../filters/FilterPanel";
import { ResultSortControl } from "../../search/ResultSortControl";
import { useFilterDiscovery } from "../../filters/useFilterDiscovery";
import { ReferenceBrowser } from "../../records/ReferenceBrowser";
import { useReferenceReading } from "../../records/useReferenceReading";
import { usePreparedReferenceSelection } from "../../records/usePreparedReferenceSelection";
import {
  ReferenceSelectionStatus,
  hasReferenceSelectionStatus,
} from "../../records/ReferenceSelectionStatus";
import {
  isArtifactGenerationChanged,
  referenceQueryMeta,
} from "../../queries/referenceQueries";

export type PickerSelection = {
  record: RecordSummaryView;
  detail: RecordDetailView;
  artifact_generation: string;
};

export function SearchPickerModal<FormValues>({
  buildRequest,
  disabled = false,
  children,
  emptyPrompt,
  emptyResults,
  form,
  onCancel,
  onFinish,
  onSelectionChange,
  open,
  okButtonProps,
  selectedRecordKey,
  title,
  dismissLabel = "Cancel",
  confirmLabel = "Add",
  status,
  summary,
  searchHeader,
  selectionContext,
  footerLayout = "form",
  initialSelection,
}: {
  buildRequest: (query: string) => SearchResultsRequest;
  disabled?: boolean;
  children?:
    React.ReactNode | ((detail: RecordDetailView | undefined) => React.ReactNode);
  emptyPrompt: string;
  emptyResults: string;
  form: FormInstance<FormValues>;
  onCancel: () => void;
  onFinish: (values: FormValues) => void;
  onSelectionChange: (selection: PickerSelection | null) => void;
  open: boolean;
  okButtonProps?: ModalProps["okButtonProps"];
  selectedRecordKey: string | null;
  title: string;
  dismissLabel?: string;
  confirmLabel?: string;
  status?: React.ReactNode;
  summary?: React.ReactNode;
  searchHeader?: (scope: SearchResultsPage["connection_scope"]) => React.ReactNode;
  selectionContext?: string;
  footerLayout?: "form" | "actions";
  initialSelection?: RecordNavigationView;
}) {
  useDismissOnReadingNavigation(open && !disabled, onCancel);
  const formId = useId();
  const searchInput = useRef<InputRef>(null);
  const { token } = theme.useToken();
  const narrow = !Grid.useBreakpoint().lg;
  const profile = useSearchProfile();
  const [search, setSearch] = useState<SearchFormState>(() => ({
    ...DEFAULT_SEARCH_STATE,
    ranking: profile.preferences.ranking,
  }));
  const [paging, setPaging] = useState({ profile: profile.id, number: 1 });
  const pageNumber = paging.profile === profile.id ? paging.number : 1;
  if (paging.profile !== profile.id) setPaging({ profile: profile.id, number: 1 });
  const setPageNumber = (number: number) => setPaging({ profile: profile.id, number });
  const base = buildRequest(search.query);
  const filter =
    base.criteria.filter && search.filter
      ? { kind: "all_of" as const, children: [base.criteria.filter, search.filter] }
      : (base.criteria.filter ?? search.filter);
  const request: SearchResultsRequest = {
    ...base,
    ...(search.sort ? { sort: search.sort } : {}),
    search_profile: profile.id ?? undefined,
    ranking: search.ranking ?? profile.preferences.ranking,
    criteria:
      base.criteria.kind === "similar_records" ||
      base.criteria.kind === "similar_traits"
        ? {
            ...base.criteria,
            filter,
            similarity:
              search.similarity ??
              base.criteria.similarity ??
              similarityFromPreferences(
                base.criteria.kind === "similar_traits"
                  ? profile.preferences.traitSimilarity
                  : profile.preferences.recordSimilarity,
                base.criteria.kind === "similar_traits",
              ),
          }
        : { ...base.criteria, filter },
    page: { number: pageNumber, size: search.pageSize },
  };
  const requestToken = JSON.stringify(request);
  const profileReady =
    profile.id !== null &&
    (request.criteria.kind === "text_search" ? profile.usable : profile.storedVectors);
  const retrieval = useReferenceGeneration(requestToken, open && profileReady);
  const resetGeneration = retrieval.reset;
  useLayoutEffect(() => {
    if (!open) resetGeneration();
  }, [open, resetGeneration]);
  const resultsKeyForGeneration = (generation: string | null) => [
    "search-picker",
    title,
    request,
    retrieval.epoch,
    generation,
  ];
  const resultsKey = resultsKeyForGeneration(retrieval.generation);
  const recordResults = useQuery<SearchResultsPage>({
    queryKey: resultsKey,
    meta: referenceQueryMeta,
    enabled: open && profileReady && !retrieval.recovering && !search.filterError,
    staleTime: 20_000,
    placeholderData: (previous, previousQuery) =>
      previousQuery?.queryKey[3] === retrieval.epoch &&
      previousQuery.queryKey[4] === retrieval.generation &&
      sameSearchPageContext(previousQuery.queryKey[2] as SearchResultsRequest, request)
        ? previous
        : undefined,
    queryFn: async ({ signal }) => {
      const ticket = retrieval.capture(signal);
      const page = await searchPage(
        {
          ...request,
          expected_artifact_generation:
            search.similarityGeneration ?? retrieval.generation,
        },
        signal,
      );
      assertSearchIdentity(
        page,
        profile.id,
        search.similarityGeneration ?? retrieval.generation,
        effectiveResultSort({
          sort: request.sort,
          query: request.criteria.kind === "text_search" ? request.criteria.query : "",
          similarSeed:
            request.criteria.kind === "similar_records"
              ? request.criteria.seed_record_key
              : null,
        }),
      );
      if (
        request.criteria.kind === "similar_records" ||
        request.criteria.kind === "similar_traits"
      )
        assertSimilarityIdentity(page, request.criteria.similarity);
      retrieval.adopt(ticket, page.artifact_generation, {
        queryKey: resultsKeyForGeneration(page.artifact_generation),
        data: page,
      });
      return page;
    },
  });
  // Scope classification belongs to the static parent context, not a direction's
  // transient result page. Retain this backend fact while different rows load.
  const scopeContext = JSON.stringify([
    request.connection_scope?.parent,
    retrieval.epoch,
    retrieval.generation,
  ]);
  const [retainedScope, setRetainedScope] = useState<{
    context: string;
    value: SearchResultsPage["connection_scope"];
  } | null>(null);
  const generationChanged = isArtifactGenerationChanged(recordResults.error);
  // Changing queries clears their error, but cannot revalidate a rejected generation.
  const scopeGeneration = JSON.stringify([retrieval.epoch, retrieval.generation]);
  const [invalidScopeGeneration, setInvalidScopeGeneration] = useState<string | null>(
    null,
  );
  if (generationChanged && invalidScopeGeneration !== scopeGeneration)
    setInvalidScopeGeneration(scopeGeneration);
  const scopeInvalid = generationChanged || invalidScopeGeneration === scopeGeneration;
  const currentScope = scopeInvalid ? undefined : recordResults.data?.connection_scope;
  if (scopeInvalid && retainedScope) {
    setRetainedScope(null);
  } else if (
    currentScope &&
    !recordResults.isPlaceholderData &&
    (retainedScope?.context !== scopeContext || retainedScope.value !== currentScope)
  ) {
    setRetainedScope({ context: scopeContext, value: currentScope });
  }
  const guidanceScope =
    currentScope ??
    (retainedScope?.context === scopeContext && !scopeInvalid
      ? retainedScope.value
      : undefined);
  useReferenceQueryRetirement(resultsKey);
  useReferenceGenerationAdoption(
    retrieval,
    recordResults,
    recordResults.data?.artifact_generation,
    resultsKeyForGeneration(recordResults.data?.artifact_generation ?? null),
  );
  const reading = useReferenceReading(
    initialSelection ? { navigation: initialSelection } : null,
    open && !retrieval.recovering,
    retrieval.epoch,
    retrieval.generation ??
      request.connection_scope?.parent.expected_artifact_generation ??
      null,
  );
  const selection = usePreparedReferenceSelection(
    reading,
    open && !disabled && !retrieval.recovering,
    retrieval.epoch,
    retrieval.generation,
  );
  const previousSelectionContext = useRef(selectionContext);
  useLayoutEffect(() => {
    if (previousSelectionContext.current === selectionContext) return;
    previousSelectionContext.current = selectionContext;
    selection.cancel();
    reading.select(null);
  }, [selectionContext, selection, reading]);
  const preparing = selection.current;
  const selectionGenerationChanged = isArtifactGenerationChanged(preparing?.error);
  const context = {
    ...buildFilterDiscoveryContext(
      { ...search, filter },
      profile.id ?? undefined,
      retrieval.generation ?? undefined,
    ),
    connection_scope: request.connection_scope,
  };
  const discovery = useFilterDiscovery({
    context,
    enabled: open && !retrieval.recovering,
    queryKeyPrefix: ["filter-discovery", "picker", retrieval.epoch],
    catalogIdentity: [retrieval.epoch],
  });
  function refreshResults() {
    if (request.connection_scope) {
      onCancel();
      return Promise.resolve();
    }
    return retrieval.refresh(() => setPageNumber(1));
  }
  function applySettings(next: SearchFormState) {
    return retrieval.applyGeneration(next.similarityGeneration, () => {
      setSearch(next);
      setPageNumber(1);
    });
  }
  const currentRows =
    recordResults.error || !profileReady ? [] : (recordResults.data?.rows ?? []);

  const [openingReady, setOpeningReady] = useState(false);
  const ready = Boolean(
    recordResults.error ||
    discovery.errorMessage ||
    (discovery.filterEditor && recordResults.data),
  );
  if (!open && openingReady) setOpeningReady(false);
  if (open && ready && !openingReady) setOpeningReady(true);
  const initialLoading = open && profileReady && !openingReady && !ready;
  const reset = () => {
    setSearch({ ...DEFAULT_SEARCH_STATE, ranking: profile.preferences.ranking });
    setPageNumber(1);
    reading.inspect(null);
    onSelectionChange(null);
  };
  const selectionStatus = hasReferenceSelectionStatus(preparing) ? (
    <ReferenceSelectionStatus
      selection={selection}
      generationChangedAction={
        request.connection_scope
          ? {
              label: "Close picker",
              message:
                "Parent results changed. Close this picker, then refresh the parent results.",
            }
          : undefined
      }
      onRetry={() =>
        selectionGenerationChanged ? void refreshResults() : selection.retry()
      }
    />
  ) : undefined;
  return (
    <ConfigProvider componentDisabled={disabled}>
      <Modal
        layout="browse"
        className="reference-picker"
        classNames={{
          container: "reference-picker__container",
          body: "reference-picker__body",
        }}
        width="min(1400px, calc(100vw - 32px))"
        styles={{
          body: { display: "flex", flexDirection: "column", overflow: "hidden" },
          footer: { containerType: "inline-size", containerName: "picker-footer" },
        }}
        afterOpenChange={(isOpen) => {
          if (!isOpen) reset();
        }}
        footer={
          <Form
            name={formId}
            className={`reference-picker__footer${summary ? " reference-picker__footer--summary" : ""}${footerLayout === "actions" ? " reference-picker__footer--actions" : ""}`}
            disabled={disabled}
            form={form}
            layout="vertical"
            onFinish={(values) => {
              if (!disabled && !preparing) onFinish(values);
            }}
          >
            {footerLayout === "actions" ? (
              <div className="reference-picker__action-status">
                {status && <div className="reference-picker__status">{status}</div>}
              </div>
            ) : (
              <>
                {summary && <div className="reference-picker__summary">{summary}</div>}
                <div className="reference-picker__footer-content">
                  <div className="reference-picker__fields">
                    {typeof children === "function"
                      ? children(reading.parentDetail)
                      : children}
                  </div>
                  {!summary && status}
                  {summary && status && (
                    <div className="reference-picker__status">{status}</div>
                  )}
                </div>
              </>
            )}
            <Space
              wrap={footerLayout !== "actions"}
              className="reference-picker__actions"
            >
              <Button disabled={disabled} onClick={onCancel}>
                {dismissLabel}
              </Button>
              <Button
                {...okButtonProps}
                disabled={
                  disabled ||
                  Boolean(preparing) ||
                  !selectedRecordKey ||
                  okButtonProps?.disabled
                }
                type="primary"
                htmlType="submit"
              >
                {confirmLabel}
              </Button>
            </Space>
          </Form>
        }
        closable={!disabled}
        keyboard={!disabled}
        mask={{ closable: !disabled }}
        onCancel={onCancel}
        open={open}
        title={title}
      >
        <div
          className="reference-picker__workspace"
          data-initial-loading={initialLoading || undefined}
          aria-busy={initialLoading}
        >
          {initialLoading && (
            <LoadingStatus
              className="reference-picker__initial-status"
              label="Loading records…"
            />
          )}
          <FilterControls
            includeLinks={!request.connection_scope}
            searchInputRef={searchInput}
            layout="toolbar"
            searchRowEnd={
              <SearchSettings
                seedTitle={
                  recordResults.data?.mode.kind === "similar_records" &&
                  request.criteria.kind === "similar_records" &&
                  recordResults.data.mode.seed.record_key ===
                    request.criteria.seed_record_key
                    ? recordResults.data.mode.seed.title
                    : recordResults.data?.mode.kind === "similar_traits" &&
                        request.criteria.kind === "similar_traits" &&
                        recordResults.data.mode.trait_key ===
                          request.criteria.seed_trait_key
                      ? recordResults.data.mode.title
                      : undefined
                }
                search={
                  request.criteria.kind === "similar_records" ||
                  request.criteria.kind === "similar_traits"
                    ? { ...search, similarity: request.criteria.similarity }
                    : search
                }
                seed={
                  request.criteria.kind === "similar_records"
                    ? { kind: "record", record_key: request.criteria.seed_record_key }
                    : request.criteria.kind === "similar_traits"
                      ? { kind: "trait", trait_key: request.criteria.seed_trait_key }
                      : null
                }
                generation={retrieval.generation}
                disabled={disabled}
                onApply={applySettings}
              />
            }
            filterState={{
              resultRequest: request,
              fixedFilter: base.criteria.filter,
              search,
              setSearch: (next) => {
                if (!disabled) {
                  if (preparing) selection.cancel();
                  setSearch(normalizeSortTransition(search, next));
                  setPageNumber(1);
                }
              },
              filterEditor: discovery.filterEditor,
              filterApplicabilityCurrent: discovery.filterApplicabilityCurrent,
              filterCountsEnabled:
                discovery.filterCountsEnabled &&
                profileReady &&
                request.criteria.kind !== "similar_records",
              filterCountQueryKey: discovery.filterCountQueryKey,
              filterDiscoveryContext: discovery.filterDiscoveryContext,
              vocabularyScope: discovery.vocabularyScope,
              filterDiscoveryLoading: discovery.loading,
              filterDiscoveryError: discovery.errorMessage,
            }}
          >
            {searchHeader?.(guidanceScope)}
          </FilterControls>
          <div className="reference-picker__feedback">
            <SearchProfileNotice />
            {recordResults.error && (
              <ErrorNotice
                error={recordResults.error}
                title="Search failed"
                description={
                  generationChanged && request.connection_scope
                    ? "The parent results changed. Close this picker, then refresh the parent results before reopening it."
                    : undefined
                }
                action={
                  <Button
                    onClick={() =>
                      generationChanged
                        ? void refreshResults()
                        : void recordResults.refetch()
                    }
                    loading={recordResults.isFetching || retrieval.recovering}
                  >
                    {generationChanged
                      ? request.connection_scope
                        ? "Close picker"
                        : "Refresh results"
                      : "Retry search"}
                  </Button>
                }
              />
            )}
          </div>
          <ReferenceBrowser
            rows={currentRows}
            query={
              recordResults.data?.mode.kind === "text_search"
                ? recordResults.data.mode.query
                : undefined
            }
            relationshipClauses={recordResults.data?.relationship_clauses}
            filter={request.criteria.filter}
            generation={recordResults.data?.artifact_generation}
            onRefreshResults={refreshResults}
            loading={
              !recordResults.error && !search.filterError && recordResults.isFetching
            }
            disabled={disabled}
            selectedRecordKey={selectedRecordKey}
            onReaderClose={selection.cancel}
            onClearSelection={() => {
              selection.cancel();
              reading.select(null);
              onSelectionChange(null);
            }}
            onSelect={(row, request) => {
              const page = recordResults.data;
              if (!disabled && page)
                void selection.select({
                  request,
                  name: row.record.title,
                  generation: page.artifact_generation,
                  commit: (detail) =>
                    onSelectionChange({
                      detail,
                      record: row.record,
                      artifact_generation: page.artifact_generation,
                    }),
                });
            }}
            readerStatus={selectionStatus}
            reading={reading}
            emptyMessage={
              recordResults.error || search.filterError
                ? null
                : recordResults.data
                  ? emptyResults
                  : emptyPrompt
            }
            resultsHeader={
              <div className="result-pane-header">
                <div
                  className="reference-picker__result-caption"
                  style={{ minHeight: token.controlHeightSM }}
                >
                  {narrow && selectionStatus ? (
                    selectionStatus
                  ) : (
                    <Typography.Text>
                      {recordResults.isPlaceholderData
                        ? "Updating results"
                        : recordResults.data?.page
                          ? recordResults.data.coverage?.exhaustive === false
                            ? `Top ${recordResults.data.page.total.toLocaleString()} matches`
                            : `${recordResults.data.page.total.toLocaleString()} records`
                          : "Results"}
                    </Typography.Text>
                  )}
                </div>
                <div className="pager pager--compact">
                  <ResultSortControl
                    search={search}
                    disabled={disabled || retrieval.recovering}
                    setSearch={(next) => {
                      setSearch(next);
                      setPageNumber(1);
                    }}
                  />
                  {recordResults.data && recordResults.data.page.total > 0n && (
                    <div
                      className="pager__navigation"
                      role="group"
                      aria-label="Result pages"
                    >
                      <Pagination
                        simple
                        size="small"
                        current={pageNumber}
                        pageSize={search.pageSize}
                        total={Number(recordResults.data?.page?.total ?? 0)}
                        disabled={
                          disabled ||
                          recordResults.isFetching ||
                          generationChanged ||
                          retrieval.recovering
                        }
                        showSizeChanger={false}
                        onChange={setPageNumber}
                      />
                    </div>
                  )}
                </div>
              </div>
            }
          />
        </div>
      </Modal>
    </ConfigProvider>
  );
}

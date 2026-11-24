import { ReferenceHelpHost } from "../../records/ReferenceHelpHost";
import { LoadingStatus } from "../LoadingStatus";
import { useDismissOnReadingNavigation } from "../overlays/useDismissOnReadingNavigation";
import { AtlasModal as Modal } from "../overlays/AtlasModal";
import { ErrorNotice } from "../../feedback/ErrorNotice";
import { skipToken, useQuery } from "@tanstack/react-query";
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
import {
  useEffect,
  useEffectEvent,
  useId,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { searchPage } from "../../../api/atlasApi";
import {
  SearchProfileContext,
  useSearchProfile,
} from "../../search/SearchProfileContext";
import {
  useReferenceGeneration,
  useReferenceGenerationAdoption,
} from "../../search/useReferenceGeneration";
import { useReferenceQueryRetirement } from "../../queries/useReferenceQueryRetirement";
import { SearchSettings } from "../../search/SearchSettings";
import { useSemanticRerunOwner } from "../../search/SemanticReadyNotice";
import { SearchProfileNotice } from "../../search/SearchProfileNotice";
import { searchExecution } from "../../search/searchExecution";
import {
  assertKeywordSearch,
  assertSearchIdentity,
  sameSearchPageContext,
} from "../../search/searchIdentity";
import type {
  QueryPredicate,
  SearchConnectionScope,
  RecordSummaryView,
  RecordNavigationView,
  RecordDetailView,
  SearchResultsPage,
  SearchResultsRequest,
} from "../../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  buildFilterDiscoveryContext,
  buildSearchRequest,
  effectiveResultSort,
  normalizeSortTransition,
  type SearchFormState,
} from "../../filters/searchState";
import { FilterControls } from "../../filters/FilterPanel";
import type { SearchSubmission } from "../../filters/filterControls";
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
  fixedFilter,
  connectionScope,
  disabled = false,
  children,
  emptyPrompt,
  emptyResults,
  form,
  onCancel,
  afterClose,
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
  compactOptions = false,
  searchHeader,
  selectionContext,
  footerLayout = "form",
  initialSelection,
  initialQuery,
  focusSearchOnOpen = false,
}: {
  fixedFilter?: QueryPredicate;
  connectionScope?: SearchConnectionScope;
  disabled?: boolean;
  children?:
    React.ReactNode | ((detail: RecordDetailView | undefined) => React.ReactNode);
  emptyPrompt: string;
  emptyResults: string;
  form: FormInstance<FormValues>;
  onCancel: () => void;
  afterClose?: () => void;
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
  compactOptions?: boolean;
  searchHeader?: (scope: SearchResultsPage["connection_scope"]) => React.ReactNode;
  selectionContext?: string;
  footerLayout?: "form" | "actions";
  initialSelection?: RecordNavigationView;
  initialQuery?: string;
  focusSearchOnOpen?: boolean;
}) {
  useDismissOnReadingNavigation(open && !disabled, onCancel);
  const formId = useId();
  const [optionsOpen, setOptionsOpen] = useState(false);
  const searchInput = useRef<InputRef>(null);
  const { token } = theme.useToken();
  const narrow = !Grid.useBreakpoint().lg;
  const inheritedProfile = useSearchProfile();
  const [submittedProfile, setSubmittedProfile] = useState<{ id: string | null }>();
  const submission = useRef(0);
  useEffect(
    () => () => {
      submission.current += 1;
    },
    [open],
  );
  const selected = inheritedProfile.profiles.find(
    (model) => model.id === submittedProfile?.id,
  );
  const profile = submittedProfile
    ? {
        ...inheritedProfile,
        id: submittedProfile.id,
        keywordSearch: submittedProfile.id === null,
        automaticKeyword: submittedProfile.id === null,
        storedVectors: selected?.stored_vectors ?? false,
        usable: selected?.usable ?? false,
        error:
          submittedProfile.id && !selected
            ? "This search model was removed. Submit a search to use the workspace default."
            : null,
      }
    : inheritedProfile;
  const [seeding, setSeeding] = useState(Boolean(initialQuery?.trim()));
  const [{ search, acceptedSearchSubmission }, setCriteria] = useState<{
    search: SearchFormState;
    acceptedSearchSubmission?: SearchSubmission;
  }>(() => ({
    search: {
      ...DEFAULT_SEARCH_STATE,
      query: initialQuery ?? "",
      mode: initialQuery?.trim() ? "text_search" : "browse",
      ranking: profile.preferences.ranking,
      searchContent: profile.preferences.searchContent,
    },
  }));
  const [searchSession, setSearchSession] = useState(0);
  const [paging, setPaging] = useState({ profile: profile.id, number: 1 });
  const pageNumber = paging.profile === profile.id ? paging.number : 1;
  if (paging.profile !== profile.id) setPaging({ profile: profile.id, number: 1 });
  const setPageNumber = (number: number) => setPaging({ profile: profile.id, number });
  const kind = search.query.trim() ? "text_search" : "list_records";
  const execution = searchExecution(profile, kind);
  const filter =
    fixedFilter && search.filter
      ? { kind: "all_of" as const, children: [fixedFilter, search.filter] }
      : (fixedFilter ?? search.filter);
  const request =
    search.filterError || search.filterWasReset
      ? undefined
      : {
          ...buildSearchRequest(
            { ...search, filter },
            pageNumber,
            null,
            execution.profile,
            execution.mode,
          ),
          result_detail: "evidence" as const,
          connection_scope: connectionScope,
        };
  const requestToken = JSON.stringify(request ?? null);
  const profileReady = execution.ready;
  const retrieval = useReferenceGeneration(
    requestToken,
    open && !seeding && profileReady,
  );
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
    enabled: open && !seeding && profileReady && !retrieval.recovering && !!request,
    staleTime: 20_000,
    placeholderData: (previous, previousQuery) =>
      request &&
      previousQuery?.queryKey[2] &&
      previousQuery.queryKey[3] === retrieval.epoch &&
      previousQuery.queryKey[4] === retrieval.generation &&
      sameSearchPageContext(previousQuery.queryKey[2] as SearchResultsRequest, request)
        ? previous
        : undefined,
    queryFn: !request
      ? skipToken
      : async ({ signal }) => {
          const ticket = retrieval.capture(signal);
          const page = await searchPage(
            {
              ...request,
              expected_artifact_generation: retrieval.generation,
            },
            signal,
          );
          assertSearchIdentity(
            page,
            execution.profile,
            retrieval.generation,
            effectiveResultSort(search),
          );
          if (
            request.criteria.kind === "text_search" &&
            request.criteria.mode === "lexical"
          )
            assertKeywordSearch(page);
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
    connectionScope?.parent,
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
      connectionScope?.parent.expected_artifact_generation ??
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
      execution.profile,
      retrieval.generation ?? undefined,
    ),
    connection_scope: connectionScope,
  };
  const discovery = useFilterDiscovery({
    context,
    enabled: open && !seeding && !retrieval.recovering,
    queryKeyPrefix: ["filter-discovery", "picker", retrieval.epoch],
    catalogIdentity: [retrieval.epoch],
  });
  function refreshResults() {
    if (connectionScope) {
      onCancel();
      return Promise.resolve();
    }
    return retrieval.refresh(() => setPageNumber(1));
  }
  function applySettings(next: SearchFormState) {
    setCriteria((current) => ({ ...current, search: next }));
    setPageNumber(1);
  }
  const currentRows =
    recordResults.error && profileReady ? [] : (recordResults.data?.rows ?? []);

  const [openingReady, setOpeningReady] = useState(false);
  const ready = Boolean(
    recordResults.error ||
    discovery.errorMessage ||
    (discovery.filterEditor && recordResults.data),
  );
  if (!open && openingReady) setOpeningReady(false);
  if (open && ready && !openingReady) setOpeningReady(true);
  const initialLoading = open && profileReady && !openingReady && !ready;
  useLayoutEffect(() => {
    if (!open || !openingReady || !focusSearchOnOpen) return;
    let frame: number;
    const deadline = performance.now() + 500;
    function focusSearch() {
      const input = searchInput.current?.input;
      if (input && getComputedStyle(input).visibility === "visible") {
        input.focus({ preventScroll: true });
      } else if (performance.now() < deadline) {
        frame = requestAnimationFrame(focusSearch);
      }
    }
    frame = requestAnimationFrame(focusSearch);
    return () => cancelAnimationFrame(frame);
  }, [focusSearchOnOpen, open, openingReady]);
  const reset = () => {
    setOptionsOpen(false);
    submission.current += 1;
    setSubmittedProfile(undefined);
    setCriteria({
      search: {
        ...DEFAULT_SEARCH_STATE,
        ranking: profile.preferences.ranking,
        searchContent: profile.preferences.searchContent,
      },
    });
    setSearchSession((session) => session + 1);
    setPageNumber(1);
    reading.inspect(null);
    onSelectionChange(null);
  };
  const selectionStatus = hasReferenceSelectionStatus(preparing) ? (
    <ReferenceSelectionStatus
      selection={selection}
      generationChangedAction={
        connectionScope
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
  function submitWithWorkspaceModel(apply: () => void) {
    const version = ++submission.current;
    const resolve = inheritedProfile.resolveWorkspaceDefault;
    if (!resolve && !profile.semanticDefault) {
      apply();
      return;
    }
    const latest = resolve ? resolve() : Promise.resolve(undefined);
    void latest.then((current) => {
      if (version !== submission.current) return;
      const commit = () => {
        if (version !== submission.current) return;
        if (current) setSubmittedProfile({ id: current.id });
        else if (profile.keywordSearch && profile.semanticDefault)
          setSubmittedProfile({ id: profile.semanticDefault });
        apply();
      };
      if (!current && profile.keywordSearch && profile.semanticDefault)
        void retrieval.refresh(commit);
      else void retrieval.applyGeneration(current?.generation, commit);
    });
  }
  // A handoff is a submitted query and observes the same workspace-model
  // resolution as pressing Enter in the picker.
  const submitSeed = useEffectEvent(() =>
    submitWithWorkspaceModel(() => setSeeding(false)),
  );
  useEffect(() => {
    if (open && seeding && !inheritedProfile.loading) submitSeed();
  }, [open, seeding, inheritedProfile.loading]);
  const rerunSemantic =
    !disabled &&
    !connectionScope &&
    !retrieval.recovering &&
    profile.keywordSearch &&
    profile.semanticDefault &&
    recordResults.data?.mode.kind === "text_search" &&
    recordResults.data.mode.mode === "lexical"
      ? () => {
          submitWithWorkspaceModel(() => setPageNumber(1));
        }
      : undefined;
  useSemanticRerunOwner(open, rerunSemantic);
  const formDetails = footerLayout === "form" && (
    <>
      {summary && <div className="reference-picker__summary">{summary}</div>}
      <div className="reference-picker__footer-content">
        <div className="reference-picker__fields">
          {typeof children === "function" ? children(reading.parentDetail) : children}
        </div>
        {!compactOptions && status && (
          <div className="reference-picker__status">{status}</div>
        )}
      </div>
    </>
  );
  return (
    <SearchProfileContext value={profile}>
      <ReferenceHelpHost generation={recordResults.data?.artifact_generation}>
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
            afterClose={afterClose}
            afterOpenChange={(isOpen) => {
              if (!isOpen) reset();
            }}
            footer={
              <Form
                name={formId}
                className={[
                  "reference-picker__footer",
                  summary && "reference-picker__footer--summary",
                  compactOptions && "reference-picker__footer--compact-options",
                  footerLayout === "actions" && "reference-picker__footer--actions",
                ]
                  .filter(Boolean)
                  .join(" ")}
                disabled={disabled}
                form={form}
                layout="vertical"
                onFinishFailed={compactOptions ? () => setOptionsOpen(true) : undefined}
                onFinish={(values) => {
                  if (!disabled && !preparing) {
                    setOptionsOpen(false);
                    onFinish(values);
                  }
                }}
              >
                {footerLayout === "actions" ? (
                  <div className="reference-picker__action-status">
                    {status && <div className="reference-picker__status">{status}</div>}
                  </div>
                ) : compactOptions ? (
                  <>
                    <div
                      className="reference-picker__options"
                      id={`${formId}-options`}
                      data-open={optionsOpen || undefined}
                      onFocusCapture={() => setOptionsOpen(true)}
                    >
                      {formDetails}
                    </div>
                    {status && <div className="reference-picker__status">{status}</div>}
                    <Button
                      className="reference-picker__options-toggle"
                      aria-controls={`${formId}-options`}
                      aria-expanded={optionsOpen}
                      onClick={() => setOptionsOpen((shown) => !shown)}
                    >
                      Options
                    </Button>
                  </>
                ) : (
                  formDetails
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
                includeLinks={!connectionScope}
                searchInputRef={searchInput}
                searchEntryKey={searchSession}
                layout="toolbar"
                searchRowEnd={
                  <SearchSettings
                    search={search}
                    disabled={disabled}
                    onApply={applySettings}
                  />
                }
                filterState={{
                  resultRequest: request,
                  fixedFilter,
                  search,
                  acceptedSearchSubmission,
                  setSearch: (next, intent, searchSubmission) => {
                    if (disabled) return;
                    if (preparing) selection.cancel();
                    const apply = () => {
                      setCriteria((current) => ({
                        search: normalizeSortTransition(search, next),
                        acceptedSearchSubmission:
                          searchSubmission ?? current.acceptedSearchSubmission,
                      }));
                      setPageNumber(1);
                    };
                    if (
                      intent === "submit" &&
                      next.query.trim() &&
                      !connectionScope &&
                      (inheritedProfile.resolveWorkspaceDefault ||
                        (profile.automaticKeyword && profile.semanticDefault))
                    ) {
                      submitWithWorkspaceModel(apply);
                    } else {
                      submission.current += 1;
                      apply();
                    }
                  },
                  filterEditor: discovery.filterEditor,
                  refreshFilterInformation: refreshResults,
                  filterApplicabilityCurrent: discovery.filterApplicabilityCurrent,
                  filterCountsEnabled: discovery.filterCountsEnabled && profileReady,
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
                <SearchProfileNotice showKeyword rerunSemantic={rerunSemantic} />
                {recordResults.error && (
                  <ErrorNotice
                    error={recordResults.error}
                    title="Search failed"
                    description={
                      generationChanged && connectionScope
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
                          ? connectionScope
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
                matchDisplay="sections"
                query={
                  recordResults.data?.mode.kind === "text_search"
                    ? recordResults.data.mode.query
                    : undefined
                }
                relationshipClauses={recordResults.data?.relationship_clauses}
                filter={filter}
                generation={recordResults.data?.artifact_generation}
                onRefreshResults={refreshResults}
                loading={
                  !recordResults.error &&
                  !search.filterError &&
                  recordResults.isFetching
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
                          setCriteria((current) => ({ ...current, search: next }));
                          setPageNumber(1);
                        }}
                      />
                      {recordResults.data && recordResults.data.page.total > 0 && (
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
                            total={recordResults.data?.page?.total ?? 0}
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
      </ReferenceHelpHost>
    </SearchProfileContext>
  );
}

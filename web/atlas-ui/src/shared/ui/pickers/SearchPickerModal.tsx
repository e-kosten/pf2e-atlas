import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Alert,
  Button,
  ConfigProvider,
  Form,
  Modal,
  Pagination,
  Space,
  Typography,
} from "antd";
import type { FormInstance, ModalProps } from "antd";
import { useId, useLayoutEffect, useRef, useState } from "react";
import { searchPage } from "../../../api/atlasApi";
import { useSearchProfile } from "../../search/SearchProfileContext";
import { SearchProfileNotice } from "../../search/SearchProfileNotice";
import {
  assertSearchIdentity,
  sameSearchPageContext,
} from "../../search/searchIdentity";
import type {
  RecordSummaryView,
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
import {
  isArtifactGenerationChanged,
  retireImmutableReferenceQueries,
} from "../../queries/immutableReferenceQueries";

export type PickerSelection = {
  record: RecordSummaryView;
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
}) {
  const formId = useId();
  const profile = useSearchProfile();
  const [search, setSearch] = useState<SearchFormState>(DEFAULT_SEARCH_STATE);
  const [paging, setPaging] = useState({ profile: profile.id, number: 1 });
  const pageNumber = paging.profile === profile.id ? paging.number : 1;
  if (paging.profile !== profile.id) setPaging({ profile: profile.id, number: 1 });
  const setPageNumber = (number: number) => setPaging({ profile: profile.id, number });
  const queryClient = useQueryClient();
  const [retrieval, setRetrieval] = useState({
    generation: null as string | null,
    epoch: 0,
    recovering: false,
  });
  const base = buildRequest(search.query);
  const filter =
    base.criteria.filter && search.filter
      ? { kind: "all_of" as const, children: [base.criteria.filter, search.filter] }
      : (base.criteria.filter ?? search.filter);
  const request: SearchResultsRequest = {
    ...base,
    ...(search.sort ? { sort: search.sort } : {}),
    search_profile: profile.id ?? undefined,
    criteria: { ...base.criteria, filter },
    page: { number: pageNumber, size: search.pageSize },
  };
  const requestToken = JSON.stringify(request);
  const profileReady =
    profile.id !== null &&
    (request.criteria.kind === "text_search" ? profile.usable : profile.storedVectors);
  const liveRequest = useRef({ token: "", epoch: 0, active: false });
  useLayoutEffect(() => {
    liveRequest.current = {
      token: requestToken,
      epoch: retrieval.epoch,
      active: open && profileReady && !retrieval.recovering,
    };
    return () => {
      liveRequest.current.active = false;
    };
  }, [requestToken, retrieval.epoch, open, profileReady, retrieval.recovering]);
  const recordResults = useQuery<SearchResultsPage>({
    queryKey: ["search-picker", title, request, retrieval.epoch, retrieval.generation],
    enabled: open && profileReady && !retrieval.recovering && !search.filterError,
    staleTime: 20_000,
    placeholderData: (previous, previousQuery) =>
      previousQuery?.queryKey[3] === retrieval.epoch &&
      previousQuery.queryKey[4] === retrieval.generation &&
      sameSearchPageContext(previousQuery.queryKey[2] as SearchResultsRequest, request)
        ? previous
        : undefined,
    queryFn: async ({ signal }) => {
      const page = await searchPage(
        { ...request, expected_artifact_generation: retrieval.generation },
        signal,
      );
      assertSearchIdentity(
        page,
        profile.id,
        retrieval.generation,
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
        retrieval.generation === null &&
        !signal.aborted &&
        liveRequest.current.active &&
        liveRequest.current.token === requestToken &&
        liveRequest.current.epoch === retrieval.epoch
      ) {
        queryClient.setQueryData(
          ["search-picker", title, request, retrieval.epoch, page.artifact_generation],
          page,
        );
        setRetrieval((current) =>
          current.epoch === retrieval.epoch && !current.recovering
            ? { ...current, generation: page.artifact_generation }
            : current,
        );
      }
      return page;
    },
  });
  const reading = useReferenceReading(
    null,
    open && !retrieval.recovering,
    retrieval.epoch,
  );
  const context = buildFilterDiscoveryContext(
    { ...search, filter },
    profile.id ?? undefined,
    retrieval.generation ?? undefined,
  );
  const discovery = useFilterDiscovery({
    context,
    enabled: open && !retrieval.recovering,
    queryKeyPrefix: ["filter-discovery", "picker", retrieval.epoch],
    catalogIdentity: [retrieval.epoch],
  });
  const generationChanged = isArtifactGenerationChanged(recordResults.error);
  async function refreshResults() {
    liveRequest.current.active = false;
    setRetrieval((current) => ({ ...current, recovering: true }));
    await retireImmutableReferenceQueries(queryClient);
    setPageNumber(1);
    setRetrieval((current) => ({
      generation: null,
      epoch: current.epoch + 1,
      recovering: false,
    }));
  }
  const currentRows =
    recordResults.error || !profileReady ? [] : (recordResults.data?.rows ?? []);

  const reset = () => {
    setSearch(DEFAULT_SEARCH_STATE);
    setPageNumber(1);
    reading.inspect(null);
    setRetrieval((current) => ({
      generation: null,
      epoch: current.epoch + 1,
      recovering: false,
    }));
    onSelectionChange(null);
  };
  return (
    <ConfigProvider componentDisabled={disabled}>
      <Modal
        className="reference-picker"
        classNames={{ body: "reference-picker__body" }}
        width="min(1400px, calc(100vw - 32px))"
        styles={{
          body: {
            height: "min(82dvh, 900px, calc(100dvh - 8rem))",
            display: "flex",
            flexDirection: "column",
          },
        }}
        afterOpenChange={(isOpen) => {
          if (!isOpen) reset();
        }}
        footer={null}
        closable={!disabled}
        keyboard={!disabled}
        mask={{ closable: !disabled }}
        onCancel={onCancel}
        open={open}
        title={title}
      >
        <FilterControls
          layout="toolbar"
          filterState={{
            search,
            setSearch: (next) => {
              if (!disabled) {
                setSearch(normalizeSortTransition(search, next));
                setPageNumber(1);
              }
            },
            filterEditor: discovery.filterEditor,
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
        />
        <SearchProfileNotice />
        {recordResults.error && (
          <Alert
            type="error"
            title="Search failed"
            description={recordResults.error.message}
            action={
              <Button
                onClick={() =>
                  generationChanged
                    ? void refreshResults()
                    : void recordResults.refetch()
                }
                loading={recordResults.isFetching || retrieval.recovering}
              >
                {generationChanged ? "Refresh results" : "Retry search"}
              </Button>
            }
          />
        )}
        <ReferenceBrowser
          rows={currentRows}
          relationshipClauses={recordResults.data?.relationship_clauses}
          filter={request.criteria.filter}
          generation={recordResults.data?.artifact_generation}
          onRefreshResults={refreshResults}
          loading={
            !recordResults.error && !search.filterError && recordResults.isFetching
          }
          disabled={disabled}
          selectedRecordKey={selectedRecordKey}
          onSelect={(row) => {
            if (!disabled) {
              if (recordResults.data)
                onSelectionChange({
                  record: row.record,
                  artifact_generation: recordResults.data.artifact_generation,
                });
            }
          }}
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
              <Typography.Text>
                {recordResults.isPlaceholderData
                  ? "Updating results"
                  : recordResults.data?.page
                    ? recordResults.data.coverage?.exhaustive === false
                      ? `Top ${recordResults.data.page.total.toLocaleString()} matches`
                      : `${recordResults.data.page.total.toLocaleString()} records`
                    : "Results"}
              </Typography.Text>
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
        <Form
          name={formId}
          className={`reference-picker__footer${summary ? " reference-picker__footer--summary" : ""}`}
          disabled={disabled}
          form={form}
          layout="vertical"
          onFinish={onFinish}
        >
          <div className="reference-picker__fields">
            {typeof children === "function" ? children(reading.detail) : children}
          </div>
          {summary && <div className="reference-picker__summary">{summary}</div>}
          {!summary && status}
          <Space className="reference-picker__actions">
            <Button disabled={disabled} onClick={onCancel}>
              {dismissLabel}
            </Button>
            <Button
              {...okButtonProps}
              disabled={disabled || !selectedRecordKey || okButtonProps?.disabled}
              type="primary"
              htmlType="submit"
            >
              {confirmLabel}
            </Button>
          </Space>
          {summary && status && (
            <div className="reference-picker__status">{status}</div>
          )}
        </Form>
      </Modal>
    </ConfigProvider>
  );
}

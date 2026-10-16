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
import { useLayoutEffect, useRef, useState } from "react";
import { searchPage } from "../../../api/atlasApi";
import type { SearchResultsRequest, SearchResultRow } from "../../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  buildFilterDiscoveryContext,
  type SearchFormState,
} from "../../filters/searchState";
import { FilterControls } from "../../filters/FilterPanel";
import { useFilterDiscovery } from "../../filters/useFilterDiscovery";
import { ReferenceBrowser } from "../../records/ReferenceBrowser";
import { useReferenceReading } from "../../records/useReferenceReading";
import {
  isArtifactGenerationChanged,
  retireImmutableReferenceQueries,
} from "../../queries/immutableReferenceQueries";

export function SearchPickerModal<FormValues>({
  buildRequest,
  disabled = false,
  children,
  emptyPrompt,
  emptyResults,
  form,
  onCancel,
  onFinish,
  onSelectedRecordKeyChange,
  open,
  okButtonProps,
  selectedLabel,
  selectedPrompt,
  selectedRecordKey,
  title,
}: {
  buildRequest: (query: string) => SearchResultsRequest;
  disabled?: boolean;
  children?: React.ReactNode;
  emptyPrompt: string;
  emptyResults: string;
  form: FormInstance<FormValues>;
  onCancel: () => void;
  onFinish: (values: FormValues) => void;
  onSelectedRecordKeyChange: (recordKey: string | null) => void;
  open: boolean;
  okButtonProps?: ModalProps["okButtonProps"];
  selectedLabel: (row: SearchResultRow) => string;
  selectedPrompt: string;
  selectedRecordKey: string | null;
  title: string;
}) {
  const [search, setSearch] = useState<SearchFormState>(DEFAULT_SEARCH_STATE);
  const [pageNumber, setPageNumber] = useState(1);
  const [candidate, setCandidate] = useState<SearchResultRow | null>(null);
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
    criteria: { ...base.criteria, filter },
    page: { number: pageNumber, size: search.pageSize },
  };
  const requestToken = JSON.stringify(request);
  const liveRequest = useRef({ token: "", epoch: 0, active: false });
  useLayoutEffect(() => {
    liveRequest.current = {
      token: requestToken,
      epoch: retrieval.epoch,
      active: open && !retrieval.recovering,
    };
    return () => {
      liveRequest.current.active = false;
    };
  }, [requestToken, retrieval.epoch, open, retrieval.recovering]);
  const recordResults = useQuery({
    queryKey: ["search-picker", title, request, retrieval.epoch, retrieval.generation],
    enabled: open && !retrieval.recovering && !search.filterError,
    staleTime: 20_000,
    queryFn: async ({ signal }) => {
      const page = await searchPage(
        { ...request, expected_artifact_generation: retrieval.generation },
        signal,
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
  const context = buildFilterDiscoveryContext({ ...search, filter });
  const discovery = useFilterDiscovery({
    context,
    enabled: open && !retrieval.recovering,
    queryKeyPrefix: ["filter-discovery", "picker", retrieval.epoch, requestToken],
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
  const currentRows = recordResults.error ? [] : (recordResults.data?.rows ?? []);
  const selectedRecord =
    candidate?.record.record_key === selectedRecordKey
      ? candidate
      : currentRows.find((row) => row.record.record_key === selectedRecordKey);

  const reset = () => {
    setSearch(DEFAULT_SEARCH_STATE);
    setPageNumber(1);
    setCandidate(null);
    reading.inspect(null);
    setRetrieval((current) => ({
      generation: null,
      epoch: current.epoch + 1,
      recovering: false,
    }));
    onSelectedRecordKeyChange(null);
  };
  return (
    <ConfigProvider componentDisabled={disabled}>
      <Modal
        className="reference-picker"
        width="min(1400px, calc(100vw - 32px))"
        styles={{
          body: {
            height: "min(70dvh, 820px)",
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
                setSearch(next);
                setPageNumber(1);
              }
            },
            filterEditor: discovery.filterEditor,
            filterCountsByField: discovery.filterCountsByField,
            filterDiscoveryContext: discovery.filterDiscoveryContext,
            vocabularyScope: discovery.vocabularyScope,
            filterDiscoveryLoading: discovery.loading,
            filterDiscoveryError: discovery.errorMessage,
          }}
        />
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
          loading={
            !recordResults.error && !search.filterError && recordResults.isFetching
          }
          disabled={disabled}
          selectedRecordKey={selectedRecordKey}
          onSelect={(row) => {
            if (!disabled) {
              setCandidate(row);
              onSelectedRecordKeyChange(row.record.record_key);
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
            <Space wrap>
              <Typography.Text>
                {recordResults.data?.page
                  ? `${recordResults.data.page.total.toLocaleString()} records`
                  : "Results"}
              </Typography.Text>
              {recordResults.data && recordResults.data.page.total > 0n && (
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
              )}
            </Space>
          }
        />
        <Form
          className="reference-picker__footer"
          disabled={disabled}
          form={form}
          layout="vertical"
          onFinish={onFinish}
        >
          <Typography.Text
            className="search-picker__selection"
            role="status"
            aria-live="polite"
          >
            {selectedRecord ? selectedLabel(selectedRecord) : selectedPrompt}
          </Typography.Text>
          <div className="reference-picker__fields">{children}</div>
          <Space>
            <Button disabled={disabled} onClick={onCancel}>
              Cancel
            </Button>
            <Button
              {...okButtonProps}
              disabled={disabled || !selectedRecordKey || okButtonProps?.disabled}
              type="primary"
              htmlType="submit"
            >
              Add
            </Button>
          </Space>
        </Form>
      </Modal>
    </ConfigProvider>
  );
}

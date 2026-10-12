import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Form, Input, Modal, Table } from "antd";
import type { FormInstance, ModalProps } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { searchPage } from "../../../api/atlasApi";
import type { SearchResultsRequest, SearchResultRow } from "../../../generated/atlas";
import {
  isArtifactGenerationChanged,
  retireImmutableReferenceQueries,
} from "../../queries/immutableReferenceQueries";

const SEARCH_PICKER_DEBOUNCE_MS = 250;

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
  const searchId = useId();
  const [search, setSearch] = useState("");
  const [activeSearch, setActiveSearch] = useState("");
  const searchTimeout = useRef<number | undefined>(undefined);
  const queryClient = useQueryClient();
  const [retrieval, setRetrieval] = useState({
    generation: null as string | null,
    epoch: 0,
    recovering: false,
  });
  const request = buildRequest(activeSearch);
  const requestToken = JSON.stringify(request);
  const liveRequest = useRef({ token: "", epoch: 0, active: false });
  useLayoutEffect(() => {
    liveRequest.current = { token: requestToken, epoch: retrieval.epoch, active: open };
    return () => {
      liveRequest.current.active = false;
    };
  }, [requestToken, retrieval.epoch, open]);
  useEffect(
    () => () => {
      window.clearTimeout(searchTimeout.current);
    },
    [],
  );
  const recordResults = useQuery({
    queryKey: ["search-picker", title, request, retrieval.epoch, retrieval.generation],
    enabled: open && activeSearch.trim().length > 0 && !retrieval.recovering,
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
  const generationChanged = isArtifactGenerationChanged(recordResults.error);
  async function refreshResults() {
    liveRequest.current.active = false;
    setRetrieval((current) => ({ ...current, recovering: true }));
    await retireImmutableReferenceQueries(queryClient);
    setRetrieval((current) => ({
      generation: null,
      epoch: current.epoch + 1,
      recovering: false,
    }));
  }
  const currentRows =
    activeSearch.trim() && search === activeSearch && !recordResults.error
      ? (recordResults.data?.rows ?? [])
      : [];
  const selectedRecord =
    currentRows.find((row) => row.record.record_key === selectedRecordKey) ?? null;

  const reset = () => {
    setSearch("");
    setActiveSearch("");
    setRetrieval((current) => ({
      generation: null,
      epoch: current.epoch + 1,
      recovering: false,
    }));
    window.clearTimeout(searchTimeout.current);
    onSelectedRecordKeyChange(null);
  };

  return (
    <Modal
      afterOpenChange={(isOpen) => {
        if (!isOpen) {
          reset();
        }
      }}
      okButtonProps={{
        ...okButtonProps,
        disabled: disabled || okButtonProps?.disabled,
      }}
      cancelButtonProps={{ disabled }}
      closable={!disabled}
      keyboard={!disabled}
      mask={{ closable: !disabled }}
      onCancel={onCancel}
      onOk={() => {
        if (!disabled) form.submit();
      }}
      open={open}
      title={title}
    >
      <Form disabled={disabled} form={form} layout="vertical" onFinish={onFinish}>
        <Form.Item label="Search" htmlFor={searchId} layout="vertical">
          <Input
            id={searchId}
            autoComplete="off"
            allowClear
            aria-label="Search"
            onChange={(event) => {
              const query = event.target.value;
              setSearch(query);
              onSelectedRecordKeyChange(null);
              window.clearTimeout(searchTimeout.current);
              searchTimeout.current = window.setTimeout(
                () => setActiveSearch(query),
                SEARCH_PICKER_DEBOUNCE_MS,
              );
            }}
            value={search}
          />
        </Form.Item>
        {recordResults.error && search === activeSearch && (
          <Alert
            type="error"
            title="Search failed"
            description={recordResults.error.message}
            action={
              <Button
                onClick={() => {
                  if (generationChanged) void refreshResults();
                  else void recordResults.refetch();
                }}
                loading={recordResults.isFetching || retrieval.recovering}
              >
                {generationChanged ? "Refresh results" : "Retry search"}
              </Button>
            }
          />
        )}
        <Table<SearchResultRow>
          columns={recordPickerColumns}
          dataSource={currentRows}
          loading={recordResults.isFetching || search !== activeSearch}
          locale={{
            emptyText: recordResults.error
              ? "Search unavailable"
              : activeSearch.trim()
                ? emptyResults
                : emptyPrompt,
          }}
          pagination={false}
          rowSelection={{
            type: "radio",
            selectedRowKeys: selectedRecordKey ? [selectedRecordKey] : [],
            onChange: (keys) => {
              if (!disabled)
                onSelectedRecordKeyChange(keys.length ? String(keys[0]) : null);
            },
            getCheckboxProps: (row) => ({
              "aria-label": `Select ${row.record.title}`,
              disabled: disabled || recordResults.isFetching,
            }),
          }}
          rowClassName="search-picker__row"
          onRow={(row) => ({
            onClick: (event) => {
              if (!disabled && !(event.target as Element).closest("input,button,label"))
                onSelectedRecordKeyChange(row.record.record_key);
            },
          })}
          rowKey={(row) => row.record.record_key}
          scroll={{ y: 280 }}
          size="small"
        />
        <p className="search-picker__selection" role="status" aria-live="polite">
          {selectedRecord ? selectedLabel(selectedRecord) : selectedPrompt}
        </p>
        {children}
      </Form>
    </Modal>
  );
}

const recordPickerColumns: ColumnsType<SearchResultRow> = [
  { title: "Name", dataIndex: ["record", "title"] },
  { title: "Kind", dataIndex: ["record", "kind_label"], width: 100 },
  {
    title: "Level",
    dataIndex: ["record", "level_label"],
    width: 90,
    render: (value) => value ?? "",
  },
];

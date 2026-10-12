import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Form, Input, Modal, Table } from "antd";
import type { FormInstance, ModalProps } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useId, useRef, useState } from "react";
import { openResultWindow } from "../../../api/atlasApi";
import type {
  OpenResultWindowRequest,
  ResultWindowRow,
} from "../../../generated/atlas";

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
  buildRequest: (query: string) => OpenResultWindowRequest;
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
  selectedLabel: (row: ResultWindowRow) => string;
  selectedPrompt: string;
  selectedRecordKey: string | null;
  title: string;
}) {
  const searchId = useId();
  const [search, setSearch] = useState("");
  const [activeSearch, setActiveSearch] = useState("");
  const searchTimeout = useRef<number | undefined>(undefined);
  useEffect(
    () => () => {
      window.clearTimeout(searchTimeout.current);
    },
    [],
  );
  const recordResults = useQuery({
    queryKey: ["search-picker", title, activeSearch],
    enabled: open && activeSearch.trim().length > 0,
    queryFn: () => openResultWindow(buildRequest(activeSearch)),
  });
  const currentRows =
    activeSearch.trim() && search === activeSearch && !recordResults.error
      ? (recordResults.data?.rows ?? [])
      : [];
  const selectedRecord =
    currentRows.find((row) => row.record.record_key === selectedRecordKey) ?? null;

  const reset = () => {
    setSearch("");
    setActiveSearch("");
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
                onClick={() => recordResults.refetch()}
                loading={recordResults.isFetching}
              >
                Retry search
              </Button>
            }
          />
        )}
        <Table<ResultWindowRow>
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

const recordPickerColumns: ColumnsType<ResultWindowRow> = [
  { title: "Name", dataIndex: ["record", "title"] },
  { title: "Kind", dataIndex: ["record", "kind_label"], width: 100 },
  {
    title: "Level",
    dataIndex: ["record", "level_label"],
    width: 90,
    render: (value) => value ?? "",
  },
];

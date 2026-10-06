import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { Form, Input, Modal, Table } from "antd";
import type { FormInstance, ModalProps } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useMemo, useRef, useState } from "react";
import { openResultWindow } from "../../../api/atlasApi";
import type {
  OpenResultWindowRequest,
  ResultWindowRow,
} from "../../../generated/atlas";

const SEARCH_PICKER_DEBOUNCE_MS = 250;

export function SearchPickerModal<FormValues>({
  buildRequest,
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
    placeholderData: keepPreviousData,
    queryFn: () => openResultWindow(buildRequest(activeSearch)),
  });
  const selectedRecord = useMemo(
    () =>
      recordResults.data?.rows.find(
        (row) => row.record.surface.metadata.record_key === selectedRecordKey,
      ) ?? null,
    [recordResults.data?.rows, selectedRecordKey],
  );

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
      okButtonProps={okButtonProps}
      onCancel={onCancel}
      onOk={() => form?.submit()}
      open={open}
      title={title}
    >
      <Form form={form} layout="vertical" onFinish={onFinish}>
        <Form.Item label="Search" layout="vertical">
          <Input
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
        <Table
          columns={recordPickerColumns(onSelectedRecordKeyChange)}
          dataSource={activeSearch.trim().length > 0 ? recordResults.data?.rows : []}
          loading={recordResults.isLoading || recordResults.isFetching}
          locale={{
            emptyText: activeSearch.trim().length > 0 ? emptyResults : emptyPrompt,
          }}
          pagination={false}
          rowClassName={(row) =>
            row.record.surface.metadata.record_key === selectedRecordKey
              ? "search-picker__row search-picker__row--selected"
              : "search-picker__row"
          }
          onRow={(row) => ({
            onClick: () => {
              const recordKey = row.record.surface.metadata.record_key;
              if (recordKey) {
                onSelectedRecordKeyChange(recordKey);
              }
            },
          })}
          rowKey={(row) =>
            row.record.surface.metadata.record_key ?? row.record.surface.metadata.title
          }
          scroll={{ y: 280 }}
          size="small"
        />
        <p className="search-picker__selection">
          {selectedRecord ? selectedLabel(selectedRecord) : selectedPrompt}
        </p>
        {children}
      </Form>
    </Modal>
  );
}

function recordPickerColumns(
  onSelectRecord: (recordKey: string) => void,
): ColumnsType<ResultWindowRow> {
  return [
    {
      title: "Name",
      dataIndex: ["record", "title"],
      render: (_, row) => (
        <span
          className="search-picker__title"
          onClick={(event) => {
            event.stopPropagation();
            const recordKey = row.record.surface.metadata.record_key;
            if (recordKey) {
              onSelectRecord(recordKey);
            }
          }}
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              const recordKey = row.record.surface.metadata.record_key;
              if (recordKey) {
                onSelectRecord(recordKey);
              }
            }
          }}
          role="button"
          tabIndex={0}
        >
          <span>{row.record.surface.metadata.title}</span>
        </span>
      ),
    },
    {
      title: "Kind",
      render: (_, row) => row.record.surface.metadata.kind_label,
      width: 100,
    },
    {
      title: "Level",
      width: 90,
      render: (_, row) => row.record.surface.metadata.level?.toString() ?? "",
    },
  ];
}

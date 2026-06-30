import { Table } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Trash2 } from "lucide-react";
import type { SavedListItemView } from "../../generated/atlas";
import { PaneIconButton } from "../../shared/ui/actions/PaneAction";
import { RowTitleButton } from "../../shared/ui/tables/IndexTable";

export function ListItemsPane({
  items,
  loading,
  onRemove,
  onSelect,
  removingKey,
  selectedRecordKey,
}: {
  items: SavedListItemView[];
  loading: boolean;
  onRemove: (recordKey: string) => void;
  onSelect: (recordKey: string) => void;
  removingKey: string | null;
  selectedRecordKey: string | null;
}) {
  const columns: ColumnsType<SavedListItemView> = [
    {
      title: "Record",
      render: (_, item) => (
        <RowTitleButton
          disabled={item.status === "unresolved"}
          onClick={() => onSelect(item.record_key)}
          title={item.record?.title ?? item.snapshot.title}
        />
      ),
    },
    {
      title: "Kind",
      width: 120,
      render: (_, item) => item.record?.kind_label ?? item.snapshot.kind ?? "",
    },
    {
      title: "",
      width: 52,
      render: (_, item) => (
        <PaneIconButton
          disabled={removingKey === item.record_key}
          icon={<Trash2 size={15} />}
          label={`Remove ${item.snapshot.title}`}
          onClick={() => onRemove(item.record_key)}
          title="Remove"
        />
      ),
    },
  ];

  return (
    <section className="results-panel">
      <div className="results-scroll">
        <Table
          columns={columns}
          dataSource={items}
          loading={loading}
          locale={{ emptyText: "No saved records" }}
          pagination={false}
          rowClassName={(item) =>
            item.record_key === selectedRecordKey
              ? "result-row ant-table-row-selected"
              : "result-row"
          }
          rowKey={(item) => item.record_key}
          size="middle"
        />
      </div>
    </section>
  );
}

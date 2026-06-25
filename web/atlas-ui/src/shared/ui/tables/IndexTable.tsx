import { Table } from "antd";
import type { TableProps } from "antd";
import type { Key } from "react";

type IndexTableProps<Row extends object> = Omit<
  TableProps<Row>,
  "onRow" | "pagination" | "rowKey" | "size"
> & {
  onActivateRow: (row: Row) => void;
  rowKey: (row: Row) => Key;
};

export function IndexTable<Row extends object>({
  onActivateRow,
  rowKey,
  ...props
}: IndexTableProps<Row>) {
  return (
    <Table<Row>
      {...props}
      pagination={false}
      onRow={(row) => ({
        className: "index-row",
        tabIndex: 0,
        onClick: () => onActivateRow(row),
        onKeyDown: (event) => {
          if (event.key === "Enter") {
            onActivateRow(row);
          }
        },
      })}
      rowKey={rowKey}
      size="middle"
    />
  );
}

export function stopIndexRowAction(event: { stopPropagation: () => void }) {
  event.stopPropagation();
}

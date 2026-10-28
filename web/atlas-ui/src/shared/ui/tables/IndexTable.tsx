import { Table, Typography } from "antd";
import type { TableProps } from "antd";
import type { Key, MouseEvent, ReactNode } from "react";

type IndexTableProps<Row extends object> = Omit<
  TableProps<Row>,
  "onRow" | "pagination" | "rowKey" | "size" | "tableLayout"
> & {
  onActivateRow: (row: Row) => void;
  rowKey: (row: Row) => Key;
};

export function IndexTable<Row extends object>({
  onActivateRow,
  rowKey,
  loading,
  locale,
  ...props
}: IndexTableProps<Row>) {
  const spinning = typeof loading === "object" ? (loading.spinning ?? true) : !!loading;
  return (
    <Table<Row>
      {...props}
      loading={
        typeof loading === "object"
          ? { delay: 500, ...loading }
          : { spinning, delay: 500 }
      }
      locale={{
        ...locale,
        emptyText: spinning && !props.dataSource?.length ? null : locale?.emptyText,
      }}
      tableLayout="auto"
      pagination={false}
      onRow={(row) => ({
        className: "index-row",
        tabIndex: 0,
        onClick: (event) => {
          if (
            event.defaultPrevented ||
            event.button !== 0 ||
            event.altKey ||
            event.ctrlKey ||
            event.metaKey ||
            event.shiftKey
          ) {
            return;
          }
          const interactive =
            event.target instanceof Element
              ? event.target.closest(
                  'a, button, input, select, textarea, summary, [role="button"], [role="link"], [role="checkbox"], [role="radio"], [role="switch"], [role="combobox"], [contenteditable]:not([contenteditable="false"]), [tabindex]:not([tabindex="-1"])',
                )
              : null;
          if (interactive && interactive !== event.currentTarget) {
            return;
          }
          onActivateRow(row);
        },
        onKeyDown: (event) => {
          if (
            event.key === "Enter" &&
            event.target === event.currentTarget &&
            !event.defaultPrevented
          ) {
            event.preventDefault();
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

export function RowTitleLink({
  disabled = false,
  href,
  onClick,
  subtitle,
  title,
}: {
  disabled?: boolean;
  href?: string;
  onClick?: (event: MouseEvent<HTMLElement>) => void;
  subtitle?: ReactNode;
  title: ReactNode;
}) {
  return (
    <Typography.Link
      style={{ display: "grid", height: "auto", padding: 0 }}
      className="index-row-link"
      disabled={disabled}
      href={disabled ? undefined : href}
      onClick={disabled ? undefined : onClick}
    >
      <span className="index-row-link__title">{title}</span>
      {subtitle ? <small>{subtitle}</small> : null}
    </Typography.Link>
  );
}

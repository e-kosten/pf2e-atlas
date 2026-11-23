import { Table, Typography } from "antd";
import type { TableProps } from "antd";
import type { TableRef } from "antd/es/table";
import { useLayoutEffect, useRef, useState } from "react";
import type { Key, MouseEvent, ReactNode } from "react";

type IndexTableProps<Row extends object> = Omit<
  TableProps<Row>,
  "onRow" | "pagination" | "rowKey" | "size" | "tableLayout"
> & {
  compact?: { below: number; columns: NonNullable<TableProps<Row>["columns"]> };
  onActivateRow: (row: Row) => void;
  rowKey: (row: Row) => Key;
};

export function IndexTable<Row extends object>({
  onActivateRow,
  compact,
  rowKey,
  loading,
  locale,
  ...props
}: IndexTableProps<Row>) {
  const ref = useRef<TableRef>(null);
  const [narrow, setNarrow] = useState(false);
  const breakpoint = compact?.below;
  useLayoutEffect(() => {
    const host = ref.current?.nativeElement;
    if (!host || breakpoint === undefined) return;
    const measure = () => {
      const width = host.clientWidth;
      if (width > 0) setNarrow(width < breakpoint);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(host);
    return () => observer.disconnect();
  }, [breakpoint]);
  const compactColumns = narrow ? compact?.columns : undefined;
  const spinning = typeof loading === "object" ? (loading.spinning ?? true) : !!loading;
  return (
    <Table<Row>
      {...props}
      ref={ref}
      columns={compactColumns ?? props.columns}
      className={[props.className, compactColumns && "index-table--compact"]
        .filter(Boolean)
        .join(" ")}
      locale={
        typeof locale?.emptyText === "string"
          ? {
              ...locale,
              emptyText: <span className="index-table-empty">{locale.emptyText}</span>,
            }
          : locale
      }
      loading={
        typeof loading === "object"
          ? { delay: 500, ...loading }
          : { spinning, delay: 500 }
      }
      tableLayout={compactColumns ? "fixed" : "auto"}
      pagination={false}
      onRow={(row) => ({
        className: "index-row",
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
          if (interactive && event.currentTarget.contains(interactive)) {
            return;
          }
          onActivateRow(row);
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

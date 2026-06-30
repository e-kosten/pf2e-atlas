import { Button, Table, Typography } from "antd";
import type { TableProps } from "antd";
import type { Key, MouseEvent, ReactNode } from "react";

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

export function RowTitleButton({
  disabled = false,
  onClick,
  subtitle,
  title,
}: {
  disabled?: boolean;
  onClick: (event: MouseEvent<HTMLElement>) => void;
  subtitle?: ReactNode;
  title: ReactNode;
}) {
  return (
    <Button
      className="index-row-link index-row-link--button"
      disabled={disabled}
      onClick={onClick}
      type="link"
    >
      <span className="index-row-link__title">{title}</span>
      {subtitle ? <small>{subtitle}</small> : null}
    </Button>
  );
}

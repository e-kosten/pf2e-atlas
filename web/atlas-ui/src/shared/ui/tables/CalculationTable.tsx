import { Table } from "antd";

export type CalculationRow = {
  key: string;
  label: string;
  value: string;
  note?: string;
  muted?: boolean;
  total?: boolean;
};

export function CalculationTable({
  label,
  valueLabel,
  rows,
  explanation,
}: {
  label: string;
  valueLabel: string;
  rows: CalculationRow[];
  explanation?: string;
}) {
  return (
    <div className="calculation-table" role="region" aria-label={label}>
      {explanation && <p>{explanation}</p>}
      <Table
        dataSource={rows}
        rowKey="key"
        pagination={false}
        rowHoverable={false}
        size="small"
        columns={[
          {
            title: "Cause",
            key: "cause",
            render: (_, row) => (
              <span className={row.muted ? "calculation-table__muted" : undefined}>
                {row.total ? <strong>{row.label}</strong> : row.label}
                {row.note && (
                  <span className="calculation-table__note">{row.note}</span>
                )}
              </span>
            ),
          },
          {
            title: valueLabel,
            key: "value",
            align: "right",
            render: (_, row) => (
              <span className={row.muted ? "calculation-table__muted" : undefined}>
                {row.total ? <strong>{row.value}</strong> : row.value}
              </span>
            ),
          },
        ]}
      />
    </div>
  );
}

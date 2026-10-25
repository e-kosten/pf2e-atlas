import type { RuntimeCountView } from "../../generated/atlas";
import { CalculationTable, type CalculationRow } from "../ui/tables/CalculationTable";

export function RuntimeCountCalculation({ count }: { count: RuntimeCountView }) {
  const rows: CalculationRow[] = [
    { key: "base", label: "Base", value: String(count.base_value) },
    ...count.adjustments.map((modifier) => ({
      key: modifier.source,
      label: modifier.label,
      value: `${modifier.value > 0n ? "+" : ""}${modifier.value}`,
      note: modifier.reason,
    })),
    ...count.suppressed_adjustments.map((modifier) => ({
      key: `suppressed-${modifier.source}`,
      label: modifier.label,
      value: "—",
      note: modifier.reason,
      muted: true,
    })),
    ...count.segments.map((segment, index) => ({
      key: `segment-${index}`,
      label: segment.label,
      value: String(segment.value),
      note: segment.reason ?? (segment.restricted ? "Restricted" : undefined),
    })),
    {
      key: "total",
      label: "Total",
      total: true,
      value:
        count.adjusted_value === null ? "Needs review" : String(count.adjusted_value),
    },
  ];
  return (
    <CalculationTable
      label={`${count.label} calculation`}
      valueLabel={count.label}
      rows={rows}
      explanation={count.explanation}
    />
  );
}

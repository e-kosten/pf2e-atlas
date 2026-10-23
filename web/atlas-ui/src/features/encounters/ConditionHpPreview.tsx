import type { ConditionHpChangeView } from "../../generated/atlas";
import { CalculationTable } from "../../shared/ui/tables/CalculationTable";
export function ConditionHpPreview({ change }: { change: ConditionHpChangeView }) {
  const value = (input: bigint | null) => input?.toString() ?? "—";
  return (
    <CalculationTable
      label="HP change"
      valueLabel="Change"
      rows={[
        {
          key: "current",
          label: "Current HP",
          value: `${value(change.current_before)} → ${value(change.current_after)}`,
        },
        {
          key: "maximum",
          label: "Maximum HP",
          value: `${value(change.maximum_before)} → ${value(change.maximum_after)}`,
        },
      ]}
    />
  );
}

import { Button, Popover, Tooltip } from "antd";
import { CalculationTable } from "../ui/tables/CalculationTable";
import { Info } from "lucide-react";
import type { ReactNode } from "react";
import type { FactView, NumberFactView } from "../../generated/atlas";

export function isEmptyFactValue(value: unknown): boolean {
  return (
    (typeof value === "string" && !value.trim()) ||
    (Array.isArray(value) && value.length === 0)
  );
}
export function hasReadingFact<T>(
  fact: FactView<T>,
  { patch = false, required = false }: { patch?: boolean; required?: boolean } = {},
): boolean {
  if (fact.state === "not_applicable" || (patch && fact.state === "missing"))
    return false;
  return patch || required || fact.state !== "value" || !isEmptyFactValue(fact.value);
}
export function factText<T>(
  fact: FactView<T>,
  format: (value: T) => string = String,
  { patch = false, required = false }: { patch?: boolean; required?: boolean } = {},
): string {
  if (patch && fact.state === "null") return "Cleared";
  if ((patch || required) && fact.state === "value" && isEmptyFactValue(fact.value))
    return "Empty";
  return fact.state === "value" && fact.value !== null
    ? format(fact.value)
    : fact.state === "not_applicable"
      ? ""
      : fact.state === "missing"
        ? "—"
        : `[${fact.state}]`;
}

function MissingFact({ label }: { label?: string }) {
  const explanation = label ? `${label} is unavailable.` : "Value is unavailable.";
  return (
    <Tooltip title={explanation} trigger={["hover", "focus"]}>
      <Button
        type="text"
        size="small"
        className="record-missing"
        aria-label={explanation}
      >
        —
      </Button>
    </Tooltip>
  );
}

export function NumberFact({
  fact,
  signed = false,
  unit = "",
  label,
}: {
  fact: NumberFactView;
  signed?: boolean;
  unit?: string;
  label?: string;
}) {
  const format = (value: number) => `${signed && value >= 0 ? "+" : ""}${value}${unit}`;
  const text = factText(fact, format);
  const adjustment = fact.adjustment;
  if (fact.state === "missing" && !adjustment) return <MissingFact label={label} />;
  if (!adjustment) return <>{text}</>;
  const contextual = adjustment.notes.length > 0;
  return (
    <span className={contextual ? "record-number-context" : undefined}>
      {contextual && text}
      <Popover
        trigger="click"
        title="Effective value"
        content={
          <CalculationTable
            label={`${label ?? "Value"} calculation`}
            valueLabel={label ?? "Value"}
            rows={[
              {
                key: "original",
                label: "Original",
                value: format(adjustment.authored),
              },
              ...adjustment.applied.map((modifier, index) => ({
                key: `applied-${index}`,
                label: modifier.label,
                value: `${modifier.value > 0n ? "+" : ""}${modifier.value}${unit}`,
                note: modifier.modifier_type,
              })),
              ...adjustment.suppressed.map((modifier, index) => ({
                key: `suppressed-${index}`,
                label: modifier.label,
                value: "—",
                note: `${modifier.value > 0n ? "+" : ""}${modifier.value}${unit} ${modifier.modifier_type} · Overridden`,
                muted: true,
              })),
              ...adjustment.notes.map((note, index) => ({
                key: `note-${index}`,
                label: note.label,
                value: "—",
                note: note.reason,
              })),
              { key: "effective", label: "Effective", value: text, total: true },
            ]}
          />
        }
      >
        <Button
          aria-label={
            label ? `Show explanation for ${label}` : "Show value explanation"
          }
          className={contextual ? undefined : "record-number"}
          type={contextual ? "text" : "link"}
          size="small"
          icon={contextual ? <Info size={14} aria-hidden="true" /> : undefined}
        >
          {contextual ? null : text}
        </Button>
      </Popover>
    </span>
  );
}

export function FactLine<T>({
  label,
  fact,
  format,
  patch = false,
  required = false,
}: {
  label: string;
  fact: FactView<T>;
  format?: (value: T) => string;
  patch?: boolean;
  required?: boolean;
}) {
  if (!hasReadingFact(fact, { patch, required })) return null;
  return (
    <FactRow label={label}>
      {fact.state === "missing" ? (
        <MissingFact label={label} />
      ) : (
        factText(fact, format, { patch, required })
      )}
    </FactRow>
  );
}

export function FactRow({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="record-fact">
      <strong>{label}</strong> <span>{children}</span>
    </div>
  );
}

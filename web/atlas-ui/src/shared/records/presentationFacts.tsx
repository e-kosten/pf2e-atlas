import { Button, Popover } from "antd";
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
      : `[${fact.state}]`;
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
  if (!adjustment) return <>{text}</>;
  return (
    <Popover
      title="Effective value"
      content={
        <div className="record-number-explanation">
          <p>Authored: {format(adjustment.authored)}</p>
          <p>Effective: {text}</p>
          {adjustment.applied.map((modifier, index) => (
            <p key={index}>
              {modifier.label}: {String(modifier.value)} ({modifier.modifier_type})
            </p>
          ))}
          {adjustment.suppressed.map((modifier, index) => (
            <p key={index}>
              {modifier.label}: {String(modifier.value)} (suppressed)
            </p>
          ))}
          {adjustment.notes.map((note, index) => (
            <p key={index}>
              {note.label}: {note.reason}
            </p>
          ))}
        </div>
      }
    >
      <Button
        aria-label={label ? `Show explanation for ${label}` : undefined}
        className="record-number"
        type="link"
        size="small"
      >
        {text}
      </Button>
    </Popover>
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
  return <FactRow label={label}>{factText(fact, format, { patch, required })}</FactRow>;
}

export function FactRow({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="record-fact">
      <strong>{label}</strong> <span>{children}</span>
    </div>
  );
}

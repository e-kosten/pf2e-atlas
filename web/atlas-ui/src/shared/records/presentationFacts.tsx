import { Button, Popover } from "antd";
import type { ReactNode } from "react";
import type { FactView, NumberFactView } from "../../generated/atlas";

export function factText<T>(
  fact: FactView<T>,
  format: (value: T) => string = String,
): string {
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
}: {
  label: string;
  fact: FactView<T>;
  format?: (value: T) => string;
  patch?: boolean;
}) {
  if (
    fact.state === "not_applicable" ||
    (patch && fact.state === "missing") ||
    (!patch &&
      fact.state === "value" &&
      (fact.value === "" || (Array.isArray(fact.value) && !fact.value.length)))
  )
    return null;
  return (
    <FactRow label={label}>
      {patch && fact.state === "null"
        ? "Cleared"
        : patch &&
            fact.state === "value" &&
            (fact.value === "" || (Array.isArray(fact.value) && !fact.value.length))
          ? "Empty"
          : factText(fact, format)}
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

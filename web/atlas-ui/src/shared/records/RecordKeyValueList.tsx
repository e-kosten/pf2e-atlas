import type React from "react";

export type RecordKeyValueItem = {
  key: React.Key;
  label: React.ReactNode;
  value: React.ReactNode;
};

export function RecordKeyValueList({
  ariaLabel,
  className,
  items,
  labelWidth = "compact",
}: {
  ariaLabel?: string;
  className?: string;
  items: RecordKeyValueItem[];
  labelWidth?: "compact" | "provenance";
}) {
  if (!items.length) return null;

  return (
    <dl
      aria-label={ariaLabel}
      className={[
        "record-key-value-list",
        `record-key-value-list--${labelWidth}`,
        className,
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {items.map((item) => (
        <div className="record-key-value-list__row" key={item.key}>
          <dt>{item.label}</dt>
          <dd>{item.value}</dd>
        </div>
      ))}
    </dl>
  );
}

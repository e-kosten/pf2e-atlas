import type React from "react";
import { RecordKeyValueList, type RecordKeyValueItem } from "./RecordKeyValueList";
import { formatSigned, formatSlug } from "./recordFormatting";

export type DefenseStatValue = {
  key: React.Key;
  label: string;
  qualifier?: string;
  signed?: boolean;
  value: number | string | undefined;
};

export type DefenseIwrValue = {
  amount?: number;
  double_vs?: string[];
  exceptions?: string[];
  kind: string;
};

export function DefenseStats({
  ariaLabel,
  values,
}: {
  ariaLabel: string;
  values: DefenseStatValue[];
}) {
  const present = values.filter(
    (value): value is DefenseStatValue & { value: number | string } =>
      value.value !== undefined,
  );
  if (!present.length) return null;
  return (
    <dl aria-label={ariaLabel} className="creature-sheet__defense-stats">
      {present.map((stat) => (
        <div key={stat.key}>
          <dt>{stat.label}</dt>
          <dd className="creature-sheet__fact-value">
            <span>
              {typeof stat.value === "number" && stat.signed
                ? formatSigned(stat.value)
                : stat.value}
            </span>
            {stat.qualifier ? <small>{stat.qualifier}</small> : null}
          </dd>
        </div>
      ))}
    </dl>
  );
}

export function DefenseIwrList({
  immunities,
  resistances,
  weaknesses,
}: {
  immunities?: DefenseIwrValue[];
  resistances?: DefenseIwrValue[];
  weaknesses?: DefenseIwrValue[];
}) {
  const items = [
    iwrItem("immunities", "Immunities", immunities),
    iwrItem("weaknesses", "Weaknesses", weaknesses),
    iwrItem("resistances", "Resistances", resistances),
  ].filter((item): item is RecordKeyValueItem => item !== null);
  return items.length ? (
    <RecordKeyValueList
      ariaLabel="Immunities, weaknesses, and resistances"
      items={items}
    />
  ) : null;
}

export function DefenseNote({ children }: { children: React.ReactNode }) {
  if (children === undefined || children === null || children === "") return null;
  return <p className="creature-sheet__detail-note">{children}</p>;
}

function iwrItem(
  key: React.Key,
  label: string,
  values: DefenseIwrValue[] | undefined,
): RecordKeyValueItem | null {
  if (!values?.length) return null;
  return { key, label, value: values.map(formatIwr).join(", ") };
}

function formatIwr(value: DefenseIwrValue) {
  const exceptions = value.exceptions?.length
    ? ` (except ${value.exceptions.join(", ")})`
    : "";
  const doubled = value.double_vs?.length
    ? `; double vs. ${value.double_vs.join(", ")}`
    : "";
  return `${formatSlug(value.kind)}${value.amount === undefined ? "" : ` ${value.amount}`}${exceptions}${doubled}`;
}

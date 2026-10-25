import {
  destinationHref,
  isPrimaryActivation,
  navigatePath,
} from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { TraitBadge } from "./TraitBadge";
import { Button } from "antd";
import type { RecordSummaryView } from "../../generated/atlas";
import { RecordRarity } from "./RecordRarity";
export function RecordBrowseSummary({
  record,
  selected,
  disabled = false,
  onPreview,
  href,
}: {
  record: RecordSummaryView;
  selected: boolean;
  disabled?: boolean;
  onPreview: (anchor: HTMLElement) => void;
  href?: string;
}) {
  const classification = [
    record.kind_label,
    record.level_label?.replace(/^(Level|Rank)\s+/, ""),
  ]
    .filter(Boolean)
    .join(" ");
  const recordHref =
    href ??
    destinationHref(
      recordDestination({
        recordKey: record.record_key,
      }),
    );
  return (
    <>
      <Button
        className="reference-result__summary"
        styles={{
          root: {
            height: "auto",
            whiteSpace: "normal",
            display: "grid",
            gridTemplateRows: "subgrid",
            gridTemplateColumns: "subgrid",
            gridRow: "1 / 3",
            gridColumn: "1 / -1",
            alignItems: "stretch",
            gap: 7,
            padding: "14px 16px 10px",
            borderRadius: 0,
            textAlign: "left",
            color: selected ? "var(--selection-text)" : undefined,
          },
        }}
        data-record-key={record.record_key}
        href={recordHref}
        block
        color="default"
        variant="text"
        disabled={disabled}
        aria-current={selected ? "true" : undefined}
        onClick={(event) => {
          if (!isPrimaryActivation(event)) return;
          event.preventDefault();
          onPreview(event.currentTarget);
        }}
        onKeyDown={(event) => {
          if (event.key === "Enter" && isPrimaryActivation(event)) {
            event.preventDefault();
            navigatePath(recordHref);
            return;
          }
          if (event.key === " " && isPrimaryActivation(event)) {
            event.preventDefault();
            onPreview(event.currentTarget);
          }
          if (
            ["ArrowDown", "ArrowUp", "j", "k"].includes(event.key) &&
            isPrimaryActivation(event)
          ) {
            event.preventDefault();
            const links = Array.from(
              event.currentTarget
                .closest(".reference-results")
                ?.querySelectorAll<HTMLElement>("[data-record-key]") ?? [],
            );
            const index = links.indexOf(event.currentTarget);
            links[
              Math.max(
                0,
                Math.min(
                  links.length - 1,
                  index + (["ArrowDown", "j"].includes(event.key) ? 1 : -1),
                ),
              )
            ]?.focus();
          }
        }}
      >
        <div className="reference-result__title">
          <strong>{record.title}</strong>
          <span>{classification}</span>
        </div>
        <div className="reference-result__metadata">
          <RecordRarity rarity={record.rarity} />
          {record.unit_price_label && <span>{record.unit_price_label}</span>}
          <span>{record.publication ?? record.pack}</span>
        </div>
      </Button>
      {!!record.traits.length && (
        <div className="reference-result__traits">
          {record.traits
            .filter((trait) => trait.kind !== "rarity")
            .map((trait) => (
              <TraitBadge key={trait.kind + ":" + trait.value} trait={trait} />
            ))}
        </div>
      )}
    </>
  );
}

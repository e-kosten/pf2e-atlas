import { Button, Tag } from "antd";
import type { RecordSummaryView } from "../../generated/atlas";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import { sentenceCase } from "../ui/labels";

export function RecordBrowseSummary({
  record,
  selected,
  disabled = false,
  onPreview,
}: {
  record: RecordSummaryView;
  selected: boolean;
  disabled?: boolean;
  onPreview: (anchor: HTMLElement) => void;
}) {
  const classification = [
    record.kind_label,
    record.level_label?.replace(/^(Level|Rank)\s+/, ""),
  ]
    .filter(Boolean)
    .join(" ");
  const rarity =
    record.rarity && record.rarity !== "common" ? sentenceCase(record.rarity) : null;
  return (
    <Button
      className="reference-result__summary"
      styles={{
        root: {
          height: "auto",
          whiteSpace: "normal",
          display: "grid",
          gridTemplateRows: "subgrid",
          gridTemplateColumns: "subgrid",
          gridRow: "1 / 4",
          gridColumn: "1 / -1",
          alignItems: "stretch",
          gap: 7,
          padding: "14px 16px 10px",
          borderRadius: 0,
          textAlign: "left",
        },
      }}
      data-record-key={record.record_key}
      href={atlasRoutePath({ kind: "record", recordKey: record.record_key })}
      block
      color={selected ? "primary" : "default"}
      variant="text"
      disabled={disabled}
      aria-current={selected ? "true" : undefined}
      onClick={(event) => {
        if (!shouldHandleAtlasRouteClick(event)) return;
        event.preventDefault();
        onPreview(event.currentTarget);
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter" && shouldHandleAtlasRouteClick(event)) {
          event.preventDefault();
          navigateToAtlasRoute({ kind: "record", recordKey: record.record_key });
          return;
        }
        if (event.key === " " && shouldHandleAtlasRouteClick(event)) {
          event.preventDefault();
          onPreview(event.currentTarget);
        }
        if (
          ["ArrowDown", "ArrowUp", "j", "k"].includes(event.key) &&
          shouldHandleAtlasRouteClick(event)
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
        {rarity && <span>{rarity}</span>}
        <span>{record.publication ?? record.pack}</span>
      </div>
      {!!record.traits.length && (
        <div className="reference-result__traits">
          {record.traits
            .filter((trait) => trait.kind !== "rarity")
            .map((trait) => (
              <Tag key={trait.kind + ":" + trait.value}>
                {sentenceCase(trait.label)}
              </Tag>
            ))}
        </div>
      )}
    </Button>
  );
}

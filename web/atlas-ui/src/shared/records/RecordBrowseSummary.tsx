import { RecordEditionBadge } from "./RecordEditionBadge";
import {
  destinationHref,
  isPrimaryActivation,
  navigatePath,
} from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Button } from "antd";
import type { ReactNode } from "react";
import type { RecordSummaryView } from "../../generated/atlas";
import { RecordIdentityBadges } from "./RecordIdentityBadges";
import { RecordRarity } from "./RecordRarity";
export function RecordBrowseSummary({
  record,
  selected,
  disabled = false,
  onPreview,
  href,
  context,
  qualifiers,
  searchCard = false,
}: {
  record: RecordSummaryView;
  selected: boolean;
  disabled?: boolean;
  onPreview: (anchor: HTMLElement) => void;
  href?: string;
  context?: ReactNode;
  qualifiers?: string[];
  searchCard?: boolean;
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
  const scenario = qualifiers === undefined ? record.creature_display : undefined;
  const source = record.publication ?? record.pack;
  const showSource =
    qualifiers === undefined &&
    source &&
    !(
      scenario?.scenario_context && scenario.uses.some(({ label }) => label === source)
    );
  const displayQualifiers = qualifiers ?? record.creature_display?.qualifiers ?? [];
  const metadata = [
    ...new Set(
      [
        scenario?.scenario_context,
        record.unit_price_label,
        showSource ? source : undefined,
        ...displayQualifiers,
      ].filter((value): value is string => Boolean(value)),
    ),
  ];
  return (
    <Button
      className={
        searchCard
          ? "reference-result__summary reference-result__summary--search"
          : "reference-result__summary"
      }
      styles={{
        root: {
          height: "auto",
          whiteSpace: "normal",
          alignItems: "stretch",
          gap: 7,
          padding: "14px var(--browse-pane-inset) 10px",
          // Include the parent's selection border inside the pane gutter.
          paddingInlineStart: "calc(var(--browse-pane-inset) - 3px)",
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
              .closest('[role="list"], .reference-results')
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
        <strong>{record.title}</strong> <span>{classification}</span>
      </div>
      {(!searchCard || metadata.length > 0 || context) && (
        <div className="reference-result__metadata">
          {!searchCard && (
            <>
              <RecordRarity rarity={record.rarity} linked={false} />
              <RecordEditionBadge edition={record.edition} />
            </>
          )}
          {searchCard
            ? metadata.length > 0 && <span>{metadata.join(" · ")}</span>
            : metadata.map((value) => <span key={value}>{value}</span>)}
          {context && <span>{context}</span>}
        </div>
      )}
      <RecordIdentityBadges
        identity={searchCard ? record : { traits: record.traits }}
        linked={false}
        className="reference-result__traits"
      />
    </Button>
  );
}

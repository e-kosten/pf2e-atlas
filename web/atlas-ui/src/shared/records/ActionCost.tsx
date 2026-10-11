import type { ActionCostView, FactView } from "../../generated/atlas";
import { FactRow } from "./presentationFacts";

/** Render authored meaning; glyph choice is already resolved by Rust. */
export function ActionCost({
  fact,
  label,
  patch = false,
}: {
  fact: FactView<ActionCostView>;
  label?: string;
  patch?: boolean;
}) {
  if (fact.state === "not_applicable" || (patch && fact.state === "missing"))
    return null;
  const cost = fact.state === "value" ? fact.value : null;
  if (cost?.label === "" && !patch) return null;
  const fallback =
    patch && fact.state === "null"
      ? "Cleared"
      : cost?.label === ""
        ? "Empty"
        : fact.state === "missing"
          ? label
            ? "Not listed"
            : "Action cost not listed"
          : fact.state === "null"
            ? label
              ? "Not specified"
              : "Action cost not specified"
            : label
              ? "Unavailable"
              : "Action cost unavailable";
  const content = (
    <span className="action-cost" title={cost?.label || undefined}>
      {cost?.glyph && (
        <span
          className="action-cost__glyph"
          aria-hidden="true"
          data-atlas-action-glyph={cost.glyph}
        />
      )}
      <span className={cost?.glyph ? "sr-only" : undefined}>
        {cost?.label || fallback}
      </span>
    </span>
  );
  return label ? <FactRow label={label}>{content}</FactRow> : content;
}

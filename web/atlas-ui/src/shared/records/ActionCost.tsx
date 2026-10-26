import type { ActionCostView, FactView } from "../../generated/atlas";
import {
  FactRow,
  hasReadingFact,
  isEmptyFactValue,
  type FactValueRenderer,
} from "./presentationFacts";

/** Render authored meaning; glyph choice is already resolved by Rust. */
export function ActionCost({
  fact,
  label,
  patch = false,
  renderValue,
}: {
  fact: FactView<ActionCostView>;
  label?: string;
  patch?: boolean;
  renderValue?: FactValueRenderer;
}) {
  if (!hasReadingFact(fact, { patch })) return null;
  const cost = fact.state === "value" ? fact.value : null;
  if (cost && isEmptyFactValue(cost.label) && !patch) return null;
  const fallback =
    patch && fact.state === "null"
      ? "Cleared"
      : cost && isEmptyFactValue(cost.label)
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
        {cost && !isEmptyFactValue(cost.label) ? cost.label : fallback}
      </span>
    </span>
  );
  return label ? (
    <FactRow label={label}>{renderValue ? renderValue(content) : content}</FactRow>
  ) : (
    <>{renderValue ? renderValue(content) : content}</>
  );
}

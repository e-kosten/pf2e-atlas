import type { CreatureSurfaceActionCostView } from "../../generated/atlas";

type FutureActionCost = {
  cost_type: string;
  count?: number;
  value?: string;
};

export function ActionGlyph({
  cost,
}: {
  cost: CreatureSurfaceActionCostView | FutureActionCost | null | undefined;
}) {
  const presentation = actionCostPresentation(cost);
  return (
    <span className="action-glyph" data-action-kind={presentation.kind}>
      {presentation.glyph ? (
        <span aria-hidden="true" className="action-glyph__mark">
          {presentation.glyph}
        </span>
      ) : null}
      <span className="action-glyph__label">{presentation.label}</span>
    </span>
  );
}

function actionCostPresentation(
  cost: CreatureSurfaceActionCostView | FutureActionCost | null | undefined,
): { glyph?: string; kind: string; label: string } {
  if (!cost) {
    return { kind: "missing", label: "The action cost is not listed." };
  }
  switch (cost.cost_type) {
    case "actions":
      if (typeof cost.count === "number" && cost.count >= 1 && cost.count <= 3) {
        return {
          glyph: cost.count.toString(),
          kind: `actions-${cost.count}`,
          label:
            cost.count === 1
              ? "One action"
              : cost.count === 2
                ? "Two actions"
                : "Three actions",
        };
      }
      return {
        kind: "actions-unusual",
        label:
          typeof cost.count === "number"
            ? `${cost.count} actions`
            : "The action count is not listed.",
      };
    case "free_action":
      return { glyph: "F", kind: "free-action", label: "Free action" };
    case "reaction":
      return { glyph: "R", kind: "reaction", label: "Reaction" };
    case "passive":
      return { kind: "passive", label: "Passive" };
    case "time":
      return {
        kind: "time",
        label: cost.value || "The action time is not listed.",
      };
    default:
      return { kind: "unknown", label: "The action cost is not recognized." };
  }
}

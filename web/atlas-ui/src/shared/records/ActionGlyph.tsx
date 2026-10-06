import type {
  CreatureSurfaceActionCostView,
  EncounterRuntimeActionCostKindView,
} from "../../generated/atlas";

type FutureActionCost = {
  cost_type: string;
  count?: number;
  value?: string;
};

export type ActionCost =
  | CreatureSurfaceActionCostView
  | EncounterRuntimeActionCostKindView
  | FutureActionCost;

export function ActionGlyph({ cost }: { cost: ActionCost | null | undefined }) {
  const presentation = actionCostPresentation(cost);
  return (
    <span className="action-glyph" data-action-kind={presentation.kind}>
      {presentation.glyph ? (
        <span aria-hidden="true" className="action-glyph__mark">
          {presentation.glyph}
        </span>
      ) : null}
      <span className={presentation.glyph ? "sr-only" : "action-glyph__label"}>
        {presentation.label}
      </span>
    </span>
  );
}

export function actionCostLabel(cost: ActionCost | null | undefined) {
  return actionCostPresentation(cost).label;
}

function actionCostPresentation(cost: ActionCost | null | undefined): {
  glyph?: string;
  kind: string;
  label: string;
} {
  if (!cost) {
    return { kind: "missing", label: "The action cost is not listed." };
  }
  const costType = "kind" in cost ? cost.kind : cost.cost_type;
  const count = "count" in cost ? cost.count : undefined;
  const value = "value" in cost ? cost.value : undefined;
  switch (costType) {
    case "actions":
      if (typeof count === "number" && count >= 1 && count <= 3) {
        return {
          glyph: count.toString(),
          kind: `actions-${count}`,
          label:
            count === 1 ? "One action" : count === 2 ? "Two actions" : "Three actions",
        };
      }
      return {
        kind: "actions-unusual",
        label:
          typeof count === "number"
            ? `${count} actions`
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
        label: value || "The action time is not listed.",
      };
    default:
      return { kind: "unknown", label: "The action cost is not recognized." };
  }
}

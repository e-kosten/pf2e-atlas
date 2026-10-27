import type { ReactNode } from "react";
import type {
  RollOutcome,
  RollUnitView,
  RollExpression,
  RollPreviewExpression,
} from "../../generated/atlas";
import { DieFace, type DiceDrawing } from "./DieFace";
import type { DiceMotion } from "./motion";
import { diceAppearance } from "./appearance";
import "../../styles/ui/dice.css";

export function RollResult({
  outcome,
  unit,
  pending = false,
  drawing = diceAppearance.drawing,
  motion = diceAppearance.motion,
}: {
  outcome: RollOutcome;
  unit?: RollUnitView;
  pending?: boolean;
  drawing?: DiceDrawing;
  motion?: DiceMotion;
}) {
  return (
    <span
      className="atlas-roll"
      role="status"
      aria-label={
        pending
          ? "Rolling"
          : `${outcome.formula}: ${outcome.total}${unit ? ` ${unitLabel(unit, outcome.total)}` : ""}`
      }
    >
      <Expression
        pending={pending}
        value={outcome.expression}
        drawing={drawing}
        motion={motion}
      />
      <span className="atlas-roll__result">
        <span aria-hidden="true">=</span>
        <span className="atlas-roll__quantity">
          <strong className="atlas-roll__total">{pending ? " " : outcome.total}</strong>
          {unit && <span>{unitLabel(unit, pending ? undefined : outcome.total)}</span>}
        </span>
      </span>
    </span>
  );
}
/** Same expression geometry as a result, with blank dice and an explicit Roll action. */
export function UnrolledRoll({
  expression,
  formula,
  unit,
  pending = false,
  children,
}: {
  expression: RollPreviewExpression;
  formula: string;
  unit?: RollUnitView;
  pending?: boolean;
  children: ReactNode;
}) {
  return (
    <span
      className="atlas-roll"
      role={pending ? "status" : "group"}
      aria-label={`${pending ? "Rolling" : "Unrolled"} ${formula}${unit ? ` ${unit.plural}` : ""}`}
    >
      <Expression
        value={expression}
        pending={pending}
        drawing={diceAppearance.drawing}
        motion={diceAppearance.motion}
      />
      <span className="atlas-roll__result">
        <span aria-hidden="true">=</span>
        <span className="atlas-roll__quantity">
          <span className="atlas-roll__total">{children}</span>
          {unit && <span>{unit.plural}</span>}
        </span>
      </span>
    </span>
  );
}
function Expression({
  value,
  pending,
  drawing,
  motion,
  minimumPrecedence = 0,
}: {
  value: RollExpression | RollPreviewExpression;
  pending: boolean;
  drawing: DiceDrawing;
  motion: DiceMotion;
  minimumPrecedence?: number;
}) {
  switch (value.kind) {
    case "number":
      return <span>{value.value}</span>;
    case "dice": {
      const dice =
        "dice" in value
          ? value.dice
          : value.sides.map((sides) => ({ sides, value: undefined, kept: true }));
      return (
        <>
          {dice.map((d, i) => (
            <DieFace
              key={i}
              {...d}
              pending={pending}
              kept={pending || d.kept}
              drawing={drawing}
              motion={motion}
            />
          ))}
        </>
      );
    }
    case "negate":
      return (
        <>
          −
          <Expression
            pending={pending}
            value={value.inner}
            drawing={drawing}
            motion={motion}
            minimumPrecedence={3}
          />
        </>
      );
    case "binary": {
      const precedence =
        value.operator === "add" || value.operator === "subtract" ? 1 : 2;
      const grouped = precedence < minimumPrecedence;
      return (
        <>
          {grouped && "("}
          <Expression
            pending={pending}
            value={value.left}
            drawing={drawing}
            motion={motion}
            minimumPrecedence={precedence}
          />
          <span>
            {{ add: "+", subtract: "−", multiply: "×", divide: "÷" }[value.operator]}
          </span>
          <Expression
            pending={pending}
            value={value.right}
            drawing={drawing}
            motion={motion}
            minimumPrecedence={precedence + (value.operator === "add" ? 0 : 1)}
          />
          {grouped && ")"}
        </>
      );
    }
  }
}

function unitLabel(unit: RollUnitView, total?: number) {
  return total !== undefined && Math.abs(total) === 1 ? unit.singular : unit.plural;
}

import type { ReactNode } from "react";
import type {
  RollOutcome,
  RollExpression,
  RollPreviewExpression,
} from "../../generated/atlas";
import { DieFace, type DiceDrawing } from "./DieFace";
import type { DiceMotion } from "./motion";
import { diceAppearance } from "./appearance";
import "../../styles/ui/dice.css";

export function RollResult({
  outcome,
  pending = false,
  drawing = diceAppearance.drawing,
  motion = diceAppearance.motion,
}: {
  outcome: RollOutcome;
  pending?: boolean;
  drawing?: DiceDrawing;
  motion?: DiceMotion;
}) {
  return (
    <span
      className="atlas-roll"
      role="status"
      aria-label={pending ? "Rolling" : `${outcome.formula}: ${outcome.total}`}
    >
      <Expression
        pending={pending}
        value={outcome.expression}
        drawing={drawing}
        motion={motion}
      />
      <span className="atlas-roll__result">
        <span aria-hidden="true">=</span>
        <strong className="atlas-roll__total">{pending ? " " : outcome.total}</strong>
      </span>
    </span>
  );
}
/** Same expression geometry as a result, with blank dice and an explicit Roll action. */
export function UnrolledRoll({
  expression,
  formula,
  pending = false,
  children,
}: {
  expression: RollPreviewExpression;
  formula: string;
  pending?: boolean;
  children: ReactNode;
}) {
  return (
    <span
      className="atlas-roll"
      role={pending ? "status" : "group"}
      aria-label={`${pending ? "Rolling" : "Unrolled"} ${formula}`}
    >
      <Expression
        value={expression}
        pending={pending}
        drawing={diceAppearance.drawing}
        motion={diceAppearance.motion}
      />
      <span className="atlas-roll__result">
        <span aria-hidden="true">=</span>
        <span className="atlas-roll__total">{children}</span>
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

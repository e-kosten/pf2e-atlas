import { useEffect, useRef, useSyncExternalStore } from "react";
import { projectedDice } from "./projected-dice";
import { diceAppearance } from "./appearance";
import { startDiceMotion, type DiceMotion } from "./motion";
import "../../styles/ui/dice.css";

export type DiceDrawing = "simple" | "shaded";
const outlines: Record<number, string> = {
  4: "M24 3 L46 43 L2 43 Z",
  6: "M4 4 H44 V44 H4 Z",
  8: "M24 2 L46 24 L24 46 L2 24 Z",
  10: "M24 2 L46 23 L24 46 L2 23 Z",
  12: "M24 2 L46 18 L37 44 L11 44 L2 18 Z",
  20: "M24 2 L44 13 L44 35 L24 46 L4 35 L4 13 Z",
};
const edges: Record<number, string> = {
  8: "M24 2 V12 M24 36 V46",
  10: "M24 2 V10 M24 37 V46",
  12: "M24 10 L38 20 L33 36 L15 36 L10 20 Z M24 2 V10",
  20: "M24 2 L4 35 H44 Z",
};
const media = () => window.matchMedia("(prefers-reduced-motion: reduce)");
function subscribe(listener: () => void) {
  const query = media();
  query.addEventListener("change", listener);
  return () => query.removeEventListener("change", listener);
}
const reduced = () => media().matches;

export function DieFace({
  sides,
  value,
  pending = false,
  kept = true,
  drawing = diceAppearance.drawing,
  motion = diceAppearance.motion,
}: {
  sides: number;
  value?: number;
  pending?: boolean;
  kept?: boolean;
  drawing?: DiceDrawing;
  motion?: DiceMotion;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  const running = useRef<ReturnType<typeof startDiceMotion> | null>(null);
  const reduce = useSyncExternalStore(subscribe, reduced, () => true);
  useEffect(() => {
    if (reduce || motion === "none") {
      running.current?.cancel();
      return;
    }
    if (pending && ref.current) {
      running.current?.cancel();
      running.current = startDiceMotion(ref.current, motion);
    } else running.current?.settle();
  }, [pending, motion, reduce]);
  useEffect(() => () => running.current?.cancel(), []);
  const nonstandard = !projectedDice[sides];
  const drawingSides = nonstandard ? diceAppearance.fallbackSides : sides;
  const solid = projectedDice[drawingSides];
  const display = pending ? "" : (value ?? "");
  return (
    <span
      className={`atlas-die ${nonstandard ? "atlas-die--nonstandard" : ""} ${kept ? "" : "atlas-die--discarded"}`}
      aria-label={
        pending
          ? `Rolling d${sides}`
          : value === undefined
            ? `Unrolled d${sides}`
            : `d${sides}: ${display}${kept ? "" : ", discarded"}`
      }
    >
      <span ref={ref} className="atlas-die__drawing" aria-hidden="true">
        <svg viewBox="0 0 48 48" focusable="false">
          {drawing === "shaded" && solid ? (
            <>
              {solid.faces.map((face, i) => (
                <g key={i}>
                  <polygon points={face.points} />
                  <polygon
                    className="atlas-die__shade"
                    points={face.points}
                    style={{ fillOpacity: face.shade }}
                  />
                </g>
              ))}
              <text
                className="atlas-die__value"
                style={sides >= 100 ? { fontSize: sides >= 1000 ? 9 : 11 } : undefined}
                x={solid.label[0]}
                y={solid.label[1]}
                textAnchor="middle"
                dominantBaseline="central"
              >
                {display}
              </text>
            </>
          ) : (
            <>
              <path d={outlines[drawingSides]} />
              {edges[drawingSides] && (
                <path className="atlas-die__edge" d={edges[drawingSides]} />
              )}
              <text
                className="atlas-die__value"
                x="24"
                y="25"
                textAnchor="middle"
                dominantBaseline="central"
              >
                {display}
              </text>
            </>
          )}
        </svg>
      </span>
      {nonstandard && (
        <small className="atlas-die__sides" aria-hidden="true">
          d{sides}
        </small>
      )}
    </span>
  );
}

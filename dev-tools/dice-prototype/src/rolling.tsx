import { useEffect, useRef, useState } from "react";
import { Button, Popover } from "antd";
import { DieFace, type DiceFinish, type DiceMotion } from './die-face';

export type Group = { label: string; formula: string };
export type Die = { sides: number; value: number; kept: boolean };
export type Result = Group & {
  total: number;
  dice: Die[];
  breakdown: string;
  modifier?: number;
};
export type Receipt = { id: string; groups: Result[]; total: number };
export type Settings = {
  animate: boolean;
  delay: number;
  fail: boolean;
  reduced: boolean;
};
export function useRoll(settings: Settings, onComplete: (r: Receipt) => void) {
  const [pending, setPending] = useState(false);
  const [slow, setSlow] = useState(false);
  const [error, setError] = useState("");
  const active = useRef(false);
  const retry = useRef<{ id: string; groups: Group[] } | undefined>(undefined);
  const complete = useRef(onComplete);
  complete.current = onComplete;
  const mounted = useRef(true);
  useEffect(
    () => () => {
      mounted.current = false;
    },
    [],
  );
  async function roll(groups: Group[], recover = false) {
    if (active.current) return;
    const request =
      recover && retry.current
        ? retry.current
        : { id: crypto.randomUUID(), groups };
    retry.current = request;
    active.current = true;
    setPending(true);
    setError("");
    const timer = window.setTimeout(() => setSlow(true), 1200);
    const started = performance.now();
    try {
      const response = await fetch("/roll", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          ...request,
          delay_ms: settings.delay,
          fail: settings.fail && !recover,
        }),
      });
      if (!response.ok) throw new Error("Roll unavailable. Try again.");
      const receipt = (await response.json()) as Receipt;
      const remaining =
        settings.animate && !settings.reduced
          ? Math.max(0, 700 - (performance.now() - started))
          : 0;
      if (remaining)
        await new Promise((resolve) => setTimeout(resolve, remaining));
      if (mounted.current) complete.current(receipt);
    } catch {
      if (mounted.current) setError("Roll unavailable.");
    } finally {
      clearTimeout(timer);
      active.current = false;
      if (mounted.current) {
        setPending(false);
        setSlow(false);
      }
    }
  }
  return { pending, slow, error, roll };
}

export function RollResult({
  result,
  pending,
  sides = [20],
  animate,
  slow,
  motion = 'tumble',
  finish = 'plain',
}: {
  result?: Result;
  pending: boolean;
  sides?: number[];
  animate: boolean;
  slow?: boolean;
  motion?: DiceMotion;
  finish?: DiceFinish;
}) {
  const dice =
    pending || !result
      ? sides.map((s) => ({ sides: s, value: 0, kept: true }))
      : result.dice;
  return (
    <div className={`roll-result motion-${motion}`} aria-busy={pending}>
      <div className="dice-line" aria-hidden="true">
        {dice?.map((die, i) => (
          <span
            key={i}
            className={`die ${pending && animate ? "rolling" : ""} ${!die.kept ? "discarded" : ""}`}
          >
            <DieFace sides={die.sides} value={result ? die.value : undefined}
              pending={pending} animate={animate} motion={motion} finish={finish}
              unrolled={!pending && !result} />
            <span className="die-caption">
              d{die.sides}
              {!die.kept ? " · dropped" : ""}
            </span>
          </span>
        ))}
        <span className="roll-total">
          {!pending && result && (
            <>
              {result.modifier ? (
                <small>
                  {result.modifier > 0 ? "+" : "−"}
                  {Math.abs(result.modifier)} ={" "}
                </small>
              ) : (
                <small>= </small>
              )}
              {result.total}
            </>
          )}
        </span>
      </div>
      <span role="status" className="sr-only">
        {pending
          ? "Rolling"
          : result
            ? `${result.label}: ${result.breakdown}. Total ${result.total}`
            : ""}
      </span>
      {slow ? (
        <span className="muted">Still rolling…</span>
      ) : !pending && result ? (
        <Popover
          trigger="click"
          content={
            <div className="breakdown">
              <strong>{result.label}</strong>
              <p>{result.formula}</p>
              <p>{result.breakdown}</p>
              <p>
                {result.dice
                  .map(
                    (d) =>
                      `d${d.sides}: ${d.value}${d.kept ? "" : " (dropped)"}`,
                  )
                  .join(" · ")}
              </p>
            </div>
          }
        >
          <Button type="link" size="small">
            Breakdown
          </Button>
        </Popover>
      ) : (
        <span className="result-spacer" />
      )}
    </div>
  );
}

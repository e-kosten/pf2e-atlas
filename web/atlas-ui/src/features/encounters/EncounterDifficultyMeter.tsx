import { Progress, theme } from "antd";
import { useLayoutEffect, useRef } from "react";
import type { EncounterDifficultyView } from "../../generated/atlas";

/** XP positions are linear; the published Extreme budget is the full scale. */
export function EncounterDifficultyMeter({
  value,
  compact = false,
}: {
  value: EncounterDifficultyView;
  compact?: boolean;
}) {
  const { token } = theme.useToken();
  const maximum = Math.max(0, ...value.budgets.map((budget) => budget.xp));
  const axis = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const element = axis.current;
    if (!element) return;
    // Large parties can put distinct XP thresholds very close together. Keep
    // their marks, but omit colliding numbers; the calculator lists every budget.
    const fitLabels = () => {
      const labels = Array.from(element.children) as HTMLElement[];
      const last = labels[labels.length - 1];
      if (!last) return;
      const finalBounds = last.getBoundingClientRect();
      let previousRight = -Infinity;
      for (const label of labels) {
        const bounds = label.getBoundingClientRect();
        const visible =
          label === last ||
          (bounds.left >= previousRight + 4 && bounds.right + 4 <= finalBounds.left);
        label.style.visibility = visible ? "visible" : "hidden";
        if (visible) previousRight = bounds.right;
      }
    };
    fitLabels();
    const observer = new ResizeObserver(fitLabels);
    observer.observe(element);
    for (const label of Array.from(element.children)) observer.observe(label);
    return () => observer.disconnect();
  }, [value.budgets]);
  if (!maximum) return null;
  const thresholds = [...new Set([0, ...value.budgets.map((budget) => budget.xp)])]
    .filter((xp) => xp >= 0)
    .sort((left, right) => left - right);
  const color =
    value.rating === "Trivial" || value.rating === "Low"
      ? token.colorSuccess
      : value.rating === "Moderate"
        ? token.colorWarning
        : "var(--hp-critical)";
  const text =
    value.total_xp === null
      ? "Difficulty incomplete"
      : `${value.rating}, ${value.total_xp} XP; Extreme budget ${maximum} XP`;
  return (
    <div
      className={`encounter-difficulty-meter${compact ? " encounter-difficulty-meter--compact" : ""}`}
    >
      <div className="encounter-difficulty-meter__track">
        {value.total_xp === null ? (
          <div
            className="encounter-difficulty-meter__unavailable"
            role="img"
            aria-label={text}
            style={{ background: token.colorFillSecondary }}
          />
        ) : (
          <Progress
            className="encounter-difficulty-meter__progress"
            percent={Math.min(100, (value.total_xp / maximum) * 100)}
            status="normal"
            showInfo={false}
            strokeColor={color}
            railColor={token.colorFillSecondary}
            size={{ height: 10 }}
            aria-label="Encounter XP"
            aria-valuetext={text}
          />
        )}
        <div className="encounter-difficulty-meter__ticks" aria-hidden="true">
          {value.budgets
            .filter((budget) => budget.xp > 0)
            .map((budget) => (
              <span
                key={budget.label}
                style={{ left: `${(budget.xp / maximum) * 100}%` }}
              />
            ))}
        </div>
      </div>
      <div ref={axis} className="encounter-difficulty-meter__axis" aria-hidden="true">
        {thresholds.map((xp) => (
          <span key={xp} style={{ left: `${(xp / maximum) * 100}%` }}>
            {xp}
          </span>
        ))}
      </div>
    </div>
  );
}

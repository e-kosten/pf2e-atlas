import { Tooltip, theme } from "antd";
import { useLayoutEffect, useRef } from "react";
import type { EncounterDifficultyView } from "../../generated/atlas";

/** XP positions are linear; the published Extreme budget is the full scale. */
export function EncounterDifficultyMeter({
  value,
  compact = false,
  current,
}: {
  value: EncounterDifficultyView;
  compact?: boolean;
  current?: EncounterDifficultyView;
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
        label.toggleAttribute("data-colliding", !visible);
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
  const ratingColor = (rating: string | null) =>
    rating === "Trivial" || rating === "Low"
      ? token.colorSuccess
      : rating === "Moderate"
        ? token.colorWarning
        : "var(--hp-critical)";
  const color = ratingColor(value.rating);
  const end = value.total_xp;
  const percent = (xp: number) => Math.min(100, (xp / maximum) * 100);
  const total = end === null ? 0 : percent(end);
  const solid = Math.min(total, percent(current?.total_xp ?? end ?? 0));
  const additional = total - solid;
  const text =
    value.total_xp === null
      ? "Difficulty incomplete"
      : `${value.rating}, ${value.total_xp} XP; Extreme budget ${maximum} XP`;
  return (
    <div
      className={`encounter-difficulty-meter${compact ? " encounter-difficulty-meter--compact" : ""}`}
    >
      <div
        className="encounter-difficulty-meter__track"
        role={end === null ? "img" : "progressbar"}
        aria-label={end === null ? text : "Encounter XP"}
        aria-valuemin={end === null ? undefined : 0}
        aria-valuemax={end === null ? undefined : maximum}
        aria-valuenow={end === null ? undefined : Math.min(end, maximum)}
        aria-valuetext={end === null ? undefined : text}
        style={{ backgroundColor: token.colorFillSecondary }}
      >
        <span
          className="encounter-difficulty-meter__solid"
          style={{
            width: `${solid}%`,
            backgroundColor: ratingColor(current?.rating ?? value.rating),
          }}
        />
        <span
          className="encounter-difficulty-meter__additional"
          style={{ width: `${additional}%`, color }}
        />
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
      <div ref={axis} className="encounter-difficulty-meter__axis">
        {thresholds.map((xp) => {
          const label = value.budgets
            .filter((budget) => budget.xp === xp)
            .map((budget) => `${budget.label} ${xp} XP`)
            .join(" / ");
          const number = (
            <span aria-hidden="true" className="encounter-difficulty-meter__number">
              {xp}
            </span>
          );
          return (
            <span key={xp} style={{ left: `${(xp / maximum) * 100}%` }}>
              {label ? (
                <Tooltip title={label} trigger={["hover", "focus"]}>
                  <button
                    type="button"
                    aria-label={label}
                    className="encounter-difficulty-meter__threshold"
                  >
                    {number}
                  </button>
                </Tooltip>
              ) : (
                number
              )}
            </span>
          );
        })}
      </div>
    </div>
  );
}

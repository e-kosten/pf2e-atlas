import { Progress, theme } from "antd";
import type { CSSProperties } from "react";
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
  if (!maximum) return null;
  const color =
    value.rating === "Trivial" || value.rating === "Low"
      ? token.colorSuccess
      : value.rating === "Moderate"
        ? token.colorWarning
        : token.colorError;
  const text =
    value.total_xp === null
      ? "Difficulty incomplete"
      : `${value.rating}, ${value.total_xp} XP; Extreme budget ${maximum} XP`;
  return (
    <div
      className={`encounter-difficulty-meter${compact ? " encounter-difficulty-meter--compact" : ""}`}
      style={{ "--difficulty-color": color } as CSSProperties}
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
      <div
        className="encounter-difficulty__budgets"
        role="group"
        aria-label="XP budgets"
      >
        {value.budgets.map((budget) => (
          <div
            key={budget.label}
            className={
              value.rating === budget.label
                ? "encounter-difficulty-meter__current"
                : undefined
            }
          >
            <span>{budget.label}</span>
            <strong>{budget.xp}</strong>
          </div>
        ))}
      </div>
    </div>
  );
}

import { Button, Collapse, Form, Input } from "antd";
import { useState } from "react";
import type {
  EncounterParticipantView,
  EncounterHpAction,
} from "../../generated/atlas";
import { asNumber, displayNumber } from "./participantEdits";

const hpOriginLabels: Record<string, string> = {
  derived_pristine: "Derived maximum and current HP",
  derived_edited: "Derived maximum; current HP edited",
  explicit: "HP set manually",
  unknown: "HP unavailable",
};
const adjustmentOriginLabels: Record<string, string> = {
  default_unadjusted: "Unadjusted by default",
  inherited_known: "Adjustment from source",
  inherited_unknown: "Source adjustment unavailable",
  explicit: "Adjustment selected",
};

export function EncounterHpControls({
  current,
  disabled = false,
  onAction,
}: {
  current: EncounterParticipantView;
  disabled?: boolean;
  onAction: (action: EncounterHpAction) => Promise<boolean>;
}) {
  const [hpDraft, setHpDraft] = useState<{
    participantKey: string;
    value: string;
  } | null>(null);
  const [tempHpDraft, setTempHpDraft] = useState<{
    participantKey: string;
    value: string;
  } | null>(null);
  const [amountDraft, setAmountDraft] = useState<{
    participantKey: string;
    value: string;
  } | null>(null);
  const hpInput =
    hpDraft?.participantKey === current.participant_key
      ? hpDraft.value
      : (current.current_hp?.toString() ?? "");
  const tempHpInput =
    tempHpDraft?.participantKey === current.participant_key
      ? tempHpDraft.value
      : (current.temporary_hp?.toString() ?? "");
  const amount =
    amountDraft?.participantKey === current.participant_key ? amountDraft.value : "";
  const hpKnown = current.current_hp !== undefined;
  const maxKnown = current.max_hp !== undefined;
  const currentHp = asNumber(current.current_hp);
  const maxHp = asNumber(current.max_hp);
  const temporaryHp = asNumber(current.temporary_hp);
  const missingHp = Math.max(0, maxHp - currentHp);
  const hpMeterTotal = Math.max(maxHp, currentHp + missingHp + temporaryHp);
  const hpPercent =
    hpMeterTotal > 0 ? Math.min(100, Math.max(0, (currentHp / hpMeterTotal) * 100)) : 0;
  const missingHpPercent =
    hpMeterTotal > 0 ? Math.min(100, Math.max(0, (missingHp / hpMeterTotal) * 100)) : 0;
  const tempHpPercent =
    hpMeterTotal > 0
      ? Math.min(100, Math.max(0, (temporaryHp / hpMeterTotal) * 100))
      : 0;
  const hpRatio = maxHp > 0 ? currentHp / maxHp : 1;
  const hpSummary = hpLabel(current);
  const hpMeterTone =
    hpRatio <= 0.25
      ? "encounter-hp-meter__current--critical"
      : hpRatio <= 0.5
        ? "encounter-hp-meter__current--bloodied"
        : "";

  const applyHpInput = () => {
    if (disabled) return;
    void onAction({ kind: "set_current", input: hpInput.trim() ? hpInput : null })
      .then((saved) => {
        if (saved) setHpDraft(null);
      })
      .catch(() => {});
  };
  const applyTempHpInput = () => {
    if (disabled) return;
    void onAction({ kind: "set_temporary", input: tempHpInput })
      .then((saved) => {
        if (saved) setTempHpDraft(null);
      })
      .catch(() => {});
  };
  const applyAmount = (kind: "damage" | "heal") => {
    if (disabled || !hpKnown || !amount.trim()) return;
    void onAction({ kind, amount })
      .then((saved) => {
        if (saved) setAmountDraft(null);
      })
      .catch(() => {});
  };

  return (
    <div className="encounter-hp-panel">
      <div className="encounter-hp-panel__header">
        <h3>HP</h3>
        <span>{hpSummary}</span>
      </div>
      {hpKnown && maxKnown && (
        <div className="encounter-hp-meter" aria-label={`HP remaining: ${hpSummary}`}>
          <span
            className={["encounter-hp-meter__current", hpMeterTone]
              .filter(Boolean)
              .join(" ")}
            style={{ width: `${hpPercent}%` }}
          />
          {missingHp > 0 && (
            <span
              className="encounter-hp-meter__missing"
              style={{ left: `${hpPercent}%`, width: `${missingHpPercent}%` }}
            />
          )}
          {temporaryHp > 0 && (
            <span
              className="encounter-hp-meter__temporary"
              style={{
                left: `${hpPercent + missingHpPercent}%`,
                width: `${tempHpPercent}%`,
              }}
            />
          )}
        </div>
      )}
      <div className="encounter-hp-grid">
        <div className="encounter-hp-control encounter-hp-control--wide">
          <span className="encounter-hp-control__label">Amount (zero or greater)</span>
          <div className="encounter-hp-control__row">
            <Form.Item style={{ marginBottom: 0 }} layout="vertical">
              <Input
                style={{ width: "100%" }}
                aria-label="HP change"
                disabled={disabled || !hpKnown}
                value={amount}
                onChange={(event) =>
                  setAmountDraft({
                    participantKey: current.participant_key,
                    value: event.target.value,
                  })
                }
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                  }
                }}
              />
            </Form.Item>
            <div className="encounter-hp-actions">
              <Button
                disabled={disabled || !hpKnown || !amount.trim()}
                onClick={() => applyAmount("damage")}
              >
                Damage
              </Button>
              <Button
                disabled={disabled || !hpKnown || !amount.trim()}
                onClick={() => applyAmount("heal")}
              >
                Heal
              </Button>
            </div>
          </div>
        </div>
        <div className="encounter-hp-control">
          <span className="encounter-hp-control__label">HP</span>
          <div className="encounter-hp-control__row">
            <Form.Item style={{ marginBottom: 0 }} layout="vertical">
              <Input
                disabled={disabled}
                aria-label="HP"
                value={hpInput}
                onChange={(event) =>
                  setHpDraft({
                    participantKey: current.participant_key,
                    value: event.target.value,
                  })
                }
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    applyHpInput();
                  }
                }}
              />
            </Form.Item>
            <div className="encounter-hp-actions">
              <Button disabled={disabled} onClick={applyHpInput}>
                Set
              </Button>
            </div>
          </div>
        </div>
        <div className="encounter-hp-control">
          <span className="encounter-hp-control__label">Temp HP</span>
          <div className="encounter-hp-control__row">
            <Form.Item style={{ marginBottom: 0 }} layout="vertical">
              <Input
                disabled={disabled}
                aria-label="Temp HP"
                value={tempHpInput}
                onChange={(event) =>
                  setTempHpDraft({
                    participantKey: current.participant_key,
                    value: event.target.value,
                  })
                }
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    applyTempHpInput();
                  }
                }}
              />
            </Form.Item>
            <div className="encounter-hp-actions">
              <Button disabled={disabled} onClick={applyTempHpInput}>
                Set
              </Button>
            </div>
          </div>
        </div>
      </div>
      <Collapse
        key={current.participant_key}
        ghost
        size="small"
        items={[
          {
            key: "hp-details",
            label: "HP settings",
            styles: { header: { paddingInline: 0 }, body: { paddingInline: 0 } },
            children: (
              <div className="encounter-hp-details">
                <p>
                  {hpOriginLabels[current.hp_origin] ?? "HP origin unavailable"};{" "}
                  {adjustmentOriginLabels[current.variant_origin] ??
                    "Adjustment origin unavailable"}
                </p>
                <CapacityControl
                  disabled={disabled}
                  current={current}
                  onAction={onAction}
                />
              </div>
            ),
          },
        ]}
      />
    </div>
  );
}

function hpLabel(participant: EncounterParticipantView): string {
  const current = displayNumber(participant.current_hp);
  const max = displayNumber(participant.max_hp);
  const temporaryHp = asNumber(participant.temporary_hp);
  return `${current}/${max}${temporaryHp > 0 ? ` +${temporaryHp}` : ""}`;
}

function CapacityControl({
  current,
  disabled = false,
  onAction,
}: {
  current: EncounterParticipantView;
  disabled?: boolean;
  onAction: (action: EncounterHpAction) => Promise<boolean>;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  const value = draft ?? current.max_hp?.toString() ?? "";
  return (
    <Form.Item style={{ marginBottom: 0 }} label="Maximum HP" layout="vertical">
      <Input
        disabled={disabled}
        aria-label="Maximum HP"
        value={value}
        onChange={(e) => setDraft(e.target.value)}
      />
      <div className="encounter-hp-actions">
        <Button
          disabled={disabled}
          onClick={() => {
            if (!disabled)
              void onAction({ kind: "set_maximum", input: value.trim() ? value : null })
                .then((saved) => {
                  if (saved) setDraft(null);
                })
                .catch(() => {});
          }}
        >
          Override maximum
        </Button>
        <Button
          disabled={disabled}
          onClick={() => {
            if (!disabled)
              void onAction({ kind: "use_derived_maximum" })
                .then((saved) => {
                  if (saved) setDraft(null);
                })
                .catch(() => {});
          }}
        >
          Use authored maximum
        </Button>
      </div>
    </Form.Item>
  );
}

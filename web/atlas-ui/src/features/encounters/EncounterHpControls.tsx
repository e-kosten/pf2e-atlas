import { Button, Form, Input, InputNumber } from "antd";
import { useState } from "react";
import type {
  EncounterParticipantView,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import {
  asNumber,
  clampCurrentHp,
  damageChanges,
  displayNumber,
  evaluateHpFormula,
  healChanges,
} from "./participantEdits";

export function EncounterHpControls({
  current,
  onUpdate,
}: {
  current: EncounterParticipantView;
  onUpdate: (changes: Partial<UpdateEncounterParticipantRequest>) => void;
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
    value: number | null;
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
    amountDraft?.participantKey === current.participant_key ? amountDraft.value : null;
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
    const hp = evaluateHpFormula(hpInput);
    if (hp !== null) {
      const clampedHp = clampCurrentHp(current, hp);
      onUpdate({
        current_hp: BigInt(clampedHp),
        defeated: clampedHp === 0 ? true : current.defeated,
      });
      setHpDraft({
        participantKey: current.participant_key,
        value: clampedHp.toString(),
      });
    }
  };
  const applyTempHpInput = () => {
    const temporaryHp = evaluateHpFormula(tempHpInput);
    if (temporaryHp !== null) {
      onUpdate({
        temporary_hp: BigInt(temporaryHp),
      });
      setTempHpDraft({
        participantKey: current.participant_key,
        value: temporaryHp.toString(),
      });
    }
  };
  const applyHpChangeInput = () => {
    if (amount === null || amount === 0) {
      return;
    }
    onUpdate(
      amount < 0
        ? damageChanges(current, Math.abs(amount))
        : healChanges(current, amount),
    );
    setAmountDraft({
      participantKey: current.participant_key,
      value: null,
    });
  };

  return (
    <div className="encounter-hp-panel">
      <div className="encounter-hp-panel__header">
        <h3>HP</h3>
        <span>{hpSummary}</span>
      </div>
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
      <div className="encounter-hp-grid">
        <div className="encounter-hp-control">
          <span className="encounter-hp-control__label">HP</span>
          <div className="encounter-hp-control__row">
            <Form.Item layout="vertical">
              <Input
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
              <Button onClick={applyHpInput}>Set</Button>
            </div>
          </div>
        </div>
        <div className="encounter-hp-control">
          <span className="encounter-hp-control__label">Temp HP</span>
          <div className="encounter-hp-control__row">
            <Form.Item layout="vertical">
              <Input
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
              <Button onClick={applyTempHpInput}>Set</Button>
            </div>
          </div>
        </div>
        <div className="encounter-hp-control encounter-hp-control--wide">
          <span className="encounter-hp-control__label">HP change</span>
          <div className="encounter-hp-control__row">
            <Form.Item layout="vertical">
              <InputNumber
                aria-label="HP change"
                value={amount}
                onChange={(value) =>
                  setAmountDraft({
                    participantKey: current.participant_key,
                    value,
                  })
                }
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    applyHpChangeInput();
                  }
                }}
              />
            </Form.Item>
            <div className="encounter-hp-actions">
              <Button
                onClick={() => {
                  if (amount !== null) {
                    onUpdate(damageChanges(current, Math.abs(amount)));
                    setAmountDraft({
                      participantKey: current.participant_key,
                      value: null,
                    });
                  }
                }}
              >
                Damage
              </Button>
              <Button
                onClick={() => {
                  if (amount !== null) {
                    onUpdate(healChanges(current, Math.abs(amount)));
                    setAmountDraft({
                      participantKey: current.participant_key,
                      value: null,
                    });
                  }
                }}
              >
                Heal
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function hpLabel(participant: EncounterParticipantView): string {
  const current = displayNumber(participant.current_hp);
  const max = displayNumber(participant.max_hp);
  const temporaryHp = asNumber(participant.temporary_hp);
  return `${current}/${max}${temporaryHp > 0 ? ` +${temporaryHp}` : ""}`;
}

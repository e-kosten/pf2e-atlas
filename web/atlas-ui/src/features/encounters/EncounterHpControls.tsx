import { EncounterHpValue } from "./EncounterHpValue";
import { NumberFact } from "../../shared/records/presentationFacts";
import { EncounterModal } from "./EncounterModal";
import { Button, Tooltip } from "antd";
import { Pencil } from "lucide-react";
import { useState, type ReactNode } from "react";
import type {
  EncounterParticipantView,
  EncounterHpAction,
} from "../../generated/atlas";
import { asNumber, displayNumber } from "./participantEdits";
import { EncounterHpAmount } from "./EncounterHpAmount";
import { EncounterHpEditor } from "./EncounterHpEditor";

export function EncounterHpControls({
  current,
  disabled = false,
  pending = false,
  error,
  recovery,
  onAction,
  statusControl,
}: {
  current: EncounterParticipantView;
  disabled?: boolean;
  pending?: boolean;
  error?: string;
  recovery?: ReactNode;
  onAction: (action: EncounterHpAction) => Promise<boolean>;
  statusControl?: ReactNode;
}) {
  const [editorOpen, setEditorOpen] = useState(false);
  const [committed, setCommitted] = useState(false);
  const hpKnown = current.current_hp !== undefined;
  const maxKnown = current.effective_max_hp.state === "value";
  const currentHp = asNumber(current.current_hp);
  const maxHp = current.effective_max_hp.value ?? 0;
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

  return (
    <div className="encounter-hp-panel">
      <div className="encounter-hp-panel__header">
        <div className="encounter-hp-panel__identity">
          <h3>HP</h3>
          <span className="encounter-hp-panel__values" aria-label="HP summary">
            <EncounterHpValue
              label="Current HP"
              value={current.current_hp}
              disabled={disabled}
              onCommit={(input) =>
                onAction({ kind: "set_current", input: input || null })
              }
            />
            <span>/</span>
            <span className="encounter-hp-maximum">
              <NumberFact fact={current.effective_max_hp} label="Maximum HP" />
            </span>
            {current.temporary_hp > 0n && (
              <>
                <span>+</span>
                <EncounterHpValue
                  label="Temporary HP"
                  value={current.temporary_hp}
                  disabled={disabled}
                  onCommit={(input) => onAction({ kind: "set_temporary", input })}
                />
                <span>temp</span>
              </>
            )}
          </span>
          {current.temporary_hp === 0n && (
            <EncounterHpValue
              label="Temporary HP"
              value={0n}
              addLabel="Add temp HP"
              disabled={disabled}
              onCommit={(input) => onAction({ kind: "set_temporary", input })}
            />
          )}
        </div>
        <div className="encounter-hp-panel__actions">
          {statusControl}
          <Tooltip title="Edit HP" trigger={["hover", "focus"]}>
            <Button
              size="small"
              type="text"
              aria-label="Edit HP"
              icon={<Pencil size={14} />}
              disabled={disabled}
              onClick={() => {
                setCommitted(false);
                setEditorOpen(true);
              }}
            />
          </Tooltip>
        </div>
      </div>
      {hpKnown && maxKnown && (
        <div className="encounter-hp-meter" aria-label={`HP remaining: ${hpSummary}`}>
          <span
            className={["encounter-hp-meter__current", hpMeterTone]
              .filter(Boolean)
              .join(" ")}
            style={{ width: `${hpPercent}%` }}
          />
          {temporaryHp > 0 && (
            <span
              className="encounter-hp-meter__temporary"
              style={{
                left: `${hpPercent}%`,
                width: `${tempHpPercent}%`,
              }}
            />
          )}
          {missingHp > 0 && (
            <span
              className="encounter-hp-meter__missing"
              style={{
                left: `${hpPercent + tempHpPercent}%`,
                width: `${missingHpPercent}%`,
              }}
            />
          )}
        </div>
      )}
      <EncounterHpAmount
        healUnavailableReason={
          current.max_hp !== undefined && !maxKnown
            ? (current.effective_max_hp.adjustment?.notes[0]?.reason ??
              "Maximum HP is unavailable.")
            : undefined
        }
        key={current.participant_key}
        disabled={disabled || !hpKnown}
        pending={pending && hpKnown}
        onAction={async (action) => {
          // A resolved write is committed even when its follow-up read failed.
          // Clear the amount so read recovery cannot repeat saved damage/healing.
          await onAction(action);
          return true;
        }}
      />
      <EncounterModal
        title="Edit HP"
        open={editorOpen && !(committed && !disabled)}
        destroyOnHidden
        footer={null}
        onCancel={() => setEditorOpen(false)}
        closable={!pending}
        keyboard={!pending}
        mask={{ closable: !pending }}
      >
        <EncounterHpEditor
          current={current}
          disabled={disabled}
          pending={pending}
          error={error}
          recovery={recovery}
          onAction={onAction}
          onCancel={() => setEditorOpen(false)}
          onCommitted={(refreshed) => {
            if (refreshed) setEditorOpen(false);
            else setCommitted(true);
          }}
        />
      </EncounterModal>
    </div>
  );
}

function hpLabel(participant: EncounterParticipantView): string {
  const current = displayNumber(participant.current_hp);
  const max = participant.effective_max_hp.value?.toString() ?? "—";
  const temporaryHp = asNumber(participant.temporary_hp);
  return `${current}/${max}${temporaryHp > 0 ? ` +${temporaryHp}` : ""}`;
}

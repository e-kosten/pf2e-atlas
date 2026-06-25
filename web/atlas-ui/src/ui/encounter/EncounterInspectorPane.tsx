import { Button, Select } from "antd";
import { ExternalLink, X } from "lucide-react";
import type React from "react";
import type { getRecordDetail } from "../../api/atlasApi";
import type {
  AddEncounterParticipantConditionRequest,
  ActivityRollView,
  DamageExpressionView,
  EncounterParticipantVariantView,
  EncounterParticipantView,
  MechanicActivityView,
  StatBlockView,
  StatValueView,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import {
  displayNumber,
  participantUpdate,
} from "../../features/encounters/participantEdits";
import { RecordPresentation } from "../recordPresentation";
import { EncounterParticipantControls } from "./EncounterParticipantControls";

export function EncounterInspectorPane({
  detailLoading,
  onCloseReferencePreview,
  onOpenReferenceFullPage,
  onReference,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  onUpdate,
  participant,
  participants,
  previewDetail,
  previewLoading,
  previewRecordKey,
  previewAnchor,
  recordDetail,
}: {
  detailLoading: boolean;
  onCloseReferencePreview: () => void;
  onOpenReferenceFullPage: (recordKey: string) => void;
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
  participant: EncounterParticipantView | undefined;
  participants: EncounterParticipantView[];
  previewDetail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
  previewLoading: boolean;
  previewRecordKey: string | null;
  previewAnchor: ReferenceAnchor | null;
  recordDetail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
}) {
  if (!participant) {
    return (
      <section className="encounter-pane detail-empty">Select a participant.</section>
    );
  }
  return (
    <section className="encounter-pane encounter-record-pane">
      <ParticipantHeader participant={participant} onUpdate={onUpdate} />
      <EncounterParticipantControls
        current={participant}
        onAddCondition={onAddCondition}
        onRemoveCondition={onRemoveCondition}
        onUpdate={onUpdate}
        onUpdateCondition={onUpdateCondition}
        participants={participants}
      />
      {participant.stat_block && <AdjustedStats statBlock={participant.stat_block} />}
      {participant.record_key && participant.status === "active" && (
        <section className="encounter-source-record">
          <h3>Source Record</h3>
          <RecordPresentation
            detail={recordDetail}
            loading={detailLoading}
            onReference={onReference}
          />
        </section>
      )}
      {previewRecordKey && (
        <ReferencePreview
          anchor={previewAnchor}
          detail={previewDetail}
          loading={previewLoading}
          onClose={onCloseReferencePreview}
          onOpenFullPage={() => onOpenReferenceFullPage(previewRecordKey)}
          onReference={onReference}
        />
      )}
    </section>
  );
}

function ReferencePreview({
  anchor,
  detail,
  loading,
  onClose,
  onOpenFullPage,
  onReference,
}: {
  anchor: ReferenceAnchor | null;
  detail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
  loading: boolean;
  onClose: () => void;
  onOpenFullPage: () => void;
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
}) {
  const position = referencePreviewPosition(anchor);
  return (
    <div
      aria-label="Reference preview overlay"
      className="encounter-reference-preview__backdrop"
      onClick={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
      role="presentation"
    >
      <div
        aria-label="Reference preview"
        className="encounter-reference-preview"
        role="dialog"
        style={position}
      >
        <header className="encounter-reference-preview__header">
          <span>Reference</span>
          <div className="encounter-actions">
            <Button
              aria-label="Open reference full page"
              icon={<ExternalLink size={14} />}
              onClick={onOpenFullPage}
              size="small"
            />
            <Button
              aria-label="Close reference preview"
              icon={<X size={14} />}
              onClick={onClose}
              size="small"
            />
          </div>
        </header>
        <div className="encounter-reference-preview__body">
          <RecordPresentation
            detail={detail}
            loading={loading}
            onReference={onReference}
          />
        </div>
      </div>
    </div>
  );
}

export type ReferenceAnchor = {
  top: number;
  right: number;
  bottom: number;
  left: number;
  width: number;
  height: number;
};

function referencePreviewPosition(anchor: ReferenceAnchor | null): React.CSSProperties {
  const margin = 16;
  const gap = 8;
  const width = Math.min(560, Math.max(360, window.innerWidth - margin * 2));
  const maxHeight = Math.min(560, window.innerHeight - margin * 2);
  if (!anchor) {
    return {
      maxHeight,
      right: margin,
      top: margin,
      width,
    };
  }
  const fitsRight = anchor.right + gap + width <= window.innerWidth - margin;
  const fitsLeft = anchor.left - gap - width >= margin;
  const left = fitsRight
    ? anchor.right + gap
    : fitsLeft
      ? anchor.left - gap - width
      : clamp(anchor.left, margin, window.innerWidth - margin - width);
  return {
    left,
    maxHeight,
    top: clamp(anchor.top, margin, window.innerHeight - margin - maxHeight),
    width,
  };
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), Math.max(min, max));
}

function ParticipantHeader({
  onUpdate,
  participant,
}: {
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
  participant: EncounterParticipantView;
}) {
  return (
    <header className="encounter-pane__header">
      <div>
        <p className="eyebrow">{participantKindLabel(participant.participant_kind)}</p>
        <h2>{participant.display_name}</h2>
      </div>
      {participant.participant_kind === "creature" && (
        <div className="encounter-variant-control">
          <span>Variant</span>
          <Select
            aria-label="Variant"
            className="encounter-variant-select"
            onChange={(value: EncounterParticipantVariantView) =>
              onUpdate(
                participantUpdate(participant, {
                  participant_variant: value,
                }),
              )
            }
            options={[
              { label: "Normal", value: "normal" },
              { label: "Elite", value: "elite" },
              { label: "Weak", value: "weak" },
            ]}
            size="small"
            value={participant.participant_variant}
          />
        </div>
      )}
    </header>
  );
}

function participantKindLabel(
  kind: EncounterParticipantView["participant_kind"],
): string {
  if (kind === "pc") {
    return "PC";
  }
  return kind.charAt(0).toUpperCase() + kind.slice(1);
}

function AdjustedStats({ statBlock }: { statBlock: StatBlockView }) {
  return (
    <section className="encounter-adjusted-stats">
      <div className="encounter-adjusted-stats__title">
        <h3>Adjusted Stats</h3>
        {statBlock.adjusted_level !== undefined &&
          statBlock.adjusted_level !== statBlock.level && (
            <span>
              Level {displayNumber(statBlock.level)} to{" "}
              {displayNumber(statBlock.adjusted_level)}
            </span>
          )}
      </div>
      <div className="encounter-stat-grid">
        {statBlock.values.map((value) => (
          <StatValue key={value.target} value={value} />
        ))}
      </div>
      {statBlock.activities.length > 0 && (
        <div className="encounter-activity-list">
          <h4>Activities</h4>
          {statBlock.activities.map((activity) => (
            <ActivityValue key={activity.activity_id} activity={activity} />
          ))}
        </div>
      )}
      {statBlock.unapplied_effects.length > 0 && (
        <div className="encounter-stat-notes">
          {statBlock.unapplied_effects.map((effect) => (
            <p key={`${effect.source}:${effect.label}`}>
              <strong>{effect.label}</strong>: {effect.reason}
            </p>
          ))}
        </div>
      )}
    </section>
  );
}

function ActivityValue({ activity }: { activity: MechanicActivityView }) {
  return (
    <div className="encounter-activity-row">
      <div className="encounter-activity-row__title">
        <strong>{activity.label}</strong>
        <span>{activity.kind.replace(/_/g, " ")}</span>
      </div>
      {activity.rolls.map((roll) => (
        <ActivityRollValue key={roll.roll_id} roll={roll} />
      ))}
      {activity.damage.map((damage) => (
        <DamageValue key={damage.damage_id} damage={damage} />
      ))}
      {activity.modes.map((mode) => (
        <div key={mode.mode_id} className="encounter-activity-mode">
          <div className="encounter-activity-mode__title">
            <strong>{mode.label}</strong>
            {modeMetadata(mode).length > 0 && (
              <small>{modeMetadata(mode).join(" / ")}</small>
            )}
          </div>
          {mode.damage.map((damage) => (
            <DamageValue key={damage.damage_id} damage={damage} />
          ))}
        </div>
      ))}
    </div>
  );
}

function modeMetadata(mode: MechanicActivityView["modes"][number]) {
  return [mode.target, mode.range, mode.time].filter(Boolean);
}

function ActivityRollValue({ roll }: { roll: ActivityRollView }) {
  const changed = roll.adjusted_value !== roll.base_value;
  const decreased = roll.adjusted_value < roll.base_value;
  return (
    <div className="encounter-damage-row">
      <span>
        {roll.label}:{" "}
        <strong className={decreased ? "encounter-stat-value--decreased" : undefined}>
          {signed(roll.adjusted_value)}
        </strong>
      </span>
      {changed && <small>base {signed(roll.base_value)}</small>}
      {roll.modifiers.length > 0 && (
        <small>
          {roll.modifiers
            .map((modifier) => `${modifier.label} ${signed(modifier.value)}`)
            .join(", ")}
        </small>
      )}
    </div>
  );
}

function DamageValue({ damage }: { damage: DamageExpressionView }) {
  const formula = damage.adjusted_formula ?? damage.formula;
  const changed =
    damage.adjusted_formula !== undefined && damage.adjusted_formula !== damage.formula;
  return (
    <div className="encounter-damage-row">
      <span>
        {formula}
        {damage.damage_type ? ` ${damage.damage_type}` : ""}
      </span>
      {changed && <small>base {damage.formula}</small>}
      {damage.effect_kind === "damage_or_healing" && <small>choose mode</small>}
      {damage.modifiers.length > 0 && (
        <small>
          {damage.modifiers
            .map((modifier) => `${modifier.label} ${signed(modifier.value)}`)
            .join(", ")}
        </small>
      )}
    </div>
  );
}

function StatValue({ value }: { value: StatValueView }) {
  const changed = value.adjusted_value !== value.base_value;
  const decreased = value.adjusted_value < value.base_value;
  return (
    <div className="encounter-stat-row">
      <span>{value.label}</span>
      <strong className={decreased ? "encounter-stat-value--decreased" : undefined}>
        {signed(value.adjusted_value)}
      </strong>
      {changed && <small>base {signed(value.base_value)}</small>}
      {value.modifiers.length > 0 && (
        <small>
          {value.modifiers
            .map((modifier) => `${modifier.label} ${signed(modifier.value)}`)
            .join(", ")}
        </small>
      )}
      {value.suppressed_modifiers.length > 0 && (
        <small>
          Suppressed:{" "}
          {value.suppressed_modifiers
            .map((modifier) => `${modifier.label} ${signed(modifier.value)}`)
            .join(", ")}
        </small>
      )}
    </div>
  );
}

function signed(value: bigint | number | undefined): string {
  if (value === undefined) {
    return "--";
  }
  return Number(value).toString();
}

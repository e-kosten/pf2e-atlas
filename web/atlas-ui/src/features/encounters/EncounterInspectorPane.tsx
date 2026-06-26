import { Select } from "antd";
import type { getRecordDetail } from "../../api/atlasApi";
import type {
  AddEncounterParticipantConditionRequest,
  ActivityRollView,
  DamageExpressionView,
  EncounterParticipantVariantView,
  EncounterParticipantView,
  EncounterConditionDefinitionView,
  MechanicActivityView,
  MovementSpeedView,
  RuntimeCountView,
  StatBlockView,
  StatValueView,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { displayNumber, participantUpdate } from "./participantEdits";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import {
  RecordPreviewPopover,
  type RecordPreviewAnchor,
} from "../../shared/records/RecordPreviewPopover";
import { EncounterParticipantControls } from "./EncounterParticipantControls";

export function EncounterInspectorPane({
  detailLoading,
  onCloseRecordPreview,
  onOpenRecordFullPage,
  onReference,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  onUpdate,
  participant,
  participants,
  conditionDefinitions,
  previewDetail,
  previewLoading,
  previewRecordKey,
  previewAnchor,
  recordDetail,
}: {
  detailLoading: boolean;
  onCloseRecordPreview: () => void;
  onOpenRecordFullPage: (recordKey: string) => void;
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
  conditionDefinitions: EncounterConditionDefinitionView[];
  previewDetail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
  previewLoading: boolean;
  previewRecordKey: string | null;
  previewAnchor: RecordPreviewAnchor | null;
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
        onReference={onReference}
        participants={participants}
        conditionDefinitions={conditionDefinitions}
      />
      {participant.stat_block && <AdjustedStats statBlock={participant.stat_block} />}
      {participant.record_key && participant.status === "active" && (
        <section className="encounter-source-record">
          <h3>Source Record</h3>
          <RecordDetailPane
            detail={recordDetail}
            loading={detailLoading}
            onReference={onReference}
          />
        </section>
      )}
      {previewRecordKey && (
        <RecordPreviewPopover
          anchor={previewAnchor}
          detail={previewDetail}
          loading={previewLoading}
          onClose={onCloseRecordPreview}
          onOpenFullPage={() => onOpenRecordFullPage(previewRecordKey)}
          onReference={onReference}
        />
      )}
    </section>
  );
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
      {(statBlock.action_budget || statBlock.speeds.length > 0) && (
        <RuntimeStats statBlock={statBlock} />
      )}
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

function RuntimeStats({ statBlock }: { statBlock: StatBlockView }) {
  return (
    <section className="encounter-runtime">
      <div className="encounter-adjusted-stats__title">
        <h3>Runtime</h3>
      </div>
      {statBlock.action_budget && (
        <div className="encounter-runtime-grid">
          <RuntimeCount count={statBlock.action_budget.actions} />
          <RuntimeCount count={statBlock.action_budget.reactions} />
          {!statBlock.action_budget.can_act.available && (
            <RuntimeCapability
              label="Can act"
              reason={statBlock.action_budget.can_act.reason}
            />
          )}
          {!statBlock.action_budget.can_react.available && (
            <RuntimeCapability
              label="Can react"
              reason={statBlock.action_budget.can_react.reason}
            />
          )}
        </div>
      )}
      {statBlock.speeds.length > 0 && (
        <div className="encounter-runtime-grid">
          {statBlock.speeds.map((speed) => (
            <MovementSpeed key={speed.movement_type} speed={speed} />
          ))}
        </div>
      )}
      {statBlock.action_budget && statBlock.action_budget.notes.length > 0 && (
        <RuntimeNotes notes={statBlock.action_budget.notes} />
      )}
    </section>
  );
}

function RuntimeCount({ count }: { count: RuntimeCountView }) {
  const changed = count.adjusted_value !== count.base_value;
  return (
    <div className="encounter-stat-row encounter-runtime-row">
      <span>{count.label}</span>
      <strong>{count.adjusted_value.toString()}</strong>
      {count.segments.some((segment) => segment.restricted) && (
        <small>
          {count.segments
            .filter((segment) => segment.restricted)
            .map((segment) => `${segment.label}: ${segment.value.toString()}`)
            .join(", ")}
        </small>
      )}
      {changed && <small>base {count.base_value.toString()}</small>}
      {count.adjustments.length > 0 && (
        <small>
          {count.adjustments
            .map((adjustment) => `${adjustment.source} ${signed(adjustment.value)}`)
            .join(", ")}
        </small>
      )}
      {count.suppressed_adjustments.length > 0 && (
        <small>
          Suppressed:{" "}
          {count.suppressed_adjustments
            .map((adjustment) => adjustment.source)
            .join(", ")}
        </small>
      )}
    </div>
  );
}

function RuntimeCapability({
  label,
  reason,
}: {
  label: string;
  reason: string | undefined;
}) {
  return (
    <div className="encounter-stat-row encounter-runtime-row">
      <span>{label}</span>
      <strong className="encounter-stat-value--decreased">No</strong>
      {reason && <small>{reason}</small>}
    </div>
  );
}

function MovementSpeed({ speed }: { speed: MovementSpeedView }) {
  const changed = speed.adjusted_value_feet !== speed.base_value_feet;
  const decreased = speed.adjusted_value_feet < speed.base_value_feet;
  return (
    <div className="encounter-stat-row encounter-runtime-row">
      <span>{speed.label}</span>
      <strong className={decreased ? "encounter-stat-value--decreased" : undefined}>
        {speed.adjusted_value_feet.toString()} ft
      </strong>
      {changed && <small>base {speed.base_value_feet.toString()} ft</small>}
      {speed.adjustments.length > 0 && (
        <small>
          {speed.adjustments
            .map((adjustment) => `${adjustment.source} ${signed(adjustment.value)}`)
            .join(", ")}
        </small>
      )}
      {speed.notes.map((note) => (
        <small key={`${note.source}:${note.label}`}>{note.reason}</small>
      ))}
    </div>
  );
}

function RuntimeNotes({
  notes,
}: {
  notes: NonNullable<StatBlockView["action_budget"]>["notes"];
}) {
  return (
    <div className="encounter-stat-notes">
      {notes.map((note) => (
        <p key={`${note.source}:${note.label}`}>
          <strong>{note.label}</strong>: {note.reason}
        </p>
      ))}
    </div>
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

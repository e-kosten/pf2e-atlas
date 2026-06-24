import { Button, Select } from "antd";
import { ExternalLink, X } from "lucide-react";
import type { getRecordDetail } from "../../api/atlasApi";
import type {
  EncounterParticipantVariantView,
  EncounterParticipantView,
  StatBlockView,
  StatValueView,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { RecordPresentation } from "../recordPresentation";

export function EncounterInspectorPane({
  detailLoading,
  onCloseReferencePreview,
  onOpenReferenceFullPage,
  onReference,
  onUpdate,
  participant,
  previewDetail,
  previewLoading,
  previewRecordKey,
  recordDetail,
}: {
  detailLoading: boolean;
  onCloseReferencePreview: () => void;
  onOpenReferenceFullPage: (recordKey: string) => void;
  onReference: (recordKey: string) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
  participant: EncounterParticipantView | undefined;
  previewDetail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
  previewLoading: boolean;
  previewRecordKey: string | null;
  recordDetail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
}) {
  if (!participant) {
    return (
      <section className="encounter-pane detail-empty">Select a participant.</section>
    );
  }
  if (participant.participant_kind === "pc") {
    return (
      <section className="encounter-pane manual-participant">
        <ParticipantHeader participant={participant} onUpdate={onUpdate} />
        <dl>
          <dt>Initiative</dt>
          <dd>{displayNumber(participant.initiative)}</dd>
          <dt>HP</dt>
          <dd>{hpLabel(participant)}</dd>
          <dt>Side</dt>
          <dd>{participant.side}</dd>
        </dl>
        {participant.note && <p>{participant.note}</p>}
      </section>
    );
  }
  return (
    <section className="encounter-pane encounter-record-pane">
      <ParticipantHeader participant={participant} onUpdate={onUpdate} />
      {participant.stat_block && <AdjustedStats statBlock={participant.stat_block} />}
      <RecordPresentation
        detail={recordDetail}
        loading={detailLoading}
        onReference={onReference}
      />
      {previewRecordKey && (
        <ReferencePreview
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
  detail,
  loading,
  onClose,
  onOpenFullPage,
  onReference,
}: {
  detail: Awaited<ReturnType<typeof getRecordDetail>> | undefined;
  loading: boolean;
  onClose: () => void;
  onOpenFullPage: () => void;
  onReference: (recordKey: string) => void;
}) {
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

function participantUpdate(
  participant: EncounterParticipantView,
  changes: Partial<UpdateEncounterParticipantRequest>,
): UpdateEncounterParticipantRequest {
  return {
    participant_key: participant.participant_key,
    display_name: participant.display_name,
    side: participant.side,
    participant_variant: participant.participant_variant,
    initiative: participant.initiative,
    max_hp: participant.max_hp,
    current_hp: participant.current_hp,
    temporary_hp: participant.temporary_hp,
    defeated: participant.defeated,
    hidden: participant.hidden,
    note: participant.note,
    ...changes,
  };
}

function signed(value: bigint | number | undefined): string {
  if (value === undefined) {
    return "--";
  }
  return Number(value).toString();
}

function hpLabel(participant: EncounterParticipantView): string {
  if (
    participant.current_hp === undefined &&
    participant.max_hp === undefined &&
    asNumber(participant.temporary_hp) === 0
  ) {
    return "";
  }
  const current = displayNumber(participant.current_hp);
  const max = displayNumber(participant.max_hp);
  const temporaryHp = asNumber(participant.temporary_hp);
  return `${current}/${max}${temporaryHp > 0 ? ` +${temporaryHp}` : ""}`;
}
function displayNumber(value: bigint | undefined): string {
  return value === undefined ? "--" : value.toString();
}
function asNumber(value: bigint | undefined): number {
  return value === undefined ? 0 : Number(value);
}

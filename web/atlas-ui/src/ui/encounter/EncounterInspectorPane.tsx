import type { getRecordDetail } from "../../api/atlasApi";
import type { EncounterParticipantView } from "../../generated/atlas";
import { RecordPresentation } from "../recordPresentation";

export function EncounterInspectorPane({
  detailLoading,
  participant,
  recordDetail,
}: {
  detailLoading: boolean;
  participant: EncounterParticipantView | undefined;
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
        <p className="eyebrow">PC</p>
        <h2>{participant.display_name}</h2>
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
    <section className="encounter-pane">
      <RecordPresentation
        detail={recordDetail}
        loading={detailLoading}
        onReference={() => undefined}
      />
    </section>
  );
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

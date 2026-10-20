import type {
  EncounterActionIssueView,
  EncounterParticipantView,
} from "../../generated/atlas";
type Context = Pick<
  EncounterActionIssueView,
  "controller_key" | "instructions" | "subject"
>;
export function actionIssueContextLabel(
  source: Context,
  participants: EncounterParticipantView[],
): string | null {
  const name =
    participants.find(
      (participant) => participant.participant_key === source.controller_key,
    )?.display_name ?? source.controller_key;
  const subjectValue = source.subject;
  const subject =
    subjectValue?.kind === "participant"
      ? (participants.find(
          (participant) => participant.participant_key === subjectValue.participant_key,
        )?.display_name ?? subjectValue.participant_key)
      : subjectValue?.description;
  return (
    [
      name
        ? `${name}${source.instructions ? `: ${source.instructions}` : ""}`
        : source.instructions,
      subject ? `Subject: ${subject}` : null,
    ]
      .filter(Boolean)
      .join(" · ") || null
  );
}
export function actionIssueQuestionLabel(
  reason: string,
  context: Context,
  participants: EncounterParticipantView[],
  sourceId: bigint | null,
): string {
  const label =
    actionIssueContextLabel(context, participants) ??
    (sourceId === null ? null : `Source ${sourceId}`);
  return label ? `${reason} · ${label}` : reason;
}
export function actionIssueSourceLabel(
  source: EncounterActionIssueView,
  participants: EncounterParticipantView[],
): string {
  return `${source.condition_name} · ${actionIssueContextLabel(source, participants) ?? `Source ${source.source_condition_id}`}`;
}

import type {
  EncounterActionIssueView,
  EncounterParticipantView,
} from "../../generated/atlas";
import { actionIssueContextLabel } from "./actionIssueLabels";
export function ActionIssueContext({
  source,
  participants,
}: {
  source: Pick<EncounterActionIssueView, "controller_key" | "instructions" | "subject">;
  participants: EncounterParticipantView[];
}) {
  const label = actionIssueContextLabel(source, participants);
  return label ? <span> · {label}</span> : null;
}

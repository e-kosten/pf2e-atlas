import type {
  EncounterParticipantView,
  EncounterTargetingRequirementsView,
} from "../../generated/atlas";
import { EncounterRelationships } from "./EncounterRelationships";
import type { useEncounterMutations } from "./useEncounterMutations";
export function TargetRelationshipReview({
  targeting,
  targetKey,
  observerKey,
  encounterRef,
  revision,
  participants,
  mutations,
  onEditCondition,
}: {
  targeting: EncounterTargetingRequirementsView | null | undefined;
  targetKey: string | undefined;
  observerKey: string | undefined;
  encounterRef: string;
  revision: bigint;
  participants: EncounterParticipantView[];
  mutations: ReturnType<typeof useEncounterMutations>;
  onEditCondition: import("./conditionEditRequest").EditConditionSource;
}) {
  if (!targeting?.required_inputs.includes("detection")) return null;
  const subject = participants.find(
    (participant) => participant.participant_key === targetKey,
  );
  return (
    <>
      {subject && observerKey && (
        <EncounterRelationships
          key={`${subject.participant_key}:${observerKey}`}
          encounterRef={encounterRef}
          revision={revision}
          subject={subject}
          onEditCondition={onEditCondition}
          participants={participants}
          initialOtherKey={observerKey}
          title="Check detection"
          properties={["detection", "awareness"]}
          disabled={mutations.pending || !!mutations.refreshError}
          onSave={async (request) => {
            const result = await mutations.run({ kind: "relationship", request });
            return result.active && result.refreshed;
          }}
          onEvent={async (request) => {
            const result = await mutations.run({ kind: "detection-event", request });
            return result.active && result.refreshed;
          }}
        />
      )}
    </>
  );
}

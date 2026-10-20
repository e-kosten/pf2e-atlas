import { Alert, Button, Collapse, Spin } from "antd";
import type {
  ApplyPersistentDamageRequest,
  EncounterParticipantView,
  PersistentDamageResultView,
  PersistentRecoveryResultView,
  RecoverPersistentDamageRequest,
} from "../../generated/atlas";
import { EncounterPersistentDamageBatch } from "./EncounterPersistentDamageBatch";
import { EncounterPersistentRecovery } from "./EncounterPersistentRecovery";
import { conditionDurationLabel } from "./conditionDurationLabel";
import { usePersistentReview } from "./usePersistentReview";
export function EncounterPersistentTurnReview({
  encounterRef,
  participant,
  encounterRevision,
  boundaryKey = null,
  recoveredEpisodeIds = [],
  generation,
  damageTypes,
  disabled,
  pending,
  onApply,
  onRecover,
}: {
  encounterRef: string;
  participant: EncounterParticipantView;
  encounterRevision: bigint;
  boundaryKey?: string | null;
  recoveredEpisodeIds?: string[];
  generation: string | null;
  damageTypes: Array<{ value: string; label: string }>;
  disabled: boolean;
  pending: boolean;
  onApply: (
    request: Omit<ApplyPersistentDamageRequest, "request_id">,
  ) => Promise<PersistentDamageResultView>;
  onRecover: (
    request: Omit<RecoverPersistentDamageRequest, "request_id">,
  ) => Promise<PersistentRecoveryResultView>;
}) {
  const query = usePersistentReview(
    encounterRef,
    participant.participant_key,
    encounterRevision,
    boundaryKey,
  );
  const review = query.data;
  const blocked = disabled || query.isFetching || !!query.error;
  return (
    <section className="encounter-turn-reminder">
      <h3>Persistent damage</h3>
      {query.error && (
        <Alert
          type="error"
          title="Could not load persistent damage"
          action={<Button onClick={() => void query.refetch()}>Retry</Button>}
        />
      )}
      {!review && query.isPending && <Spin />}
      {review && (
        <>
          <EncounterPersistentDamageBatch
            key={`${participant.participant_key}:${review.round_number}`}
            participant={participant}
            boundaryKey={boundaryKey}
            encounterRevision={encounterRevision}
            generation={generation}
            damageApplied={review.damage_applied}
            savedResult={review.damage_result}
            damageTypes={damageTypes}
            disabled={blocked}
            pending={pending}
            onApply={onApply}
          />
          {participant.persistent_damage_groups.map((group) => {
            const type =
              damageTypes.find((type) => type.value === group.damage_type)?.label ??
              group.damage_type;
            const selectedSource = group.sources.find(
              (source) => source.condition_id === group.selected_source_id,
            );
            const episode = review.episodes.find(
              (episode) => episode.episode_id === group.episode_id,
            );
            return (
              <div key={group.episode_id}>
                <p>
                  {participant.persistent_damage_groups.length === 1
                    ? "Recovery"
                    : type}
                  {selectedSource?.recovery_dc == null
                    ? ""
                    : ` · Flat check DC ${selectedSource.recovery_dc}`}
                </p>
                {group.sources.length > 1 && (
                  <Collapse
                    ghost
                    items={[
                      {
                        key: "sources",
                        label: "Additional sources",
                        children: group.sources
                          .filter(
                            (source) =>
                              source.condition_id !== group.selected_source_id,
                          )
                          .map((source) => {
                            const condition = participant.conditions.find(
                              (condition) =>
                                condition.condition_id === source.condition_id,
                            );
                            return (
                              <p key={source.condition_id.toString()}>
                                {source.formula}
                                {source.multiplier !== "normal"
                                  ? ` · ${source.multiplier}`
                                  : ""}
                                {condition?.duration
                                  ? ` · ${conditionDurationLabel(condition.duration)}`
                                  : ""}
                                {condition?.note ? ` · ${condition.note}` : ""}
                              </p>
                            );
                          }),
                      },
                    ]}
                  />
                )}
                {(!recoveredEpisodeIds.includes(group.episode_id) ||
                  review.recovery_results.some(
                    (result) => result.episode_id === group.episode_id,
                  )) && (
                  <EncounterPersistentRecovery
                    group={group}
                    recordedResult={review.recovery_results.find(
                      (result) => result.episode_id === group.episode_id,
                    )}
                    size="middle"
                    boundaryKey={boundaryKey}
                    mode="end_turn"
                    available={
                      review.episodes.find(
                        (episode) => episode.episode_id === group.episode_id,
                      )?.end_turn_recovery_available ?? false
                    }
                    unavailableReason={episode?.end_turn_recovery_unavailable_reason}
                    rollAvailable={episode?.end_turn_roll_available ?? false}
                    rollUnavailableReason={episode?.end_turn_roll_unavailable_reason}
                    participantKey={participant.participant_key}
                    encounterRevision={encounterRevision}
                    disabled={blocked}
                    pending={pending}
                    damageType={type}
                    onRecover={onRecover}
                    onResult={() => {
                      void query.refetch();
                    }}
                  />
                )}
              </div>
            );
          })}
        </>
      )}
    </section>
  );
}

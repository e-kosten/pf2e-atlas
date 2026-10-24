import { Alert, Button, Spin } from "antd";
import type {
  ApplyPersistentDamageRequest,
  EncounterParticipantView,
  PersistentDamageResultView,
  PersistentRecoveryResultView,
  RecoverPersistentDamageRequest,
} from "../../generated/atlas";
import { EncounterPersistentDamageBatch } from "./EncounterPersistentDamageBatch";
import { conditionDurationLabel } from "./conditionDurationLabel";
import { usePersistentReview } from "./usePersistentReview";
export function EncounterPersistentTurnReview({
  encounterRef,
  participant,
  participants = [],
  encounterRevision,
  boundaryKey = null,
  generation,
  damageTypes,
  disabled,
  pending,
  onApply,
  onRecover,
}: {
  encounterRef: string;
  participant: EncounterParticipantView;
  participants?: EncounterParticipantView[];
  encounterRevision: bigint;
  boundaryKey?: string | null;
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
            encounterRef={encounterRef}
            onRecover={onRecover}
            preparedDamage={review.prepared_damage}
            preparedRecovery={review.prepared_recovery}
            recoveryResults={review.recovery_results}
            episodes={review.episodes}
            participant={{
              ...participant,
              persistent_damage_groups: review.damage_groups,
            }}
            boundaryKey={boundaryKey}
            encounterRevision={review.encounter_revision}
            generation={generation}
            damageApplied={review.damage_applied}
            savedResult={review.damage_result}
            resolutions={Object.fromEntries(
              review.episodes
                .filter((episode) => episode.damage_resolution)
                .map((episode) => [episode.episode_id, episode.damage_resolution]),
            )}
            damageTypes={damageTypes}
            disabled={blocked}
            pending={pending}
            onApply={onApply}
            displayGroups={participant.persistent_damage_groups}
            renderGroup={(group, damage, recovery) => {
              const type =
                damageTypes.find((type) => type.value === group.damage_type)?.label ??
                group.damage_type;
              const ended = !review.damage_groups.some(
                (active) => active.episode_id === group.episode_id,
              );
              const selected = group.sources.find(
                (source) => source.condition_id === group.selected_source_id,
              );
              return (
                <section
                  className="encounter-persistent-type"
                  aria-label={`${type} persistent damage`}
                >
                  <div role="group" aria-label={`${type} sources`}>
                    <div className="encounter-persistent-heading">
                      <h4>{type}</h4>
                      {selected && (
                        <strong>
                          {selected.formula}
                          {selected.multiplier === "double"
                            ? " ×2"
                            : selected.multiplier === "half"
                              ? " ×½"
                              : ""}
                        </strong>
                      )}
                      {ended && <span className="muted">Ended</span>}
                    </div>
                    <div className="encounter-persistent-sources">
                      {group.sources.map((source) => {
                        const condition = participant.conditions.find(
                          (c) => c.condition_id === source.condition_id,
                        );
                        const from = participants.find(
                          (p) =>
                            p.participant_key === condition?.source_participant_key,
                        )?.display_name;
                        const superseded =
                          source.condition_id !== group.selected_source_id;
                        if (
                          !superseded &&
                          !from &&
                          !condition?.duration &&
                          !condition?.note
                        )
                          return null;
                        return (
                          <div
                            key={source.condition_id.toString()}
                            className={
                              superseded
                                ? "encounter-persistent-source--superseded"
                                : undefined
                            }
                          >
                            {superseded && (
                              <span>
                                {source.formula}
                                {source.multiplier === "normal"
                                  ? ""
                                  : source.multiplier === "double"
                                    ? " ×2"
                                    : " ×½"}
                                {superseded && (
                                  <span className="encounter-condition-meta">
                                    {" "}
                                    · Superseded
                                  </span>
                                )}
                              </span>
                            )}
                            {(from || condition?.duration || condition?.note) && (
                              <div className="encounter-condition-meta">
                                {[
                                  from && `From ${from}`,
                                  condition?.duration &&
                                    conditionDurationLabel(condition.duration),
                                  condition?.note,
                                ]
                                  .filter(Boolean)
                                  .join(" · ")}
                              </div>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  </div>
                  {damage}
                  {recovery}
                </section>
              );
            }}
          />
        </>
      )}
    </section>
  );
}

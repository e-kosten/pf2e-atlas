import { Alert, Button, Space, Spin, Tooltip } from "antd";
import { X } from "lucide-react";
import { useState } from "react";
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
  const [clearError, setClearError] = useState<string>();
  const review = query.data;
  const blocked = disabled || query.isFetching || !!query.error;
  return (
    <section className="encounter-turn-reminder">
      <h3>Persistent damage</h3>
      {clearError && <Alert type="error" title={clearError} />}
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
            renderGroup={(group, damage) => {
              const type =
                damageTypes.find((type) => type.value === group.damage_type)?.label ??
                group.damage_type;
              const ended = !review.damage_groups.some(
                (active) => active.episode_id === group.episode_id,
              );
              const episode = review.episodes.find(
                (episode) => episode.episode_id === group.episode_id,
              );
              return (
                <section
                  className="encounter-persistent-type"
                  aria-label={`${type} persistent damage`}
                >
                  <div className="encounter-persistent-heading">
                    <h4>{type}</h4>
                    {ended && <span className="muted">Ended</span>}
                  </div>
                  <div
                    className="encounter-persistent-sources"
                    role="group"
                    aria-label={`${type} sources`}
                  >
                    {[...group.sources]
                      .sort(
                        (a, b) =>
                          Number(b.condition_id === group.selected_source_id) -
                          Number(a.condition_id === group.selected_source_id),
                      )
                      .map((source) => {
                        const condition = participant.conditions.find(
                          (c) => c.condition_id === source.condition_id,
                        );
                        const from = participants.find(
                          (p) =>
                            p.participant_key === condition?.source_participant_key,
                        )?.display_name;
                        const superseded =
                          group.selected_source_id !== null &&
                          source.condition_id !== group.selected_source_id;
                        return (
                          <div
                            key={source.condition_id.toString()}
                            className={
                              superseded
                                ? "encounter-persistent-source--superseded"
                                : undefined
                            }
                          >
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
                  {damage}
                  <div
                    className="encounter-persistent-recovery-row"
                    role="group"
                    aria-label={`${type} recovery`}
                  >
                    <span>Recovery</span>
                    <Space size="small" align="center">
                      <EncounterPersistentRecovery
                        group={group}
                        recordedResult={review.recovery_results.find(
                          (result) => result.episode_id === group.episode_id,
                        )}
                        size="small"
                        appearance="icon"
                        boundaryKey={boundaryKey}
                        mode="end_turn"
                        available={
                          (!ended &&
                            review.episodes.find(
                              (episode) => episode.episode_id === group.episode_id,
                            )?.end_turn_recovery_available) ??
                          false
                        }
                        rollAvailable={
                          !ended && (episode?.end_turn_roll_available ?? false)
                        }
                        rollUnavailableReason={
                          episode?.end_turn_roll_unavailable_reason
                        }
                        participantKey={participant.participant_key}
                        encounterRevision={review.encounter_revision}
                        disabled={blocked}
                        pending={pending}
                        damageType={type}
                        onRecover={onRecover}
                        onResult={() => {
                          void query.refetch();
                        }}
                      />

                      <Tooltip
                        title={`End ${type.toLowerCase()} damage`}
                        trigger={["hover", "focus"]}
                      >
                        <Button
                          size="small"
                          type="text"
                          aria-label={`End ${type.toLowerCase()} damage`}
                          icon={<X size={14} />}
                          disabled={blocked || pending || ended}
                          onClick={async () => {
                            setClearError(undefined);
                            try {
                              await onRecover({
                                resolution: "clear",
                                boundary_key: boundaryKey,
                                mode: "end_turn",
                                participant_key: participant.participant_key,
                                episode_id: group.episode_id,
                                expected_encounter_revision: review.encounter_revision,
                                selected_source_id: null,
                                dc_override: null,
                              });
                              void query.refetch();
                            } catch (cause) {
                              setClearError(
                                cause instanceof Error
                                  ? cause.message
                                  : "Could not end persistent damage.",
                              );
                            }
                          }}
                        />
                      </Tooltip>
                    </Space>
                  </div>
                </section>
              );
            }}
          />
        </>
      )}
    </section>
  );
}

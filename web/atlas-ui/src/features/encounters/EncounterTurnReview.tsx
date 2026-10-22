import { EncounterModal } from "./EncounterModal";
import { RollResult } from "../../shared/dice/RollResult";
import { conditionDurationLabel } from "./conditionDurationLabel";
import { Alert, Button } from "antd";
import type {
  EncounterParticipantView,
  EncounterTurnReminderView,
  EncounterTurnReviewView,
} from "../../generated/atlas";
import {
  EncounterConditionRecovery,
  EncounterTurnRecoveryActions,
} from "./EncounterConditionLifecycle";
import { EncounterPersistentTurnReview } from "./EncounterPersistentTurnReview";
import { DeleteOutlined } from "@ant-design/icons";
import { type ReactNode } from "react";

export function EncounterTurnReview({
  open,
  boundaryKey,
  data,
  participants,
  pending,
  disabled,
  loading,
  error,
  actionError,
  recovery,
  receipt,
  outcomes = [],
  onRetry,
  onCancel,
  encounterRef,
  generation,
  damageTypes,
  onApplyPersistent,
  onRecoverPersistent,
  onLifecycle,
  onRemove,
}: {
  open: boolean;
  boundaryKey?: string;
  data?: EncounterTurnReviewView;
  participants: EncounterParticipantView[];
  pending: boolean;
  disabled: boolean;
  loading: boolean;
  error: Error | null;
  actionError?: string;
  recovery?: ReactNode;
  receipt: string;
  outcomes?: import("../../generated/atlas").RollOutcome[];
  onRetry: () => void;
  onCancel: () => void;
  encounterRef: string;
  generation: string | null;
  damageTypes: Array<{ value: string; label: string }>;
  onApplyPersistent: (
    request: Omit<
      import("../../generated/atlas").ApplyPersistentDamageRequest,
      "request_id"
    >,
  ) => Promise<import("../../generated/atlas").PersistentDamageResultView>;
  onLifecycle: React.ComponentProps<typeof EncounterConditionRecovery>["onApply"];
  onRecoverPersistent: (
    request: Omit<
      import("../../generated/atlas").RecoverPersistentDamageRequest,
      "request_id"
    >,
  ) => Promise<import("../../generated/atlas").PersistentRecoveryResultView>;
  onRemove: (
    reminder: Extract<
      EncounterTurnReminderView,
      { kind: "duration_ending" | "duration_ended" | "persistent_damage" }
    >,
    boundaryKey: string,
  ) => Promise<unknown>;
}) {
  const candidate = data?.outgoing;
  const outgoing =
    candidate && candidate.boundary_key === boundaryKey ? candidate : undefined;
  const reminders = outgoing?.reminders ?? [];
  const meaningful =
    reminders.length > 0 ||
    outcomes.length > 0 ||
    !!error ||
    !!actionError ||
    !!recovery;

  return (
    <EncounterModal
      title="Turn reminders"
      open={open && meaningful}
      footer={null}
      onCancel={onCancel}
      closable={!pending}
      keyboard={!pending}
      mask={{ closable: !pending }}
      transitionName=""
      maskTransitionName=""
      destroyOnHidden
    >
      {receipt && <Alert type="success" title={receipt} />}
      {recovery}
      {actionError && (
        <Alert type="error" title="Turn action failed" description={actionError} />
      )}
      {error && (
        <Alert
          type="error"
          title="Turn review could not load"
          description={error.message}
          action={
            <Button loading={loading} disabled={pending} onClick={onRetry}>
              Retry turn review
            </Button>
          }
        />
      )}
      {outgoing && (
        <p>
          {outgoing.participant_name} · Round {outgoing.round_number.toString()}
        </p>
      )}
      {outgoing?.participant_removed && (
        <>
          <p>Participant removed</p>
          {outgoing.persistent_damage_groups.map((group) => (
            <p key={group.episode_id}>
              Persistent Damage ·{" "}
              {group.sources.find(
                (source) => source.condition_id === group.selected_source_id,
              )?.formula ??
                group.sources.map((source) => source.formula).join(" / ")}{" "}
              {group.damage_type}
              {outgoing.recovered_episode_ids.includes(group.episode_id)
                ? " · Recovered"
                : ""}
            </p>
          ))}
          {outgoing.damage_applied && <p>Persistent damage applied</p>}
        </>
      )}
      {outgoing &&
        !outgoing.participant_removed &&
        reminders.some((reminder) => reminder.kind === "condition_recovery") &&
        participants.find(
          (participant) => participant.participant_key === outgoing.participant_key,
        ) && (
          <EncounterConditionRecovery
            key={outgoing.boundary_key}
            encounterRef={encounterRef}
            boundaryKey={outgoing.boundary_key}
            revision={data!.expected_revision}
            participant={
              participants.find(
                (participant) =>
                  participant.participant_key === outgoing.participant_key,
              )!
            }
            disabled={disabled || pending}
            onApply={onLifecycle}
          >
            <EncounterTurnRecoveryActions />
          </EncounterConditionRecovery>
        )}
      <div className="encounter-turn-review">
        {reminders.some((reminder) => reminder.kind === "persistent_damage") && (
          <PersistentParticipants
            data={data}
            outgoing={outgoing}
            participants={participants}
            encounterRef={encounterRef}
            generation={generation}
            damageTypes={damageTypes}
            disabled={disabled || loading || !!error}
            pending={pending}
            onApply={onApplyPersistent}
            onRecover={onRecoverPersistent}
          />
        )}
        {reminders
          .filter((reminder) => reminder.kind !== "condition_recovery")
          .filter(
            (reminder) =>
              reminder.kind !== "persistent_damage" ||
              !outgoing?.persistent_damage_groups.some((group) =>
                group.sources.some(
                  (source) => source.condition_id === reminder.condition_id,
                ),
              ),
          )
          .map((reminder) => {
            return (
              <section
                key={`${reminder.kind}:${reminder.participant_key}:${reminder.condition_id}`}
                className="encounter-turn-reminder"
              >
                <div className="encounter-turn-reminder__heading">
                  <h3>
                    {reminder.participant_name}: {reminder.condition_name}
                  </h3>
                  {(reminder.kind === "duration_ending" ||
                    reminder.kind === "duration_ended" ||
                    reminder.kind === "persistent_damage") && (
                    <Button
                      disabled={disabled || loading || !!error}
                      size="small"
                      type="text"
                      icon={<DeleteOutlined />}
                      aria-label={`Remove ${reminder.condition_name} from ${reminder.participant_name}`}
                      onClick={() => {
                        void onRemove(reminder, outgoing!.boundary_key).catch(() => {
                          // The mutation owner shows the failure and retains retry state.
                        });
                      }}
                    />
                  )}
                </div>
                <p>{reminderText(reminder, participants)}</p>
              </section>
            );
          })}
      </div>
      {!reminders.length &&
        outcomes.map((outcome, index) => <RollResult key={index} outcome={outcome} />)}
    </EncounterModal>
  );
}
function reminderText(
  reminder: EncounterTurnReminderView,
  participants: EncounterParticipantView[],
): string {
  switch (reminder.kind) {
    case "condition_recovery":
      return "";
    case "persistent_damage":
      return reminder.note ?? "Resolve at the table.";
    case "duration_ending": {
      const anchor = participants.find(
        (participant) =>
          participant.participant_key === reminder.timing.participant_key,
      );
      const turn = anchor ? `${anchor.display_name}’s turn` : "the configured turn";
      return `Expires at the ${reminder.timing.event} of ${turn}.`;
    }
    case "duration_ended":
      return "Duration ended";
    case "duration_reminder":
      return `${conditionDurationLabel(reminder.duration)}: check this duration at the table.`;
    case "timing_unavailable":
      return "Round timing is not configured. Edit the condition to choose whose turn and when its rounds count down.";
  }
}

function PersistentParticipants(
  props: Omit<
    React.ComponentProps<typeof EncounterPersistentTurnReview>,
    "participant" | "encounterRevision"
  > & {
    data?: EncounterTurnReviewView;
    outgoing?: NonNullable<EncounterTurnReviewView["outgoing"]>;
    participants: EncounterParticipantView[];
  },
) {
  const outgoing = props.outgoing;
  const participant = props.participants.find(
    (item) => item.participant_key === outgoing?.participant_key,
  );
  if (
    !outgoing ||
    !participant ||
    outgoing.participant_removed ||
    !props.data ||
    !outgoing.persistent_damage_groups.length
  )
    return null;
  return (
    <EncounterPersistentTurnReview
      key={outgoing.boundary_key}
      {...props}
      participant={{
        ...participant,
        persistent_damage_groups: outgoing.persistent_damage_groups,
      }}
      encounterRevision={props.data.expected_revision}
      boundaryKey={outgoing.boundary_key}
    />
  );
}

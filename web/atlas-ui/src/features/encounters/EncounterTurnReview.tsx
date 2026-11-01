import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
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
import { useLayoutEffect, useMemo, useRef, type ReactNode } from "react";

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
  onRemoveTracker,
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
  onRemoveTracker?: (
    reminder: Extract<EncounterTurnReminderView, { kind: "tracker_elapsed" }>,
    boundaryKey: string,
  ) => Promise<unknown>;
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
  const reminders = useMemo(() => outgoing?.reminders ?? [], [outgoing]);
  const reminderRows = useRef<HTMLDivElement>(null);
  const removalFocus = useRef<string | null>(null);
  useLayoutEffect(() => {
    const identity = removalFocus.current;
    if (
      !identity ||
      !reminders.some(
        (r) =>
          (r.kind === "resolved" &&
            `${r.participant_key}:${r.condition_id}` === identity) ||
          (r.kind === "tracker_elapsed" &&
            !!r.result &&
            `tracker:${r.participant_key}:${r.tracker_key}` === identity),
      )
    )
      return;
    const row = Array.from(
      reminderRows.current?.querySelectorAll<HTMLElement>("[data-reminder-identity]") ??
        [],
    ).find((el) => el.dataset.reminderIdentity === identity);
    if (
      document.activeElement === document.body ||
      row?.contains(document.activeElement)
    )
      row
        ?.querySelector<HTMLElement>('[role="status"]')
        ?.focus({ preventScroll: true });
    removalFocus.current = null;
  }, [reminders]);
  const hasPersistentDamage = !!outgoing?.persistent_damage_groups.length;
  const meaningful =
    reminders.length > 0 ||
    hasPersistentDamage ||
    outcomes.length > 0 ||
    !!error ||
    !!actionError ||
    !!recovery;

  return (
    <EncounterModal
      title="Turn reminders"
      width={640}
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
        <ErrorNotice
          error={actionError}
          title="Turn action could not be confirmed"
          description={actionError}
        />
      )}
      {error && (
        <ErrorNotice
          error={error}
          title="Turn review could not load"
          action={
            <Button loading={loading} disabled={pending} onClick={onRetry}>
              Retry turn review
            </Button>
          }
        />
      )}
      {outgoing && (
        <header className="encounter-turn-review__identity">
          <h3>{outgoing.participant_name}</h3>
          <span className="muted">Round {outgoing.round_number.toString()}</span>
        </header>
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
      <div className="encounter-turn-review" ref={reminderRows}>
        {hasPersistentDamage && (
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
            if (reminder.kind === "tracker_elapsed") {
              const identity = `tracker:${reminder.participant_key}:${reminder.tracker_key}`;
              return (
                <section
                  key={identity}
                  data-reminder-identity={identity}
                  className="encounter-turn-reminder encounter-turn-reminder--condition"
                >
                  <div className="encounter-turn-reminder__heading">
                    <h3>
                      {reminder.participant_key !== outgoing?.participant_key &&
                        `${reminder.participant_name}: `}
                      {reminder.tracker_name}
                    </h3>
                  </div>
                  <p role="status" tabIndex={-1}>
                    {reminder.result ?? "Elapsed"}
                  </p>
                  {!reminder.result && (
                    <Button
                      style={{ justifySelf: "start" }}
                      disabled={
                        disabled || pending || loading || !!error || !onRemoveTracker
                      }
                      icon={<DeleteOutlined />}
                      aria-label={`Remove tracker ${reminder.tracker_name} from ${reminder.participant_name}`}
                      onClick={(event) => {
                        if (document.activeElement === event.currentTarget)
                          removalFocus.current = identity;
                        void onRemoveTracker?.(reminder, outgoing!.boundary_key).catch(
                          () => {
                            removalFocus.current = null;
                          },
                        );
                      }}
                    >
                      Remove tracker
                    </Button>
                  )}
                </section>
              );
            }
            if (reminder.kind === "condition_recovery") {
              const participant = participants.find(
                (p) => p.participant_key === reminder.participant_key,
              );
              const identity = `${reminder.participant_key}:${reminder.condition_ids[0]}`;
              return (
                <section
                  key={identity}
                  data-reminder-identity={identity}
                  className="encounter-turn-reminder encounter-turn-reminder--condition"
                >
                  <div className="encounter-turn-reminder__heading">
                    <h3>
                      {reminder.participant_key !== outgoing?.participant_key &&
                        `${reminder.participant_name}: `}
                      {reminder.condition_name}
                      {reminder.condition_value != null
                        ? ` ${reminder.condition_value}`
                        : ""}
                    </h3>
                  </div>
                  {participant ? (
                    <div className="encounter-turn-reminder__actions">
                      <EncounterConditionRecovery
                        key={outgoing!.boundary_key}
                        encounterRef={encounterRef}
                        boundaryKey={outgoing!.boundary_key}
                        revision={data!.expected_revision}
                        participant={participant}
                        disabled={disabled || pending || loading || !!error}
                        contextualTurnActions
                        onApply={async (request) => {
                          const row = Array.from(
                            reminderRows.current?.querySelectorAll<HTMLElement>(
                              "[data-reminder-identity]",
                            ) ?? [],
                          ).find((el) => el.dataset.reminderIdentity === identity);
                          if (row?.contains(document.activeElement))
                            removalFocus.current = identity;
                          try {
                            return await onLifecycle(request);
                          } catch (cause) {
                            removalFocus.current = null;
                            throw cause;
                          }
                        }}
                      >
                        <EncounterTurnRecoveryActions />
                      </EncounterConditionRecovery>
                    </div>
                  ) : (
                    <p>Participant unavailable</p>
                  )}
                </section>
              );
            }
            const removable =
              reminder.kind === "duration_ending" ||
              reminder.kind === "duration_ended" ||
              reminder.kind === "persistent_damage";
            const present = participants.some(
              (participant) =>
                participant.participant_key === reminder.participant_key &&
                participant.conditions.some(
                  (condition) => condition.condition_id === reminder.condition_id,
                ),
            );
            return (
              <section
                key={`${reminder.participant_key}:${reminder.condition_id}`}
                data-reminder-identity={`${reminder.participant_key}:${reminder.condition_id}`}
                className="encounter-turn-reminder encounter-turn-reminder--condition"
              >
                <div className="encounter-turn-reminder__heading">
                  <h3>
                    {reminder.participant_key !== outgoing?.participant_key &&
                      `${reminder.participant_name}: `}
                    {reminder.condition_name}
                  </h3>
                </div>
                <p
                  role={reminder.kind === "resolved" ? "status" : undefined}
                  tabIndex={reminder.kind === "resolved" ? -1 : undefined}
                >
                  {reminderText(reminder, participants)}
                </p>
                {removable && (
                  <Button
                    style={{ justifySelf: "start" }}
                    disabled={disabled || pending || loading || !!error || !present}
                    icon={<DeleteOutlined />}
                    aria-label={`Remove ${reminder.condition_name} from ${reminder.participant_name}`}
                    onClick={(event) => {
                      if (document.activeElement === event.currentTarget)
                        removalFocus.current = `${reminder.participant_key}:${reminder.condition_id}`;
                      void onRemove(reminder, outgoing!.boundary_key).catch(() => {
                        removalFocus.current = null;
                        // The mutation owner shows the failure and retains retry state.
                      });
                    }}
                  >
                    Remove condition
                  </Button>
                )}
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
    case "tracker_elapsed":
      return reminder.result ?? "Elapsed";
    case "resolved":
      return reminder.result;
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

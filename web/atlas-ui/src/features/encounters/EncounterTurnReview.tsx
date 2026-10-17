import { Alert, Button, Modal, Space } from "antd";
import type {
  EncounterHpAction,
  EncounterParticipantView,
  EncounterTurnReminderView,
  EncounterTurnReviewView,
} from "../../generated/atlas";
import { EncounterHpAmount } from "./EncounterHpAmount";
import { useConfirmDangerAction } from "../../shared/ui/actions/useConfirmDangerAction";
import type { ReactNode } from "react";

export function EncounterTurnReview({
  open,
  data,
  participants,
  pending,
  disabled,
  loading,
  error,
  actionError,
  recovery,
  receipt,
  onRetry,
  onCancel,
  onContinue,
  onHpAction,
  onRemove,
}: {
  open: boolean;
  data?: EncounterTurnReviewView;
  participants: EncounterParticipantView[];
  pending: boolean;
  disabled: boolean;
  loading: boolean;
  error: Error | null;
  actionError?: string;
  recovery?: ReactNode;
  receipt: string;
  onRetry: () => void;
  onCancel: () => void;
  onContinue: () => void;
  onHpAction: (
    participant: EncounterParticipantView,
    action: EncounterHpAction,
  ) => Promise<boolean>;
  onRemove: (reminder: EncounterTurnReminderView) => Promise<unknown>;
}) {
  const confirmRemove = useConfirmDangerAction();
  return (
    <Modal
      title="Review turn"
      open={open}
      footer={null}
      onCancel={onCancel}
      closable={!pending}
      keyboard={!pending}
      mask={{ closable: !pending }}
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
      <div className="encounter-turn-review">
        {data?.reminders.map((reminder) => {
          const participant = participants.find(
            (p) => p.participant_key === reminder.participant_key,
          );
          return (
            <section
              key={`${reminder.kind}:${reminder.participant_key}:${reminder.condition_id}`}
              className="encounter-turn-reminder"
            >
              <h3>
                {reminder.participant_name}: {reminder.condition_name}
              </h3>
              <p>{reminderText(reminder, participants)}</p>
              {reminder.kind === "persistent_damage" && (
                <>
                  {reminder.note && <p>{reminder.note}</p>}
                  {participant && (
                    <EncounterHpAmount
                      disabled={
                        disabled ||
                        loading ||
                        !!error ||
                        participant.current_hp === undefined
                      }
                      actions={["damage"]}
                      onAction={(action) => onHpAction(participant, action)}
                    />
                  )}
                </>
              )}
              {(reminder.kind === "persistent_damage" ||
                reminder.kind === "duration_ending" ||
                reminder.kind === "duration_ended") && (
                <Button
                  disabled={disabled || loading || !!error}
                  onClick={() =>
                    confirmRemove({
                      title: `Remove ${reminder.condition_name} from ${reminder.participant_name}?`,
                      okText: "Remove condition",
                      onConfirm: () => onRemove(reminder),
                    })
                  }
                >
                  {reminder.kind === "persistent_damage"
                    ? "Recovery succeeded: remove"
                    : "Remove condition"}
                </Button>
              )}
            </section>
          );
        })}
      </div>
      {data && data.reminders.length === 0 && <p>No remaining reminders.</p>}
      <p className="muted">Saved actions remain applied if you close this review.</p>
      <Space wrap>
        <Button
          type="primary"
          disabled={disabled || loading || !!error || !data}
          loading={pending}
          onClick={onContinue}
        >
          Continue to next turn
        </Button>
        <Button disabled={pending} onClick={onCancel}>
          Cancel
        </Button>
      </Space>
    </Modal>
  );
}
function reminderText(
  reminder: EncounterTurnReminderView,
  participants: EncounterParticipantView[],
): string {
  switch (reminder.kind) {
    case "persistent_damage":
      return "Roll persistent damage, then attempt the recovery flat check.";
    case "duration_ending": {
      const anchor = participants.find(
        (participant) =>
          participant.participant_key === reminder.timing.participant_key,
      );
      const turn = anchor ? `${anchor.display_name}’s turn` : "the configured turn";
      return `Expires at the ${reminder.timing.event} of ${turn} when you continue. Remove now to end this condition immediately.`;
    }
    case "duration_ended":
      return "Duration ended. Confirm removal when ready.";
    case "duration_reminder":
      return `${reminder.duration.amount} ${reminder.duration.unit}: check this duration at the table.`;
    case "timing_unavailable":
      return "Round timing is not configured. Edit the condition to choose whose turn and when its rounds count down.";
  }
}

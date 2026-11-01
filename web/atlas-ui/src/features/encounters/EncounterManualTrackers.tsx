import { Pencil, Plus, RotateCcw, X } from "lucide-react";
import { Button, Form, Input, InputNumber, Select, Space, Tooltip } from "antd";
import { useId, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import type {
  ConditionDurationView,
  EncounterManualTrackerView,
  EncounterParticipantView,
  UpdateEncounterManualTrackerRequest,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { nonNegativeIntegerRule } from "../../shared/ui/forms/nonNegativeIntegerRule";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
import { EncounterModal } from "./EncounterModal";
import { conditionDurationLabel } from "./conditionDurationLabel";

export type ManualTrackerHandler = (
  request: Omit<UpdateEncounterManualTrackerRequest, "request_id">,
) => Promise<{ active: boolean; refreshed: boolean }>;
type Values = {
  name: string;
  amount: number;
  unit: ConditionDurationView["unit"];
  event: "start" | "end";
  participant: string;
};
export function EncounterManualTrackers({
  participant,
  participants,
  encounterRevision,
  disabled,
  pending,
  error,
  recovery,
  onChange,
}: {
  participant: EncounterParticipantView;
  participants: EncounterParticipantView[];
  encounterRevision: bigint;
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  onChange?: ManualTrackerHandler;
}) {
  const heading = useId();
  const formId = useId();
  const addTrigger = useRef<HTMLButtonElement>(null);
  const reloadTrigger = useRef<HTMLButtonElement>(null);
  const saveTrigger = useRef<HTMLButtonElement>(null);
  const opener = useRef<HTMLElement | null>(null);
  const focusRecovery = useRef<{
    target: HTMLElement | null;
    fallback: HTMLElement | null;
  } | null>(null);
  const [committed, setCommitted] = useState(false);
  const [editor, setEditor] = useState<{
    tracker?: EncounterManualTrackerView;
    revision: bigint;
    participantRevision: bigint;
  } | null>(null);
  const stale =
    !!editor &&
    !committed &&
    (editor.revision !== encounterRevision ||
      editor.participantRevision !== participant.revision);
  const editorOpen = editor !== null && !(committed && !disabled);
  const currentTracker = editor?.tracker
    ? participant.manual_trackers.find(
        (t) => t.tracker_key === editor.tracker!.tracker_key,
      )
    : undefined;
  useLayoutEffect(() => {
    if (pending || disabled || !focusRecovery.current) return;
    const recovery = focusRecovery.current;
    focusRecovery.current = null;
    const target = stale
      ? reloadTrigger.current
      : recovery.target?.isConnected
        ? recovery.target
        : recovery.fallback;
    restoreOverlayFocus(target, null);
  }, [pending, disabled, stale, error, committed]);
  const open = (tracker?: EncounterManualTrackerView) => {
    opener.current =
      document.activeElement instanceof HTMLElement ? document.activeElement : null;
    setCommitted(false);
    setEditor({
      tracker,
      revision: encounterRevision,
      participantRevision: participant.revision,
    });
  };
  const apply = async (
    action: UpdateEncounterManualTrackerRequest["action"],
    editing = false,
  ) => {
    if (disabled || !onChange) return;
    focusRecovery.current = {
      target:
        document.activeElement instanceof HTMLElement ? document.activeElement : null,
      fallback: editing ? opener.current : addTrigger.current,
    };
    try {
      const result = await onChange({
        participant_key: participant.participant_key,
        expected_encounter_revision: editing ? editor!.revision : encounterRevision,
        expected_participant_revision: editing
          ? editor!.participantRevision
          : participant.revision,
        action,
      });
      if (result.active && editing) {
        setCommitted(true);
        if (result.refreshed) setEditor(null);
      }
    } catch {
      /* Mutation owner retains feedback and drafts. */
    }
  };
  return (
    <section
      aria-labelledby={heading}
      className="encounter-conditions encounter-manual-trackers"
    >
      <div className="encounter-conditions__header">
        <h3 id={heading}>Trackers</h3>
        <Button
          ref={addTrigger}
          size="small"
          type="text"
          aria-label="Add tracker"
          title="Add tracker"
          icon={<Plus size={16} aria-hidden="true" />}
          disabled={disabled || !onChange}
          onClick={(event) => {
            event.currentTarget.focus({ preventScroll: true });
            open();
          }}
        />
      </div>
      {error && !editor && (
        <ErrorNotice
          error={error}
          title="Tracker update could not be confirmed"
          description={error}
        />
      )}
      {!participant.manual_trackers?.length && <p className="muted">No trackers</p>}
      <div className="encounter-condition-list">
        {participant.manual_trackers?.map((tracker) => (
          <div key={tracker.tracker_key} className="encounter-condition-row">
            <span className="encounter-condition-row__name">{tracker.name}</span>
            <span className="encounter-condition-actions">
              <Tooltip
                title="Restart"
                trigger={["hover", "focus"]}
                open={editorOpen ? false : undefined}
                styles={{ root: { pointerEvents: "none" } }}
              >
                <Button
                  size="small"
                  type="text"
                  icon={<RotateCcw size={14} aria-hidden="true" />}
                  disabled={disabled || !onChange || !tracker.timing_available}
                  aria-label={`Restart ${tracker.name}`}
                  onClick={() =>
                    void apply({ kind: "restart", tracker_key: tracker.tracker_key })
                  }
                />
              </Tooltip>
              <Tooltip
                title="Edit"
                trigger={["hover", "focus"]}
                open={editorOpen ? false : undefined}
                styles={{ root: { pointerEvents: "none" } }}
              >
                <Button
                  size="small"
                  type="text"
                  icon={<Pencil size={14} aria-hidden="true" />}
                  disabled={disabled || !onChange}
                  aria-label={`Edit tracker ${tracker.name}`}
                  onClick={(event) => {
                    event.currentTarget.focus({ preventScroll: true });
                    open(tracker);
                  }}
                />
              </Tooltip>
              <Tooltip
                title="Remove"
                trigger={["hover", "focus"]}
                open={editorOpen ? false : undefined}
                styles={{ root: { pointerEvents: "none" } }}
              >
                <Button
                  size="small"
                  type="text"
                  icon={<X size={14} aria-hidden="true" />}
                  disabled={disabled || !onChange}
                  aria-label={`Remove tracker ${tracker.name}`}
                  onClick={() =>
                    void apply({ kind: "remove", tracker_key: tracker.tracker_key })
                  }
                />
              </Tooltip>
            </span>
            <span className="encounter-condition-meta">
              <span role="status">
                {tracker.remaining_rounds === 0n
                  ? "Ready"
                  : `${tracker.remaining_rounds} ${tracker.remaining_rounds === 1n ? "round" : "rounds"} remaining`}
              </span>
              {tracker.duration.unit !== "rounds"
                ? ` · ${conditionDurationLabel(tracker.duration)}`
                : ""}{" "}
              · {tracker.timing_label}
            </span>
          </div>
        ))}
      </div>
      <EncounterModal
        open={editorOpen}
        title={editor?.tracker ? "Edit tracker" : "Add tracker"}
        onCancel={() => {
          if (!pending) setEditor(null);
        }}
        footer={
          <Space>
            <Button disabled={pending} onClick={() => setEditor(null)}>
              Cancel
            </Button>
            <Button
              type="primary"
              ref={saveTrigger}
              htmlType="submit"
              form={formId}
              disabled={disabled || stale || committed}
              loading={pending}
            >
              Save tracker
            </Button>
          </Space>
        }
        destroyOnHidden
        closable={!pending}
        keyboard={!pending}
        mask={{ closable: !pending }}
      >
        {recovery}
        {stale ? (
          <ErrorNotice
            error={error}
            type="warning"
            title={editor?.tracker ? "Tracker values changed" : "Encounter changed"}
            description={
              editor?.tracker
                ? currentTracker
                  ? "Reload current tracker values to continue. This replaces your draft."
                  : "This tracker was removed. Close this editor to continue."
                : "Keep your draft and use the refreshed encounter to continue."
            }
            action={
              (!editor?.tracker || currentTracker) && (
                <Button
                  ref={reloadTrigger}
                  disabled={disabled}
                  onClick={() => {
                    focusRecovery.current = {
                      target: saveTrigger.current,
                      fallback: opener.current,
                    };
                    setEditor((previous) =>
                      previous
                        ? {
                            ...previous,
                            ...(previous.tracker ? { tracker: currentTracker } : {}),
                            revision: encounterRevision,
                            participantRevision: participant.revision,
                          }
                        : null,
                    );
                  }}
                >
                  {editor?.tracker ? "Reload tracker" : "Use refreshed encounter"}
                </Button>
              )
            }
          />
        ) : (
          error && (
            <ErrorNotice
              error={error}
              title="Tracker update could not be confirmed"
              description={error}
            />
          )
        )}
        <Form<Values>
          name={formId}
          id={formId}
          key={
            editor?.tracker
              ? `${editor.tracker.tracker_key}:${editor.revision}:${editor.participantRevision}`
              : editor
                ? "new"
                : "closed"
          }
          initialValues={{
            name: editor?.tracker?.name ?? "",
            amount: Number(editor?.tracker?.duration.amount ?? 1n),
            unit: editor?.tracker?.duration.unit ?? "rounds",
            event: editor?.tracker?.duration.timing?.event ?? "end",
            participant:
              editor?.tracker?.duration.timing?.participant_key ??
              participant.participant_key,
          }}
          layout="vertical"
          disabled={disabled}
          onFinish={(values) => {
            const duration: ConditionDurationView = {
              amount: BigInt(values.amount),
              unit: values.unit,
              timing: { participant_key: values.participant, event: values.event },
            };
            void apply(
              editor?.tracker
                ? {
                    kind: "edit",
                    tracker_key: editor.tracker.tracker_key,
                    name: values.name,
                    duration,
                  }
                : { kind: "add", name: values.name, duration },
              true,
            );
          }}
        >
          <Form.Item
            name="name"
            label="Name"
            rules={[
              { required: true, whitespace: true, message: "Enter a tracker name." },
              { max: 120 },
            ]}
          >
            <Input maxLength={120} />
          </Form.Item>
          <Space align="start" wrap>
            <Form.Item
              name="amount"
              label="Duration"
              extra={
                editor?.tracker
                  ? "Changing duration or timing restarts the countdown."
                  : undefined
              }
              rules={[nonNegativeIntegerRule(true)]}
            >
              <InputNumber
                aria-label="Tracker duration"
                min={0}
                max={Number.MAX_SAFE_INTEGER}
                precision={0}
              />
            </Form.Item>
            <Form.Item name="unit" label="Unit" rules={[{ required: true }]}>
              <Select
                aria-label="Tracker duration unit"
                style={{ minWidth: 120 }}
                options={["rounds", "minutes", "hours", "days"].map((value) => ({
                  value,
                  label: value,
                }))}
              />
            </Form.Item>
          </Space>
          <Space align="start" wrap>
            <Form.Item name="event" label="Count down at" rules={[{ required: true }]}>
              <Select
                aria-label="Tracker turn boundary"
                style={{ minWidth: 130 }}
                options={[
                  { value: "start", label: "Start of turn" },
                  { value: "end", label: "End of turn" },
                ]}
              />
            </Form.Item>
            <Form.Item
              name="participant"
              label="Whose turn"
              rules={[
                { required: true, message: "Choose whose turn counts down." },
                {
                  validator: (_, value) =>
                    participants.some((p) => p.participant_key === value)
                      ? Promise.resolve()
                      : Promise.reject(
                          new Error("Choose a participant in this encounter."),
                        ),
                },
              ]}
            >
              <Select
                aria-label="Tracker timing participant"
                style={{ minWidth: 180 }}
                options={participants.map((p) => ({
                  value: p.participant_key,
                  label: p.display_name,
                }))}
              />
            </Form.Item>
          </Space>
        </Form>
      </EncounterModal>
    </section>
  );
}

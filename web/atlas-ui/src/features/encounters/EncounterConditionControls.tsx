import { recordPath, shouldHandleAtlasRouteClick } from "../../app/routes";
import { Alert, Button, Modal } from "antd";
import { useState, type ReactNode } from "react";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterConditionDefinitionView,
  EncounterParticipantConditionView,
  EncounterParticipantView,
  UpdateEncounterParticipantConditionRequest,
} from "../../generated/atlas";
import { useConfirmDangerAction } from "../../shared/ui/actions/useConfirmDangerAction";
import {
  EncounterConditionEditor,
  conditionDuration,
  type ConditionFormValues,
} from "./EncounterConditionEditor";
import { EncounterConditionValue } from "./EncounterConditionValue";

export function EncounterConditionControls({
  disabled,
  encounterRevision,
  pending,
  addDisabled,
  error,
  recovery,
  current,
  conditionDefinitions,
  onAddCondition,
  onRemoveCondition,
  onReference,
  onUpdateCondition,
  participants,
}: {
  disabled: boolean;
  encounterRevision: bigint;
  pending: boolean;
  addDisabled: boolean;
  error?: string;
  recovery?: ReactNode;
  current: EncounterParticipantView;
  conditionDefinitions: EncounterConditionDefinitionView[];
  onAddCondition: (
    condition: AddEncounterParticipantConditionRequest,
  ) => Promise<boolean>;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => Promise<boolean>;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => Promise<boolean>;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  participants: EncounterParticipantView[];
}) {
  const [editor, setEditor] = useState<
    "add" | { conditionId: bigint; revision: bigint } | null
  >(null);
  const [committed, setCommitted] = useState(false);
  const confirmRemove = useConfirmDangerAction();
  const selected =
    editor !== null && editor !== "add"
      ? current.conditions.find((c) => c.condition_id === editor.conditionId)
      : undefined;
  const open = editor !== null && !(committed && !disabled);
  const close = () => setEditor(null);
  const finished = (refreshed: boolean) => {
    if (refreshed) close();
    else setCommitted(true);
  };
  const update = (
    condition: EncounterParticipantConditionView,
    changes: Partial<UpdateEncounterParticipantConditionRequest>,
    revision = encounterRevision,
  ) =>
    onUpdateCondition(current.participant_key, {
      expected_encounter_revision: revision,
      condition_id: condition.condition_id,
      name: condition.name,
      value: condition.value,
      duration: condition.duration,
      source_participant_key: condition.source_participant_key,
      note: condition.note,
      ...changes,
    });
  const save = async (values: ConditionFormValues, durationEdited: boolean) => {
    if (disabled) return;
    if (selected) {
      finished(
        await update(
          selected,
          {
            value:
              values.value === null || values.value === undefined
                ? undefined
                : BigInt(values.value),
            duration: durationEdited ? conditionDuration(values) : selected.duration,
            source_participant_key: values.sourceParticipantKey,
            note: values.note || undefined,
          },
          editor !== null && editor !== "add" ? editor.revision : encounterRevision,
        ),
      );
    } else {
      const definition = conditionDefinitions.find(
        (d) => d.condition_ref === values.conditionRef,
      );
      if (!definition) return;
      finished(
        await onAddCondition({
          participant_key: current.participant_key,
          condition_ref: definition.condition_ref,
          value:
            definition.has_value && values.value !== null && values.value !== undefined
              ? BigInt(values.value)
              : undefined,
          duration: conditionDuration(values),
          source_participant_key: values.sourceParticipantKey,
          note: values.note || undefined,
        }),
      );
    }
  };
  return (
    <section className="encounter-conditions">
      <div className="encounter-conditions__header">
        <h3>Conditions</h3>
        <Button
          size="small"
          disabled={disabled || addDisabled}
          onClick={() => {
            setCommitted(false);
            setEditor("add");
          }}
        >
          Add Condition
        </Button>
      </div>
      {error && !open && (
        <Alert type="error" title="Condition action failed" description={error} />
      )}
      {current.conditions.length === 0 ? (
        <p className="encounter-empty-note">No conditions</p>
      ) : (
        <div className="encounter-condition-list">
          {current.conditions.map((condition) => {
            const definition = conditionDefinitions.find(
              (d) => d.condition_ref === condition.condition_key,
            );
            const hasValue = definition?.has_value ?? condition.value !== undefined;
            return (
              <div
                className="encounter-condition-row"
                key={condition.condition_id.toString()}
              >
                {condition.condition_key ? (
                  <Button
                    type="link"
                    className="encounter-condition-row__name"
                    style={{ padding: 0, height: "auto", justifyContent: "start" }}
                    href={recordPath(condition.condition_key)}
                    onClick={(event) => {
                      if (!shouldHandleAtlasRouteClick(event)) return;
                      event.preventDefault();
                      onReference(condition.condition_key!, event.currentTarget);
                    }}
                  >
                    {condition.name}
                  </Button>
                ) : (
                  <span className="encounter-condition-row__name">
                    {condition.name}
                  </span>
                )}
                {hasValue && (
                  <EncounterConditionValue
                    name={condition.name}
                    value={condition.value}
                    disabled={disabled}
                    onCommit={(value) => update(condition, { value })}
                  />
                )}
                <Button
                  size="small"
                  disabled={disabled}
                  aria-label={`Edit ${condition.name}`}
                  onClick={() => {
                    setCommitted(false);
                    setEditor({
                      conditionId: condition.condition_id,
                      revision: encounterRevision,
                    });
                  }}
                >
                  Edit
                </Button>
                {condition.duration && (
                  <span className="encounter-condition-meta">
                    {condition.duration.unit === "rounds" &&
                    condition.duration.amount === 0n
                      ? "Duration ended"
                      : `${condition.duration.amount} ${condition.duration.unit}`}
                    {condition.duration.unit === "rounds" &&
                    condition.duration.timing === null
                      ? " · timing not configured"
                      : ""}
                  </span>
                )}
              </div>
            );
          })}
        </div>
      )}
      <Modal
        title={selected ? `Edit ${selected.name}` : "Add condition"}
        open={open}
        destroyOnHidden
        footer={null}
        onCancel={close}
        closable={!pending}
        keyboard={!pending}
        mask={{ closable: !pending }}
      >
        <EncounterConditionEditor
          key={
            editor === null
              ? "closed"
              : editor === "add"
                ? "add"
                : `${editor.conditionId}:${editor.revision}`
          }
          condition={selected}
          definitions={conditionDefinitions}
          participants={participants}
          disabled={disabled || (editor === "add" && addDisabled)}
          pending={pending}
          error={error}
          recovery={recovery}
          onReload={
            editor !== null && editor !== "add" && editor.revision !== encounterRevision
              ? () => setEditor({ ...editor, revision: encounterRevision })
              : undefined
          }
          onSave={save}
          onCancel={close}
          onRemove={
            selected
              ? () =>
                  confirmRemove({
                    title: `Remove ${selected.name}?`,
                    okText: "Remove condition",
                    onConfirm: async () =>
                      finished(
                        await onRemoveCondition(
                          current.participant_key,
                          selected.condition_id,
                        ),
                      ),
                  })
              : undefined
          }
        />
      </Modal>
    </section>
  );
}

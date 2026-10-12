import { Alert, Button, Form, Input, InputNumber, Popover, Select } from "antd";
import type { GetRef } from "antd";
import { MoreHorizontal, Trash2 } from "lucide-react";
import { useRef, useState } from "react";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterConditionDefinitionView,
  EncounterParticipantConditionView,
  EncounterParticipantView,
  UpdateEncounterParticipantConditionRequest,
} from "../../generated/atlas";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";
import { optionalNumber } from "./participantEdits";

type AddConditionForm = {
  conditionRef: string;
  value?: number;
  duration?: number;
  sourceParticipantKey?: string;
  note?: string;
};

type ConditionDetailsForm = {
  duration?: number;
  sourceParticipantKey?: string;
  note?: string;
};

export function EncounterConditionControls({
  disabled,
  addDisabled,
  error,
  current,
  conditionDefinitions,
  onAddCondition,
  onRemoveCondition,
  onReference,
  onUpdateCondition,
  participants,
}: {
  disabled: boolean;
  addDisabled: boolean;
  error?: string;
  current: EncounterParticipantView;
  conditionDefinitions: EncounterConditionDefinitionView[];
  onAddCondition: (
    condition: AddEncounterParticipantConditionRequest,
  ) => Promise<unknown>;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => Promise<unknown>;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => Promise<unknown>;
  participants: EncounterParticipantView[];
}) {
  const [conditionForm] = Form.useForm<AddConditionForm>();
  const [addConditionOpen, setAddConditionOpen] = useState(false);
  const [addDetailsOpen, setAddDetailsOpen] = useState(false);
  const addConditionSelectRef = useRef<GetRef<typeof Select>>(null);
  const addConditionRef = Form.useWatch("conditionRef", conditionForm);
  const addCondition = conditionDefinition(addConditionRef, conditionDefinitions);
  const addConditionHasValue = addCondition?.has_value ?? false;
  const openAddCondition = (open: boolean) => {
    if (disabled || addDisabled) return;
    setAddConditionOpen(open);
    if (open) {
      window.setTimeout(() => addConditionSelectRef.current?.focus(), 0);
    }
  };

  return (
    <section className="encounter-conditions">
      <div className="encounter-conditions__header">
        <h3>Conditions</h3>
        <Popover
          content={
            <Form
              disabled={disabled || addDisabled}
              form={conditionForm}
              onFinish={async (values) => {
                if (disabled || addDisabled) return;
                const condition = conditionDefinition(
                  values.conditionRef,
                  conditionDefinitions,
                );
                if (!condition) {
                  return;
                }
                try {
                  await onAddCondition({
                    participant_key: current.participant_key,
                    condition_ref: condition.condition_ref,
                    ...(!condition.has_value || values.value === undefined
                      ? {}
                      : { value: BigInt(values.value) }),
                    ...(values.duration === undefined
                      ? {}
                      : { duration_rounds: BigInt(values.duration) }),
                    ...(values.sourceParticipantKey
                      ? { source_participant_key: values.sourceParticipantKey }
                      : {}),
                    ...(values.note ? { note: values.note } : {}),
                  });
                  conditionForm.resetFields();
                  setAddDetailsOpen(false);
                  setAddConditionOpen(false);
                } catch {
                  /* Keep the failed addition draft. */
                }
              }}
            >
              {error && (
                <Alert
                  type="error"
                  title="Condition action failed"
                  description={error}
                />
              )}
              <div className="encounter-add-condition__row">
                <Form.Item
                  style={{ marginBottom: 0 }}
                  name="conditionRef"
                  rules={[{ required: true }]}
                >
                  <Select
                    aria-label="Add condition"
                    ref={addConditionSelectRef}
                    showSearch={{ optionFilterProp: "label" }}
                    options={conditionDefinitions.map((condition) => ({
                      label: condition.name,
                      value: condition.condition_ref,
                    }))}
                    placeholder="Condition"
                    onChange={(conditionRef) => {
                      const condition = conditionDefinition(
                        conditionRef,
                        conditionDefinitions,
                      );
                      conditionForm.setFieldValue(
                        "value",
                        condition?.has_value
                          ? (optionalNumber(condition.default_value) ?? 1)
                          : undefined,
                      );
                    }}
                  />
                </Form.Item>
                {addConditionHasValue && (
                  <Form.Item style={{ marginBottom: 0 }} name="value">
                    <InputNumber
                      className="encounter-condition-number"
                      aria-label="Condition value"
                      min={0}
                    />
                  </Form.Item>
                )}
                <Popover
                  content={<ConditionDetailsFields participants={participants} />}
                  open={addDetailsOpen}
                  onOpenChange={setAddDetailsOpen}
                  placement="bottomRight"
                  trigger="click"
                >
                  <Button
                    aria-label="Condition details"
                    icon={<MoreHorizontal size={14} />}
                  />
                </Popover>
                <Button
                  disabled={disabled || addDisabled}
                  type="primary"
                  onClick={() => conditionForm.submit()}
                >
                  Add
                </Button>
              </div>
            </Form>
          }
          open={addConditionOpen}
          onOpenChange={openAddCondition}
          placement="bottomRight"
          trigger="click"
        >
          <Button disabled={disabled || addDisabled} size="small">
            Add Condition
          </Button>
        </Popover>
      </div>
      {current.conditions.length === 0 ? (
        <p className="encounter-empty-note">No conditions</p>
      ) : (
        <div className="encounter-condition-list">
          {current.conditions.map((condition) => (
            <ConditionEditor
              disabled={disabled}
              error={error}
              condition={condition}
              key={condition.condition_id.toString()}
              participantKey={current.participant_key}
              participants={participants}
              conditionDefinitions={conditionDefinitions}
              onReference={onReference}
              onRemove={onRemoveCondition}
              onUpdate={onUpdateCondition}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function ConditionDetailsFields({
  participants,
}: {
  participants: EncounterParticipantView[];
}) {
  return (
    <div className="encounter-condition-details">
      <Form.Item style={{ marginBottom: 0 }} name="duration" label="Duration">
        <InputNumber
          className="encounter-condition-number"
          aria-label="Duration rounds"
          min={0}
        />
      </Form.Item>
      <Form.Item style={{ marginBottom: 0 }} name="sourceParticipantKey" label="Source">
        <Select
          allowClear
          aria-label="Source"
          options={participants.map((participant) => ({
            value: participant.participant_key,
            label: participant.display_name,
          }))}
        />
      </Form.Item>
      <Form.Item style={{ marginBottom: 0 }} name="note" label="Note">
        <Input.TextArea
          aria-label="Condition note"
          autoSize={{ minRows: 2, maxRows: 8 }}
          className="encounter-condition-note-input"
        />
      </Form.Item>
    </div>
  );
}

function ConditionEditor({
  disabled,
  error,
  condition,
  participantKey,
  participants,
  conditionDefinitions,
  onReference,
  onRemove,
  onUpdate,
}: {
  disabled: boolean;
  error?: string;
  condition: EncounterParticipantConditionView;
  participantKey: string;
  participants: EncounterParticipantView[];
  conditionDefinitions: EncounterConditionDefinitionView[];
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  onRemove: (participantKey: string, conditionId: bigint) => Promise<unknown>;
  onUpdate: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => Promise<unknown>;
}) {
  const [detailsForm] = Form.useForm<ConditionDetailsForm>();
  const [detailsOpen, setDetailsOpen] = useState(false);
  const hasValue = conditionTakesValue(condition, conditionDefinitions);
  const hasDetails = Boolean(
    condition.duration_rounds !== undefined ||
    condition.source_participant_key ||
    condition.note,
  );
  const update = (changes: Partial<UpdateEncounterParticipantConditionRequest>) =>
    onUpdate(participantKey, {
      condition_id: condition.condition_id,
      name: condition.name,
      value: condition.value,
      source_participant_key: condition.source_participant_key,
      duration_rounds: condition.duration_rounds,
      note: condition.note,
      ...changes,
    });
  const saveDetails = async () => {
    if (disabled) return;
    const values = detailsForm.getFieldsValue();
    try {
      await update({
        duration_rounds:
          values.duration === undefined ? undefined : BigInt(values.duration),
        source_participant_key: values.sourceParticipantKey,
        note: values.note || undefined,
      });
      detailsForm.resetFields();
      setDetailsOpen(false);
    } catch {
      /* Keep the failed details draft. */
    }
  };
  const openDetails = (open: boolean) => {
    if (disabled) return;
    if (open && !detailsForm.isFieldsTouched()) {
      detailsForm.setFieldsValue({
        duration: optionalNumber(condition.duration_rounds),
        sourceParticipantKey: condition.source_participant_key,
        note: condition.note,
      });
    }
    setDetailsOpen(open);
  };
  return (
    <div
      className={[
        "encounter-condition-row",
        hasValue ? "" : "encounter-condition-row--no-value",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {condition.condition_key ? (
        <Button
          type="link"
          style={{ padding: 0, height: "auto", justifyContent: "start" }}
          className="encounter-condition-row__name"
          onClick={(event) =>
            onReference(condition.condition_key!, event.currentTarget)
          }
        >
          {condition.name}
        </Button>
      ) : (
        <span className="encounter-condition-row__name">{condition.name}</span>
      )}
      {hasValue && (
        <EditableCommitField
          disabled={disabled}
          ariaLabel={`${condition.name} value`}
          inputMode="numeric"
          onCommit={(value) => commitConditionValue(value, update)}
          size="small"
          value={inputNumberValue(condition.value)}
        />
      )}
      {condition.duration_rounds !== undefined && (
        <span className="encounter-condition-chip">
          {condition.duration_rounds.toString()} rounds
        </span>
      )}
      {hasDetails && <span className="encounter-condition-meta">Details</span>}
      <Popover
        content={
          <Form disabled={disabled} form={detailsForm} layout="vertical">
            {error && (
              <Alert type="error" title="Condition action failed" description={error} />
            )}
            <ConditionDetailsFields participants={participants} />
            <div className="encounter-condition-details__actions">
              <Button
                disabled={disabled}
                type="primary"
                onClick={() => void saveDetails()}
              >
                Save
              </Button>
            </div>
          </Form>
        }
        open={detailsOpen}
        onOpenChange={openDetails}
        placement="bottom"
        trigger="click"
      >
        <Button
          disabled={disabled}
          aria-label={`Edit ${condition.name} details`}
          icon={<MoreHorizontal size={14} />}
          size="small"
        />
      </Popover>
      <Button
        disabled={disabled}
        danger
        aria-label={`Remove ${condition.name}`}
        icon={<Trash2 size={12} />}
        size="small"
        onClick={() =>
          void onRemove(participantKey, condition.condition_id).catch(() => {})
        }
      />
    </div>
  );
}

function conditionDefinition(
  conditionRef: string | undefined,
  definitions: EncounterConditionDefinitionView[],
): EncounterConditionDefinitionView | undefined {
  return definitions.find((condition) => condition.condition_ref === conditionRef);
}

function conditionTakesValue(
  condition: EncounterParticipantConditionView,
  definitions: EncounterConditionDefinitionView[],
): boolean {
  const definition = conditionDefinition(condition.condition_key, definitions);
  return definition?.has_value ?? condition.value !== undefined;
}

function commitConditionValue(
  value: string,
  update: (
    changes: Partial<UpdateEncounterParticipantConditionRequest>,
  ) => Promise<unknown>,
) {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return update({ value: undefined });
  }
  if (!/^\d+$/.test(trimmed)) {
    return;
  }
  return update({ value: BigInt(trimmed) });
}

function inputNumberValue(value: bigint | undefined): string {
  return value === undefined ? "" : value.toString();
}

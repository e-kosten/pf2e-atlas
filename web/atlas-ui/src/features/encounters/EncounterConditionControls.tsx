import { Button, Form, Input, InputNumber, Popover, Select } from "antd";
import { MoreHorizontal, Trash2 } from "lucide-react";
import { useState } from "react";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterParticipantConditionView,
  EncounterParticipantView,
  UpdateEncounterParticipantConditionRequest,
} from "../../generated/atlas";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";
import { optionalNumber } from "./participantEdits";

type AddConditionForm = {
  name: string;
  value?: number;
  duration?: number;
  sourceParticipantKey?: string;
  note?: string;
  sourceNote?: string;
};

type ConditionDetailsForm = {
  duration?: number;
  sourceParticipantKey?: string;
  note?: string;
  sourceNote?: string;
};

const MODELED_CONDITIONS = [
  { label: "Frightened", value: "Frightened", hasValue: true },
  { label: "Sickened", value: "Sickened", hasValue: true },
  { label: "Off-Guard", value: "Off-Guard", hasValue: false },
  { label: "Clumsy", value: "Clumsy", hasValue: true },
  { label: "Enfeebled", value: "Enfeebled", hasValue: true },
  { label: "Stupefied", value: "Stupefied", hasValue: true },
];

const MODELED_CONDITION_OPTIONS = MODELED_CONDITIONS.map(({ label, value }) => ({
  label,
  value,
}));

export function EncounterConditionControls({
  current,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  participants,
}: {
  current: EncounterParticipantView;
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  participants: EncounterParticipantView[];
}) {
  const [conditionForm] = Form.useForm<AddConditionForm>();
  const [addConditionOpen, setAddConditionOpen] = useState(false);
  const [addDetailsOpen, setAddDetailsOpen] = useState(false);
  const addConditionName = Form.useWatch("name", conditionForm);
  const addConditionHasValue = conditionTakesValue(addConditionName);

  return (
    <section className="encounter-conditions">
      <div className="encounter-conditions__header">
        <h3>Conditions</h3>
        <Popover
          content={
            <Form
              form={conditionForm}
              onFinish={(values) => {
                const conditionHasValue = conditionTakesValue(values.name);
                onAddCondition({
                  participant_key: current.participant_key,
                  name: values.name,
                  ...(!conditionHasValue || values.value === undefined
                    ? {}
                    : { value: BigInt(values.value) }),
                  ...(values.duration === undefined
                    ? {}
                    : { duration_rounds: BigInt(values.duration) }),
                  ...(values.sourceParticipantKey
                    ? { source_participant_key: values.sourceParticipantKey }
                    : {}),
                  ...(values.note ? { note: values.note } : {}),
                  ...(values.sourceNote ? { source_note: values.sourceNote } : {}),
                });
                conditionForm.resetFields();
                setAddDetailsOpen(false);
                setAddConditionOpen(false);
              }}
            >
              <div className="encounter-add-condition__row">
                <Form.Item name="name" rules={[{ required: true }]}>
                  <Select
                    aria-label="Add condition"
                    showSearch
                    optionFilterProp="label"
                    options={MODELED_CONDITION_OPTIONS}
                    placeholder="Condition"
                    onChange={(name) =>
                      conditionForm.setFieldValue(
                        "value",
                        conditionTakesValue(name) ? 1 : undefined,
                      )
                    }
                  />
                </Form.Item>
                {addConditionHasValue && (
                  <Form.Item name="value">
                    <InputNumber aria-label="Condition value" min={0} />
                  </Form.Item>
                )}
                <Popover
                  content={
                    <ConditionDetailsFields
                      participants={participants}
                      sourceLabel="Source"
                    />
                  }
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
                <Button type="primary" onClick={() => conditionForm.submit()}>
                  Add
                </Button>
              </div>
            </Form>
          }
          open={addConditionOpen}
          onOpenChange={setAddConditionOpen}
          placement="bottomRight"
          trigger="click"
        >
          <Button size="small">Add Condition</Button>
        </Popover>
      </div>
      {current.conditions.length === 0 ? (
        <p className="encounter-empty-note">No conditions</p>
      ) : (
        <div className="encounter-condition-list">
          {current.conditions.map((condition) => (
            <ConditionEditor
              condition={condition}
              key={condition.condition_id.toString()}
              participantKey={current.participant_key}
              participants={participants}
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
  sourceLabel,
}: {
  participants: EncounterParticipantView[];
  sourceLabel: string;
}) {
  return (
    <div className="encounter-condition-details">
      <Form.Item name="duration" label="Duration">
        <InputNumber aria-label="Duration rounds" min={0} />
      </Form.Item>
      <Form.Item name="sourceParticipantKey" label={sourceLabel}>
        <Select
          allowClear
          aria-label={sourceLabel}
          options={participants.map((participant) => ({
            value: participant.participant_key,
            label: participant.display_name,
          }))}
        />
      </Form.Item>
      <Form.Item name="note" label="Note">
        <Input aria-label="Condition note" />
      </Form.Item>
      <Form.Item name="sourceNote" label="Source Note">
        <Input aria-label="Condition source note" />
      </Form.Item>
    </div>
  );
}

function ConditionEditor({
  condition,
  participantKey,
  participants,
  onRemove,
  onUpdate,
}: {
  condition: EncounterParticipantConditionView;
  participantKey: string;
  participants: EncounterParticipantView[];
  onRemove: (participantKey: string, conditionId: bigint) => void;
  onUpdate: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
}) {
  const [detailsForm] = Form.useForm<ConditionDetailsForm>();
  const [detailsOpen, setDetailsOpen] = useState(false);
  const hasValue = conditionTakesValue(condition.name);
  const hasDetails = Boolean(
    condition.duration_rounds !== undefined ||
    condition.source_participant_key ||
    condition.note ||
    condition.source_note,
  );
  const update = (changes: Partial<UpdateEncounterParticipantConditionRequest>) =>
    onUpdate(participantKey, {
      condition_id: condition.condition_id,
      name: condition.name,
      value: condition.value,
      source_participant_key: condition.source_participant_key,
      duration_rounds: condition.duration_rounds,
      note: condition.note,
      source_note: condition.source_note,
      ...changes,
    });
  const saveDetails = () => {
    const values = detailsForm.getFieldsValue();
    update({
      duration_rounds:
        values.duration === undefined ? undefined : BigInt(values.duration),
      source_participant_key: values.sourceParticipantKey,
      note: values.note || undefined,
      source_note: values.sourceNote || undefined,
    });
    setDetailsOpen(false);
  };
  const openDetails = (open: boolean) => {
    if (open) {
      detailsForm.setFieldsValue({
        duration: optionalNumber(condition.duration_rounds),
        sourceParticipantKey: condition.source_participant_key,
        note: condition.note,
        sourceNote: condition.source_note,
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
      <span className="encounter-condition-row__name">{condition.name}</span>
      {hasValue && (
        <EditableCommitField
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
          <Form form={detailsForm} layout="vertical">
            <ConditionDetailsFields
              participants={participants}
              sourceLabel="Condition source"
            />
            <div className="encounter-condition-details__actions">
              <Button type="primary" onClick={saveDetails}>
                Save
              </Button>
            </div>
          </Form>
        }
        open={detailsOpen}
        onOpenChange={openDetails}
        placement="bottomRight"
        trigger="click"
      >
        <Button
          aria-label={`Edit ${condition.name} details`}
          icon={<MoreHorizontal size={14} />}
          size="small"
        />
      </Popover>
      <Button
        danger
        aria-label={`Remove ${condition.name}`}
        icon={<Trash2 size={12} />}
        size="small"
        onClick={() => onRemove(participantKey, condition.condition_id)}
      />
    </div>
  );
}

function conditionTakesValue(name: string | undefined): boolean {
  return (
    MODELED_CONDITIONS.find((condition) => condition.value === name)?.hasValue ?? true
  );
}

function commitConditionValue(
  value: string,
  update: (changes: Partial<UpdateEncounterParticipantConditionRequest>) => void,
) {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    update({ value: undefined });
    return;
  }
  if (!/^\d+$/.test(trimmed)) {
    return;
  }
  update({ value: BigInt(trimmed) });
}

function inputNumberValue(value: bigint | undefined): string {
  return value === undefined ? "" : value.toString();
}

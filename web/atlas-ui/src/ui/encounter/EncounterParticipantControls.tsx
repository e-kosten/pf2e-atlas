import { Button, Form, Input, InputNumber, Popover, Select } from "antd";
import { MoreHorizontal, Trash2 } from "lucide-react";
import { useState } from "react";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterParticipantConditionView,
  EncounterParticipantSideView,
  EncounterParticipantView,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";

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

export function EncounterParticipantControls({
  current,
  participants,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  onUpdate,
}: {
  current: EncounterParticipantView | null;
  participants: EncounterParticipantView[];
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
}) {
  const [hpDraft, setHpDraft] = useState<{
    participantKey: string;
    value: string;
  } | null>(null);
  const [tempHpDraft, setTempHpDraft] = useState<{
    participantKey: string;
    value: string;
  } | null>(null);
  const [amountDraft, setAmountDraft] = useState<{
    participantKey: string;
    value: number | null;
  } | null>(null);
  const [conditionForm] = Form.useForm<AddConditionForm>();
  const [addConditionOpen, setAddConditionOpen] = useState(false);
  const [addDetailsOpen, setAddDetailsOpen] = useState(false);
  const [projectedCurrent, setProjectedCurrent] = useState<{
    source: EncounterParticipantView | null;
    participant: EncounterParticipantView | null;
  }>({ source: null, participant: null });
  const activeCurrent =
    projectedCurrent.source === current &&
    projectedCurrent.participant?.participant_key === current?.participant_key
      ? projectedCurrent.participant
      : current;
  const currentParticipantKey = activeCurrent?.participant_key ?? null;
  const hpInput =
    currentParticipantKey && hpDraft?.participantKey === currentParticipantKey
      ? hpDraft.value
      : (activeCurrent?.current_hp?.toString() ?? "");
  const tempHpInput =
    currentParticipantKey && tempHpDraft?.participantKey === currentParticipantKey
      ? tempHpDraft.value
      : (activeCurrent?.temporary_hp?.toString() ?? "");
  const amount =
    currentParticipantKey && amountDraft?.participantKey === currentParticipantKey
      ? amountDraft.value
      : null;
  const applyHpInput = () => {
    if (!activeCurrent) {
      return;
    }
    const hp = evaluateHpFormula(hpInput);
    if (hp !== null) {
      const clampedHp = clampCurrentHp(activeCurrent, hp);
      updateParticipant({
        current_hp: BigInt(clampedHp),
        defeated: clampedHp === 0 ? true : activeCurrent.defeated,
      });
      setHpDraft({
        participantKey: activeCurrent.participant_key,
        value: clampedHp.toString(),
      });
    }
  };
  const applyTempHpInput = () => {
    if (!activeCurrent) {
      return;
    }
    const temporaryHp = evaluateHpFormula(tempHpInput);
    if (temporaryHp !== null) {
      updateParticipant({
        temporary_hp: BigInt(temporaryHp),
      });
      setTempHpDraft({
        participantKey: activeCurrent.participant_key,
        value: temporaryHp.toString(),
      });
    }
  };
  const applyHpChangeInput = () => {
    if (!activeCurrent || amount === null || amount === 0) {
      return;
    }
    const changes =
      amount < 0
        ? damageChanges(activeCurrent, Math.abs(amount))
        : healChanges(activeCurrent, amount);
    updateParticipant(changes);
    setAmountDraft({
      participantKey: activeCurrent.participant_key,
      value: null,
    });
  };
  const updateParticipant = (changes: Partial<UpdateEncounterParticipantRequest>) => {
    if (!activeCurrent) {
      return;
    }
    const request = participantUpdate(activeCurrent, changes);
    setProjectedCurrent({
      source: current,
      participant: applyParticipantUpdate(activeCurrent, request),
    });
    onUpdate(request);
  };
  const currentHp = asNumber(activeCurrent?.current_hp);
  const maxHp = asNumber(activeCurrent?.max_hp);
  const temporaryHp = asNumber(activeCurrent?.temporary_hp);
  const missingHp = Math.max(0, maxHp - currentHp);
  const hpMeterTotal = Math.max(maxHp, currentHp + missingHp + temporaryHp);
  const hpPercent =
    hpMeterTotal > 0 ? Math.min(100, Math.max(0, (currentHp / hpMeterTotal) * 100)) : 0;
  const missingHpPercent =
    hpMeterTotal > 0 ? Math.min(100, Math.max(0, (missingHp / hpMeterTotal) * 100)) : 0;
  const tempHpPercent =
    hpMeterTotal > 0
      ? Math.min(100, Math.max(0, (temporaryHp / hpMeterTotal) * 100))
      : 0;
  const hpRatio = maxHp > 0 ? currentHp / maxHp : 1;
  const hpSummary = hpLabel(activeCurrent);
  const addConditionName = Form.useWatch("name", conditionForm);
  const addConditionHasValue = conditionTakesValue(addConditionName);
  const hpMeterTone =
    hpRatio <= 0.25
      ? "encounter-hp-meter__current--critical"
      : hpRatio <= 0.5
        ? "encounter-hp-meter__current--bloodied"
        : "";

  return (
    <section className="encounter-participant-controls">
      {activeCurrent ? (
        <div key={activeCurrent.participant_key} className="encounter-turn__body">
          <div className="encounter-form-grid">
            <Form.Item label="Name" layout="vertical">
              <Input
                defaultValue={activeCurrent.display_name}
                onBlur={(event) =>
                  updateParticipant({ display_name: event.target.value })
                }
              />
            </Form.Item>
            <Form.Item label="Initiative" layout="vertical">
              <InputNumber
                defaultValue={optionalNumber(activeCurrent.initiative)}
                onBlur={(event) =>
                  updateParticipant({
                    initiative:
                      event.target.value === ""
                        ? undefined
                        : BigInt(Number(event.target.value)),
                  })
                }
              />
            </Form.Item>
          </div>
          <div className="encounter-hp-panel">
            <div className="encounter-hp-panel__header">
              <h3>HP</h3>
              <span>{hpSummary}</span>
            </div>
            <div
              className="encounter-hp-meter"
              aria-label={`HP remaining: ${hpSummary}`}
            >
              <span
                className={["encounter-hp-meter__current", hpMeterTone]
                  .filter(Boolean)
                  .join(" ")}
                style={{ width: `${hpPercent}%` }}
              />
              {missingHp > 0 && (
                <span
                  className="encounter-hp-meter__missing"
                  style={{ left: `${hpPercent}%`, width: `${missingHpPercent}%` }}
                />
              )}
              {temporaryHp > 0 && (
                <span
                  className="encounter-hp-meter__temporary"
                  style={{
                    left: `${hpPercent + missingHpPercent}%`,
                    width: `${tempHpPercent}%`,
                  }}
                />
              )}
            </div>
            <div className="encounter-hp-grid">
              <div className="encounter-hp-control">
                <span className="encounter-hp-control__label">HP</span>
                <div className="encounter-hp-control__row">
                  <Form.Item layout="vertical">
                    <Input
                      aria-label="HP"
                      value={hpInput}
                      onChange={(event) =>
                        setHpDraft({
                          participantKey: activeCurrent.participant_key,
                          value: event.target.value,
                        })
                      }
                      onKeyDown={(event) => {
                        if (event.key === "Enter") {
                          event.preventDefault();
                          applyHpInput();
                        }
                      }}
                    />
                  </Form.Item>
                  <div className="encounter-hp-actions">
                    <Button onClick={applyHpInput}>Set</Button>
                  </div>
                </div>
              </div>
              <div className="encounter-hp-control">
                <span className="encounter-hp-control__label">Temp HP</span>
                <div className="encounter-hp-control__row">
                  <Form.Item layout="vertical">
                    <Input
                      aria-label="Temp HP"
                      value={tempHpInput}
                      onChange={(event) =>
                        setTempHpDraft({
                          participantKey: activeCurrent.participant_key,
                          value: event.target.value,
                        })
                      }
                      onKeyDown={(event) => {
                        if (event.key === "Enter") {
                          event.preventDefault();
                          applyTempHpInput();
                        }
                      }}
                    />
                  </Form.Item>
                  <div className="encounter-hp-actions">
                    <Button onClick={applyTempHpInput}>Set</Button>
                  </div>
                </div>
              </div>
              <div className="encounter-hp-control encounter-hp-control--wide">
                <span className="encounter-hp-control__label">HP change</span>
                <div className="encounter-hp-control__row">
                  <Form.Item layout="vertical">
                    <InputNumber
                      aria-label="HP change"
                      value={amount}
                      onChange={(value) =>
                        setAmountDraft({
                          participantKey: activeCurrent.participant_key,
                          value,
                        })
                      }
                      onKeyDown={(event) => {
                        if (event.key === "Enter") {
                          event.preventDefault();
                          applyHpChangeInput();
                        }
                      }}
                    />
                  </Form.Item>
                  <div className="encounter-hp-actions">
                    <Button
                      onClick={() => {
                        if (amount !== null) {
                          updateParticipant(
                            damageChanges(activeCurrent, Math.abs(amount)),
                          );
                          setAmountDraft({
                            participantKey: activeCurrent.participant_key,
                            value: null,
                          });
                        }
                      }}
                    >
                      Damage
                    </Button>
                    <Button
                      onClick={() => {
                        if (amount !== null) {
                          updateParticipant(
                            healChanges(activeCurrent, Math.abs(amount)),
                          );
                          setAmountDraft({
                            participantKey: activeCurrent.participant_key,
                            value: null,
                          });
                        }
                      }}
                    >
                      Heal
                    </Button>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <Form.Item label="Participant note" layout="vertical">
            <Input.TextArea
              key={`note-${activeCurrent.participant_key}-${activeCurrent.note ?? ""}`}
              defaultValue={activeCurrent.note ?? ""}
              onBlur={(event) => {
                const nextNote = event.currentTarget.value;
                if (nextNote !== (activeCurrent.note ?? "")) {
                  updateParticipant({ note: nextNote });
                }
              }}
            />
          </Form.Item>
          <Form.Item label="Side" layout="vertical">
            <Select<EncounterParticipantSideView>
              value={activeCurrent.side}
              onChange={(side) => updateParticipant({ side })}
              options={["pc", "ally", "enemy", "neutral", "hazard"].map((value) => ({
                value: value as EncounterParticipantSideView,
                label: value,
              }))}
            />
          </Form.Item>
          <Button
            onClick={() => updateParticipant({ defeated: !activeCurrent.defeated })}
          >
            {activeCurrent.defeated ? "Mark active" : "Mark defeated"}
          </Button>
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
                        participant_key: activeCurrent.participant_key,
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
                        ...(values.sourceNote
                          ? { source_note: values.sourceNote }
                          : {}),
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
            {activeCurrent.conditions.length === 0 ? (
              <p className="encounter-empty-note">No conditions</p>
            ) : (
              <div className="encounter-condition-list">
                {activeCurrent.conditions.map((condition) => (
                  <ConditionEditor
                    condition={condition}
                    key={condition.condition_id.toString()}
                    participantKey={activeCurrent.participant_key}
                    participants={participants}
                    onRemove={onRemoveCondition}
                    onUpdate={onUpdateCondition}
                  />
                ))}
              </div>
            )}
          </section>
        </div>
      ) : (
        <div className="detail-empty">Select a participant to edit.</div>
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
        <InputNumber
          aria-label={`${condition.name} value`}
          min={0}
          defaultValue={optionalNumber(condition.value)}
          onBlur={(event) =>
            update({
              value:
                event.target.value === ""
                  ? undefined
                  : BigInt(Number(event.target.value)),
            })
          }
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

function participantUpdate(
  participant: EncounterParticipantView,
  changes: Partial<UpdateEncounterParticipantRequest>,
): UpdateEncounterParticipantRequest {
  return {
    participant_key: participant.participant_key,
    display_name: participant.display_name,
    side: participant.side,
    participant_variant: participant.participant_variant,
    initiative: participant.initiative,
    max_hp: participant.max_hp,
    current_hp: participant.current_hp,
    temporary_hp: participant.temporary_hp,
    defeated: participant.defeated,
    hidden: participant.hidden,
    note: participant.note,
    ...changes,
  };
}
function damageChanges(
  participant: EncounterParticipantView,
  amount: number,
): Partial<UpdateEncounterParticipantRequest> {
  const temporaryHp = asNumber(participant.temporary_hp);
  const currentHp = asNumber(participant.current_hp);
  const tempDamage = Math.min(temporaryHp, amount);
  const remaining = amount - tempDamage;
  const current_hp = BigInt(Math.max(0, currentHp - remaining));
  return {
    temporary_hp: BigInt(temporaryHp - tempDamage),
    current_hp,
    defeated: current_hp === BigInt(0) ? true : participant.defeated,
  };
}
function healChanges(
  participant: EncounterParticipantView,
  amount: number,
): Partial<UpdateEncounterParticipantRequest> {
  const currentHp = clampCurrentHp(
    participant,
    asNumber(participant.current_hp) + amount,
  );
  return {
    current_hp: BigInt(currentHp),
  };
}
function clampCurrentHp(participant: EncounterParticipantView, hp: number): number {
  const lowerBounded = Math.max(0, hp);
  if (participant.max_hp === undefined) {
    return lowerBounded;
  }
  return Math.min(lowerBounded, Number(participant.max_hp));
}
function applyParticipantUpdate(
  participant: EncounterParticipantView,
  request: UpdateEncounterParticipantRequest,
): EncounterParticipantView {
  return {
    ...participant,
    display_name: request.display_name,
    side: request.side,
    participant_variant: request.participant_variant,
    initiative: request.initiative,
    max_hp: request.max_hp,
    current_hp: request.current_hp,
    temporary_hp: request.temporary_hp,
    defeated: request.defeated,
    hidden: request.hidden,
    note: request.note,
  };
}
function evaluateHpFormula(value: string): number | null {
  const trimmed = value.trim();
  if (!/^\d+(\s*[+-]\s*\d+)*$/.test(trimmed)) {
    return null;
  }
  const tokens = trimmed.match(/\d+|[+-]/g);
  if (!tokens || tokens.length === 0) {
    return null;
  }
  let result = Number(tokens[0]);
  for (let index = 1; index < tokens.length; index += 2) {
    const operator = tokens[index];
    const next = Number(tokens[index + 1]);
    result = operator === "-" ? result - next : result + next;
  }
  return Math.max(0, result);
}
function asNumber(value: bigint | undefined): number {
  return value === undefined ? 0 : Number(value);
}
function hpLabel(participant: EncounterParticipantView | null): string {
  if (!participant) {
    return "--/--";
  }
  const current = displayNumber(participant.current_hp);
  const max = displayNumber(participant.max_hp);
  const temporaryHp = asNumber(participant.temporary_hp);
  return `${current}/${max}${temporaryHp > 0 ? ` +${temporaryHp}` : ""}`;
}
function displayNumber(value: bigint | undefined): string {
  return value === undefined ? "--" : value.toString();
}
function optionalNumber(value: bigint | undefined): number | undefined {
  return value === undefined ? undefined : Number(value);
}

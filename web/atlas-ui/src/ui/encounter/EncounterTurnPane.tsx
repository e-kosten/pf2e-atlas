import { Button, Form, Input, InputNumber, Select } from "antd";
import { Edit2, Play, Trash2 } from "lucide-react";
import { useState } from "react";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterParticipantConditionView,
  EncounterParticipantSideView,
  EncounterParticipantView,
  EncounterSummaryView,
  UpdateEncounterParticipantConditionRequest,
  UpdateEncounterParticipantRequest,
  UpdateEncounterRequest,
} from "../../generated/atlas";
import { EditEncounterModal } from "./EncounterModals";

type AddConditionForm = {
  name: string;
  value?: number;
  duration?: number;
  sourceParticipantKey?: string;
  note?: string;
  sourceNote?: string;
};

export function EncounterTurnPane({
  current,
  encounter,
  encounterNote,
  participants,
  onDeleteEncounter,
  onStart,
  onUpdateEncounter,
  onAddCondition,
  onRemoveCondition,
  onUpdateCondition,
  onUpdate,
}: {
  current: EncounterParticipantView | null;
  encounter: EncounterSummaryView | null;
  encounterNote: string | undefined;
  participants: EncounterParticipantView[];
  onDeleteEncounter: () => void;
  onStart: () => void;
  onUpdateEncounter: (encounter: UpdateEncounterRequest) => void;
  onAddCondition: (condition: AddEncounterParticipantConditionRequest) => void;
  onRemoveCondition: (participantKey: string, conditionId: bigint) => void;
  onUpdateCondition: (
    participantKey: string,
    condition: UpdateEncounterParticipantConditionRequest,
  ) => void;
  onUpdate: (participant: UpdateEncounterParticipantRequest) => void;
}) {
  const [hpInput, setHpInput] = useState("");
  const [amount, setAmount] = useState<number | null>(null);
  const [editEncounterOpen, setEditEncounterOpen] = useState(false);
  const [conditionForm] = Form.useForm<AddConditionForm>();
  const encounterName = encounter?.name ?? "Encounter";
  const applyHpInput = () => {
    if (!current) {
      return;
    }
    const hp = evaluateHpFormula(hpInput);
    if (hp !== null) {
      onUpdate(
        participantUpdate(current, {
          current_hp: BigInt(hp),
          defeated: hp === 0 ? true : current.defeated,
        }),
      );
      setHpInput("");
    }
  };

  return (
    <section className="encounter-pane encounter-turn">
      <header className="encounter-pane__header">
        <div>
          <h2>{encounterName}</h2>
          <p>{current ? `Turn: ${current.display_name}` : "Not started"}</p>
        </div>
        <div className="encounter-actions">
          <Button
            aria-label="Edit encounter"
            icon={<Edit2 size={14} />}
            onClick={() => setEditEncounterOpen(true)}
          />
          <Button icon={<Play size={16} />} onClick={onStart} type="primary">
            {current ? "Next" : "Play"}
          </Button>
        </div>
      </header>
      {current ? (
        <div key={current.participant_key} className="encounter-turn__body">
          <div className="encounter-form-grid">
            <Form.Item label="Name" layout="vertical">
              <Input
                defaultValue={current.display_name}
                onBlur={(event) =>
                  onUpdate(
                    participantUpdate(current, { display_name: event.target.value }),
                  )
                }
              />
            </Form.Item>
            <Form.Item label="Initiative" layout="vertical">
              <InputNumber
                defaultValue={optionalNumber(current.initiative)}
                onBlur={(event) =>
                  onUpdate(
                    participantUpdate(current, {
                      initiative:
                        event.target.value === ""
                          ? undefined
                          : BigInt(Number(event.target.value)),
                    }),
                  )
                }
              />
            </Form.Item>
            <Form.Item label="Temp HP" layout="vertical">
              <InputNumber
                min={0}
                defaultValue={optionalNumber(current.temporary_hp)}
                onBlur={(event) =>
                  onUpdate(
                    participantUpdate(current, {
                      temporary_hp: BigInt(
                        Math.max(0, Number(event.target.value || 0)),
                      ),
                    }),
                  )
                }
              />
            </Form.Item>
          </div>
          <div className="encounter-control-row">
            <Form.Item label="HP or formula" layout="vertical">
              <Input
                aria-label="HP or formula"
                value={hpInput}
                onChange={(event) => setHpInput(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    applyHpInput();
                  }
                }}
              />
            </Form.Item>
            <Button onClick={applyHpInput}>Set</Button>
          </div>
          <div className="encounter-control-row">
            <Form.Item label="Amount" layout="vertical">
              <InputNumber
                aria-label="Amount"
                min={0}
                value={amount}
                onChange={(value) => setAmount(value)}
              />
            </Form.Item>
            <Button
              onClick={() => {
                if (amount !== null) {
                  onUpdate(applyDamage(current, amount));
                  setAmount(null);
                }
              }}
            >
              Damage
            </Button>
            <Button
              onClick={() => {
                if (amount !== null) {
                  onUpdate(
                    participantUpdate(current, {
                      current_hp: BigInt(
                        Math.max(0, asNumber(current.current_hp) + amount),
                      ),
                    }),
                  );
                  setAmount(null);
                }
              }}
            >
              Heal
            </Button>
          </div>
          <Form.Item label="Participant note" layout="vertical">
            <Input.TextArea
              key={`note-${current.participant_key}-${current.note ?? ""}`}
              defaultValue={current.note ?? ""}
              onBlur={(event) => {
                const nextNote = event.currentTarget.value;
                if (nextNote !== (current.note ?? "")) {
                  onUpdate(participantUpdate(current, { note: nextNote }));
                }
              }}
            />
          </Form.Item>
          <Form.Item label="Side" layout="vertical">
            <Select<EncounterParticipantSideView>
              value={current.side}
              onChange={(side) => onUpdate(participantUpdate(current, { side }))}
              options={["pc", "ally", "enemy", "neutral", "hazard"].map((value) => ({
                value: value as EncounterParticipantSideView,
                label: value,
              }))}
            />
          </Form.Item>
          <Button
            onClick={() =>
              onUpdate(participantUpdate(current, { defeated: !current.defeated }))
            }
          >
            {current.defeated ? "Mark active" : "Mark defeated"}
          </Button>
          <section className="encounter-conditions">
            <h3>Conditions</h3>
            {current.conditions.length === 0 ? (
              <div className="detail-empty">No conditions</div>
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
            <Form
              form={conditionForm}
              layout="vertical"
              onFinish={(values) => {
                onAddCondition({
                  participant_key: current.participant_key,
                  name: values.name,
                  ...(values.value === undefined
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
              }}
            >
              <Form.Item name="name" label="Condition" rules={[{ required: true }]}>
                <Input />
              </Form.Item>
              <div className="encounter-control-row">
                <Form.Item name="value" label="Value">
                  <InputNumber min={0} />
                </Form.Item>
                <Form.Item name="duration" label="Rounds">
                  <InputNumber min={0} />
                </Form.Item>
              </div>
              <Form.Item name="note" label="Note">
                <Input />
              </Form.Item>
              <Form.Item name="sourceParticipantKey" label="Source">
                <Select
                  allowClear
                  options={participants.map((participant) => ({
                    value: participant.participant_key,
                    label: participant.display_name,
                  }))}
                />
              </Form.Item>
              <Form.Item name="sourceNote" label="Source Note">
                <Input />
              </Form.Item>
              <Button onClick={() => conditionForm.submit()}>Add Condition</Button>
            </Form>
          </section>
        </div>
      ) : (
        <div className="detail-empty">Press play after setting initiative.</div>
      )}
      {encounter && (
        <div className="encounter-actions">
          <Button
            onClick={() =>
              onUpdateEncounter({
                ...encounterUpdate(encounter),
                note: encounterNote,
                status: "complete",
              })
            }
          >
            Complete
          </Button>
          <Button
            onClick={() =>
              onUpdateEncounter({
                ...encounterUpdate(encounter),
                note: encounterNote,
                status: "archived",
              })
            }
          >
            Archive
          </Button>
        </div>
      )}
      <Button danger onClick={onDeleteEncounter}>
        Delete encounter
      </Button>
      {encounter && (
        <EditEncounterModal
          encounter={encounter}
          encounterNote={encounterNote}
          open={editEncounterOpen}
          onCancel={() => setEditEncounterOpen(false)}
          onSave={(request) => {
            onUpdateEncounter(request);
            setEditEncounterOpen(false);
          }}
        />
      )}
    </section>
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
  return (
    <div className="encounter-condition-row">
      <Form.Item label="Condition" layout="vertical">
        <Input
          defaultValue={condition.name}
          onBlur={(event) => update({ name: event.target.value })}
        />
      </Form.Item>
      <Form.Item label="Value" layout="vertical">
        <InputNumber
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
      </Form.Item>
      <Form.Item label="Rounds" layout="vertical">
        <InputNumber
          min={0}
          defaultValue={optionalNumber(condition.duration_rounds)}
          onBlur={(event) =>
            update({
              duration_rounds:
                event.target.value === ""
                  ? undefined
                  : BigInt(Number(event.target.value)),
            })
          }
        />
      </Form.Item>
      <Form.Item label="Source" layout="vertical">
        <Select
          allowClear
          defaultValue={condition.source_participant_key}
          onChange={(value) => update({ source_participant_key: value })}
          options={participants.map((participant) => ({
            value: participant.participant_key,
            label: participant.display_name,
          }))}
        />
      </Form.Item>
      <Form.Item label="Note" layout="vertical">
        <Input
          defaultValue={condition.note}
          onBlur={(event) => update({ note: event.target.value || undefined })}
        />
      </Form.Item>
      <Form.Item label="Source Note" layout="vertical">
        <Input
          defaultValue={condition.source_note}
          onBlur={(event) => update({ source_note: event.target.value || undefined })}
        />
      </Form.Item>
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

function encounterUpdate(encounter: EncounterSummaryView): UpdateEncounterRequest {
  return {
    encounter_key: encounter.encounter_key,
    slug: encounter.slug,
    name: encounter.name,
    description: encounter.description,
    status: encounter.status,
  };
}
function participantUpdate(
  participant: EncounterParticipantView,
  changes: Partial<UpdateEncounterParticipantRequest>,
): UpdateEncounterParticipantRequest {
  return {
    participant_key: participant.participant_key,
    display_name: participant.display_name,
    side: participant.side,
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
function applyDamage(
  participant: EncounterParticipantView,
  amount: number,
): UpdateEncounterParticipantRequest {
  const temporaryHp = asNumber(participant.temporary_hp);
  const currentHp = asNumber(participant.current_hp);
  const tempDamage = Math.min(temporaryHp, amount);
  const remaining = amount - tempDamage;
  const current_hp = BigInt(Math.max(0, currentHp - remaining));
  return participantUpdate(participant, {
    temporary_hp: BigInt(temporaryHp - tempDamage),
    current_hp,
    defeated: current_hp === BigInt(0) ? true : participant.defeated,
  });
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
function optionalNumber(value: bigint | undefined): number | undefined {
  return value === undefined ? undefined : Number(value);
}

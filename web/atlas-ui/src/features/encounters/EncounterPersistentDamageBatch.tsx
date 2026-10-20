import {
  Alert,
  Button,
  Form,
  InputNumber,
  Segmented,
  Select,
  Space,
  type FormInstance,
} from "antd";
import { useState } from "react";
import type {
  ApplyPersistentDamageRequest,
  EncounterParticipantView,
  PersistentDamageGroupView,
  PersistentDamageResultView,
} from "../../generated/atlas";
import { RollResult } from "../../shared/dice/RollResult";
import { integerRule } from "./EncounterConditionEditor";
type Values = {
  groups: Record<
    string,
    { source?: string; mode: "roll" | "table"; amount?: number | null }
  >;
};
function DamageFields({
  group,
  form,
  label,
}: {
  group: PersistentDamageGroupView;
  form: FormInstance<Values>;
  label: string;
}) {
  const mode = Form.useWatch(["groups", group.episode_id, "mode"], form) ?? "roll";
  const selected = group.sources.find(
    (source) => source.condition_id === group.selected_source_id,
  );
  return (
    <section>
      <h4>
        {label}
        {selected ? ` · ${selected.formula}` : ""}
      </h4>
      {group.selected_source_id === null && (
        <Form.Item
          name={["groups", group.episode_id, "source"]}
          label="Damage source"
          rules={[
            { required: true, message: "Choose the source with the greater damage." },
          ]}
        >
          <Select
            aria-label={`${label} damage source`}
            options={group.sources.map((source) => ({
              value: source.condition_id.toString(),
              label: `${source.formula}${source.multiplier === "normal" ? "" : ` · ${source.multiplier}`}`,
            }))}
          />
        </Form.Item>
      )}
      <Form.Item name={["groups", group.episode_id, "mode"]}>
        <Segmented
          aria-label={`${label} damage result`}
          options={[
            { value: "roll", label: "Roll in Atlas" },
            { value: "table", label: "Table result" },
          ]}
        />
      </Form.Item>
      {mode === "table" && (
        <Form.Item
          name={["groups", group.episode_id, "amount"]}
          label="Damage after defenses"
          rules={[integerRule(true)]}
        >
          <InputNumber aria-label={`${label} damage amount`} step={1} />
        </Form.Item>
      )}
    </section>
  );
}
export function EncounterPersistentDamageBatch({
  participant,
  encounterRevision,
  boundaryKey = null,
  generation,
  damageApplied,
  damageTypes,
  disabled,
  pending,
  onApply,
  savedResult,
}: {
  participant: EncounterParticipantView;
  encounterRevision: bigint;
  boundaryKey?: string | null;
  generation: string | null;
  damageApplied: boolean;
  damageTypes: Array<{ value: string; label: string }>;
  disabled: boolean;
  pending: boolean;
  onApply: (
    request: Omit<ApplyPersistentDamageRequest, "request_id">,
  ) => Promise<PersistentDamageResultView>;
  savedResult?: PersistentDamageResultView | null;
}) {
  const [form] = Form.useForm<Values>();
  const [revision, setRevision] = useState(encounterRevision);
  const [result, setResult] = useState<PersistentDamageResultView | null>(null);
  const [error, setError] = useState<string>();
  const committed = result ?? savedResult;
  const changed = revision !== encounterRevision;
  if (damageApplied || committed)
    return (
      <>
        {committed && (
          <p>
            Total: {committed.total.toString()} damage applied
            {committed.hardness_absorbed > 0n
              ? ` · Hardness absorbed ${committed.hardness_absorbed}`
              : ""}
          </p>
        )}
        {committed ? (
          committed.groups.map((group) => (
            <p key={group.episode_id}>
              {damageTypes.find((type) => type.value === group.damage_type)?.label ??
                group.damage_type}
              : {group.amount.toString()}
              {group.roll && <RollResult outcome={group.roll} />}
            </p>
          ))
        ) : (
          <p>Persistent damage applied</p>
        )}
      </>
    );
  return (
    <>
      {error && (
        <Alert type="error" title="Persistent damage failed" description={error} />
      )}
      {changed && (
        <Alert
          type="warning"
          title="Encounter changed"
          action={
            <Button
              onClick={() => {
                form.resetFields();
                setRevision(encounterRevision);
              }}
            >
              Reload damage
            </Button>
          }
        />
      )}
      <Form
        form={form}
        layout="vertical"
        disabled={disabled || pending || changed}
        initialValues={{
          groups: Object.fromEntries(
            participant.persistent_damage_groups.map((group) => [
              group.episode_id,
              { mode: "roll" },
            ]),
          ),
        }}
        onFinish={async (values) => {
          if (disabled || pending || changed) return;
          try {
            setError(undefined);
            const receipt = await onApply({
              boundary_key: boundaryKey,
              expected_encounter_revision: revision,
              participant_key: participant.participant_key,
              artifact_generation: generation,
              groups: participant.persistent_damage_groups.map((group) => {
                const draft = values.groups[group.episode_id];
                return {
                  episode_id: group.episode_id,
                  selected_source_id: draft.source
                    ? BigInt(draft.source)
                    : group.selected_source_id,
                  damage:
                    draft.mode === "table"
                      ? { kind: "table_result", amount: BigInt(draft.amount!) }
                      : { kind: "roll" },
                };
              }),
              context: {
                lethality: null,
                critical: null,
                enemy_source: null,
                responsible_participant_key: null,
                base_death_threshold: null,
                keep_current_initiative: null,
                attack_or_spell_damage: null,
                natural_rest_healing: null,
                keep_upright: null,
              },
            });
            setResult(receipt);
          } catch (cause) {
            setError(
              cause instanceof Error
                ? cause.message
                : "Could not apply persistent damage.",
            );
          }
        }}
      >
        {participant.persistent_damage_groups.map((group) => (
          <DamageFields
            key={group.episode_id}
            group={group}
            form={form}
            label={
              damageTypes.find((type) => type.value === group.damage_type)?.label ??
              group.damage_type
            }
          />
        ))}
        <Space wrap>
          <Button
            type="primary"
            htmlType="submit"
            disabled={
              disabled ||
              pending ||
              changed ||
              !participant.persistent_damage_groups.length
            }
            loading={pending}
          >
            Apply persistent damage
          </Button>
        </Space>
      </Form>
    </>
  );
}

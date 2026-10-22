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
import { useState, type ReactNode } from "react";
import type {
  ApplyPersistentDamageRequest,
  EncounterParticipantView,
  PersistentDamageGroupView,
  PersistentDamageResultView,
  PersistentDamageResolutionView,
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
  heading = true,
  tableReason,
}: {
  group: PersistentDamageGroupView;
  form: FormInstance<Values>;
  label: string;
  heading?: boolean;
  tableReason?: string;
}) {
  const draftMode = Form.useWatch(["groups", group.episode_id, "mode"], form) ?? "roll";
  const mode = tableReason ? "table" : draftMode;
  const selected = group.sources.find(
    (source) => source.condition_id === group.selected_source_id,
  );
  return (
    <section>
      {heading && (
        <h4>
          {label}
          {selected ? ` · ${selected.formula}` : ""}
        </h4>
      )}
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
      <Form.Item
        name={["groups", group.episode_id, "mode"]}
        getValueProps={(value) => ({ value: tableReason ? "table" : value })}
      >
        <Segmented
          aria-label={`${label} damage result`}
          options={[
            { value: "roll", label: "Roll in Atlas", disabled: !!tableReason },
            { value: "table", label: "Table result" },
          ]}
        />
      </Form.Item>
      {mode === "table" && (
        <Form.Item
          name={["groups", group.episode_id, "amount"]}
          label="Damage taken"
          tooltip="After IWR"
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
  displayGroups = participant.persistent_damage_groups,
  renderGroup,
  resolutions = {},
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
  displayGroups?: PersistentDamageGroupView[];
  resolutions?: Record<string, PersistentDamageResolutionView>;
  renderGroup?: (group: PersistentDamageGroupView, damage: ReactNode) => ReactNode;
}) {
  const [form] = Form.useForm<Values>();
  const membership = participant.persistent_damage_groups
    .map(
      (group) =>
        `${group.episode_id}:${group.sources.map((source) => source.condition_id).join(",")}`,
    )
    .join(";");
  const [anchor, setAnchor] = useState({ membership, revision: encounterRevision });
  if (anchor.membership !== membership)
    setAnchor({ membership, revision: encounterRevision });
  const revision =
    anchor.membership === membership ? anchor.revision : encounterRevision;
  const [result, setResult] = useState<PersistentDamageResultView | null>(null);
  const [error, setError] = useState<string>();
  const committed = result ?? savedResult;
  const changed = revision !== encounterRevision;
  const applied = damageApplied || !!committed;
  return (
    <>
      {error && (
        <Alert type="error" title="Persistent damage failed" description={error} />
      )}
      {changed && !applied && (
        <Alert
          type="warning"
          title="Encounter changed"
          action={
            <Button
              onClick={() => {
                form.resetFields();
                setAnchor({ membership, revision: encounterRevision });
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
        disabled={disabled || pending || (changed && !applied)}
        initialValues={{
          groups: Object.fromEntries(
            participant.persistent_damage_groups.map((group) => [
              group.episode_id,
              {
                mode:
                  resolutions[group.episode_id]?.kind === "table" ? "table" : "roll",
              },
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
                    draft.mode === "table" ||
                    resolutions[group.episode_id]?.kind === "table"
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
        <div className="encounter-persistent-groups">
          {displayGroups.map((group) => {
            const label =
              damageTypes.find((type) => type.value === group.damage_type)?.label ??
              group.damage_type;
            const active = participant.persistent_damage_groups.find(
              (candidate) => candidate.episode_id === group.episode_id,
            );
            const result = committed?.groups.find(
              (result) => result.episode_id === group.episode_id,
            );
            const resolution = resolutions[group.episode_id];
            const protection = result
              ? result.protection
              : active && resolution?.kind === "known"
                ? resolution.protection
                : undefined;
            const protectionNote = protection?.immune
              ? `Immune to ${label.toLowerCase()}`
              : protection
                ? [
                    protection.weakness > 0n && `Weakness ${protection.weakness}`,
                    protection.resistance > 0n && `Resistance ${protection.resistance}`,
                  ]
                    .filter(Boolean)
                    .join(" · ")
                : undefined;
            const damage = (
              <>
                {protectionNote && (
                  <p className="encounter-condition-meta">{protectionNote}</p>
                )}
                {!applied && active && resolution?.kind === "table" && (
                  <p className="encounter-condition-meta">{resolution.reason}</p>
                )}
                {result ? (
                  <div className="encounter-persistent-applied">
                    <span>
                      {renderGroup
                        ? `${result.amount} damage`
                        : `${label}: ${result.amount}`}
                    </span>
                    {result.roll && <RollResult outcome={result.roll} />}
                  </div>
                ) : !applied && active ? (
                  <DamageFields
                    group={active}
                    form={form}
                    label={label}
                    heading={!renderGroup}
                    tableReason={
                      resolution?.kind === "table" ? resolution.reason : undefined
                    }
                  />
                ) : null}
              </>
            );
            return (
              <div key={group.episode_id}>
                {renderGroup ? renderGroup(group, damage) : damage}
              </div>
            );
          })}
        </div>
        {committed && (
          <p className="encounter-persistent-total">
            Total: {committed.total.toString()} damage applied
            {committed.hardness_absorbed > 0n
              ? ` · Hardness absorbed ${committed.hardness_absorbed}`
              : ""}
          </p>
        )}
        {applied && !committed && <p>Persistent damage applied</p>}
        {!applied && participant.persistent_damage_groups.length > 0 && (
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
        )}
      </Form>
    </>
  );
}

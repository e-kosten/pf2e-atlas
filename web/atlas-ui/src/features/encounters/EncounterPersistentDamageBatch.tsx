import { Alert, Button, Form, InputNumber, Segmented, Space, Tooltip } from "antd";
import { useState, type ReactNode } from "react";
import type {
  ApplyPersistentDamageRequest,
  EncounterParticipantView,
  PersistentDamageGroupView,
  PersistentDamageResultView,
  PersistentDamageResolutionView,
} from "../../generated/atlas";
import { RollResult } from "../../shared/dice/RollResult";
import { nonNegativeIntegerRule } from "../../shared/ui/forms/nonNegativeIntegerRule";
type Values = {
  mode: "roll" | "table";
  groups: Record<string, { amount?: number | null }>;
};
function DamageFields({
  group,
  mode,
  label,
  heading = true,
}: {
  group: PersistentDamageGroupView;
  mode: Values["mode"];
  label: string;
  heading?: boolean;
}) {
  const selected = group.sources.find(
    (source) => source.condition_id === group.selected_source_id,
  );
  if (!heading && mode !== "table") return null;
  return (
    <section className="encounter-persistent-damage-fields">
      {heading && (
        <h4>
          {label}
          {selected ? ` · ${selected.formula}` : ""}
        </h4>
      )}
      {mode === "table" && (
        <Form.Item
          style={{ marginBottom: 0 }}
          name={["groups", group.episode_id, "amount"]}
          label="Damage taken"
          tooltip="After IWR"
          rules={[nonNegativeIntegerRule(true)]}
        >
          <InputNumber
            aria-label={`${label} damage amount`}
            min={0}
            max={1_000_000_000}
            step={1}
          />
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
  const draftMode = Form.useWatch("mode", form) ?? "roll";
  const amounts = Form.useWatch("groups", { form, preserve: true }) as
    | Values["groups"]
    | undefined;
  const requiresTable = participant.persistent_damage_groups.some(
    (group) => resolutions[group.episode_id]?.kind === "table",
  );
  const mode: Values["mode"] = requiresTable ? "table" : draftMode;
  const entered = participant.persistent_damage_groups.map(
    (group) => amounts?.[group.episode_id]?.amount,
  );
  const total = entered.every(
    (amount) =>
      typeof amount === "number" &&
      Number.isSafeInteger(amount) &&
      amount >= 0 &&
      amount <= 1_000_000_000,
  )
    ? entered.reduce<number>((sum, amount) => sum + amount!, 0)
    : undefined;
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
        initialValues={{ mode: "roll" }}
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
                return {
                  episode_id: group.episode_id,

                  damage:
                    mode === "table"
                      ? {
                          kind: "table_result",
                          amount: BigInt(values.groups[group.episode_id].amount!),
                        }
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
        {!applied && participant.persistent_damage_groups.length > 0 && (
          <Form.Item
            name="mode"
            label="Damage"
            style={{ marginBottom: "var(--space-group)" }}
            getValueProps={(value) => ({ value: requiresTable ? "table" : value })}
          >
            <Segmented
              aria-label="Persistent damage resolution"
              options={[
                { value: "roll", label: "Roll in Atlas", disabled: requiresTable },
                { value: "table", label: "Table results" },
              ]}
            />
          </Form.Item>
        )}
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
                    mode={mode}
                    label={label}
                    heading={!renderGroup}
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
            <Tooltip
              title="All persistent damage types resolve together. Effects triggered by taking damage, such as increasing Dying, happen once."
              trigger={["hover", "focus"]}
            >
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
                {mode === "roll"
                  ? "Roll & apply damage"
                  : total === undefined
                    ? "Apply damage"
                    : `Apply ${total} damage`}
              </Button>
            </Tooltip>
          </Space>
        )}
      </Form>
    </>
  );
}

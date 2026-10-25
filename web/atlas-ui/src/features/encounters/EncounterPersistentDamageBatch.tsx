import { Alert, Button, Form, InputNumber, Segmented, Space, Tooltip } from "antd";
import { Undo2, X } from "lucide-react";
import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import type {
  ApplyPersistentDamageRequest,
  EncounterParticipantView,
  PersistentDamageGroupInput,
  PersistentDamageGroupView,
  PersistentDamageResultView,
  PersistentDamageResolutionView,
  PersistentEpisodeReviewView,
  PersistentRecoveryResultView,
  PersistentRecoveryInput,
  PreparePersistentDamageRequest,
  PreparedPersistentDamageGroupView,
  PreparedPersistentDamageView,
  RecoverPersistentDamageRequest,
} from "../../generated/atlas";
import { prepareEncounterPersistentDamage } from "../../api/atlasApi";
import { RollResult, UnrolledRoll } from "../../shared/dice/RollResult";
import { useDiceRoll } from "../../shared/dice/useDiceRoll";
import { nonNegativeIntegerRule } from "../../shared/ui/forms/nonNegativeIntegerRule";
import { EncounterPersistentRecovery } from "./EncounterPersistentRecovery";

type Values = {
  mode: "roll" | "table";
  groups: Record<string, { amount?: number | null }>;
};
const validAmount = (value: unknown): value is number =>
  typeof value === "number" &&
  Number.isSafeInteger(value) &&
  value >= 0 &&
  value <= 1_000_000_000;
export function EncounterPersistentDamageBatch({
  encounterRef,
  participant,
  encounterRevision,
  boundaryKey = null,
  generation,
  damageApplied,
  damageTypes,
  disabled,
  pending,
  onApply,
  onRecover,
  savedResult,
  preparedDamage = [],
  preparedRecovery = [],
  recoveryResults = [],
  episodes = [],
  displayGroups = participant.persistent_damage_groups,
  renderGroup,
  resolutions = {},
}: {
  encounterRef: string;
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
  onRecover: (
    request: Omit<RecoverPersistentDamageRequest, "request_id">,
  ) => Promise<PersistentRecoveryResultView>;
  savedResult?: PersistentDamageResultView | null;
  preparedDamage?: PreparedPersistentDamageGroupView[];
  preparedRecovery?: PersistentRecoveryResultView[];
  recoveryResults?: PersistentRecoveryResultView[];
  episodes?: PersistentEpisodeReviewView[];
  displayGroups?: PersistentDamageGroupView[];
  resolutions?: Record<string, PersistentDamageResolutionView>;
  renderGroup?: (
    group: PersistentDamageGroupView,
    damage: ReactNode,
    recovery: ReactNode,
  ) => ReactNode;
}) {
  const [form] = Form.useForm<Values>();
  const draftMode = Form.useWatch("mode", form) ?? "roll";
  const amounts = Form.useWatch("groups", { form, preserve: true }) as
    | Values["groups"]
    | undefined;
  const requiresTable = participant.persistent_damage_groups.some(
    (g) => resolutions[g.episode_id]?.kind === "table",
  );
  const mode: Values["mode"] = requiresTable ? "table" : draftMode;
  const [rolls, setRolls] = useState<Record<string, PreparedPersistentDamageGroupView>>(
    () => Object.fromEntries(preparedDamage.map((r) => [r.group.episode_id, r])),
  );
  const [recoveries, setRecoveries] = useState<
    Record<string, PersistentRecoveryResultView>
  >(() => Object.fromEntries(preparedRecovery.map((r) => [r.episode_id, r])));
  const [clears, setClears] = useState<Record<string, boolean>>({});
  const [busy, setBusy] = useState<string>();
  const damageRoll = useDiceRoll<PreparedPersistentDamageView>();
  const [rollingEpisode, setRollingEpisode] = useState<string>();
  const [error, setError] = useState<string>();
  const requests = useRef(new Map<string, PreparePersistentDamageRequest>());
  const membership = participant.persistent_damage_groups
    .map((g) => `${g.episode_id}:${g.sources.map((s) => s.condition_id).join(",")}`)
    .join(";");
  const [anchor, setAnchor] = useState({ membership, revision: encounterRevision });
  const changed =
    anchor.membership !== membership || anchor.revision !== encounterRevision;
  const [result, setResult] = useState<PersistentDamageResultView | null>(null);
  const committed = result ?? savedResult;
  const applied = damageApplied || !!committed;
  const groups = useMemo<PersistentDamageGroupInput[]>(
    () =>
      participant.persistent_damage_groups.flatMap<PersistentDamageGroupInput>(
        (group) => {
          if (mode === "table") {
            const amount = amounts?.[group.episode_id]?.amount;
            return validAmount(amount)
              ? [
                  {
                    episode_id: group.episode_id,
                    damage: { kind: "table_result" as const, amount: BigInt(amount) },
                  },
                ]
              : [];
          }
          const prepared = rolls[group.episode_id];
          return prepared
            ? [
                {
                  episode_id: group.episode_id,
                  damage: {
                    kind: "prepared" as const,
                    request_id: prepared.request_id,
                  },
                },
              ]
            : [];
        },
      ),
    [participant.persistent_damage_groups, mode, amounts, rolls],
  );
  const [previewAttempt, setPreviewAttempt] = useState(0);
  const [preview, setPreview] = useState<{
    inputs: PersistentDamageGroupInput[];
    value?: PreparedPersistentDamageView;
    error?: string;
  }>();
  useEffect(() => {
    if (applied || changed || disabled || pending) return;
    let active = true;
    const timer = setTimeout(() => {
      void prepareEncounterPersistentDamage(encounterRef, {
        request_id: crypto.randomUUID(),
        expected_encounter_revision: anchor.revision,
        participant_key: participant.participant_key,
        boundary_key: boundaryKey,
        artifact_generation: generation,
        groups,
        recovery: null,
      }).then(
        (value) => {
          if (active) setPreview({ inputs: groups, value });
        },
        (cause) => {
          if (active)
            setPreview({
              inputs: groups,
              error:
                cause instanceof Error ? cause.message : "Could not calculate damage.",
            });
        },
      );
    }, 180);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [
    groups,
    applied,
    changed,
    disabled,
    pending,
    encounterRef,
    anchor.revision,
    participant.participant_key,
    boundaryKey,
    generation,
    previewAttempt,
  ]);
  const current = preview?.inputs === groups ? preview : undefined;
  const clearingTypes = participant.persistent_damage_groups
    .filter(
      (group) => clears[group.episode_id] || recoveries[group.episode_id]?.success,
    )
    .map(
      (group) =>
        damageTypes.find((type) => type.value === group.damage_type)?.label ??
        group.damage_type,
    );
  const ready =
    !applied && !changed && !disabled && !pending && !busy && !damageRoll.busy;
  const prepare = async (
    slot: string,
    inputs: PersistentDamageGroupInput[],
    recovery: PreparePersistentDamageRequest["recovery"] = null,
  ) => {
    if (!ready)
      throw new Error("Wait for the current update before preparing another result.");
    let request = requests.current.get(slot);
    if (!request) {
      request = {
        request_id: crypto.randomUUID(),
        expected_encounter_revision: anchor.revision,
        participant_key: participant.participant_key,
        boundary_key: boundaryKey,
        artifact_generation: generation,
        groups: inputs,
        recovery,
      };
      requests.current.set(slot, request);
    }
    setBusy(slot);
    setError(undefined);
    try {
      const completed = await prepareEncounterPersistentDamage(encounterRef, request);
      requests.current.delete(slot);
      return completed;
    } finally {
      setBusy(undefined);
    }
  };
  return (
    <>
      {(error || damageRoll.error) && (
        <Alert type="error" title={error ?? damageRoll.error} />
      )}
      {changed && !applied && (
        <Alert
          type="warning"
          title="Encounter changed"
          description="Review the current damage and recovery before applying."
          action={
            <Button
              onClick={() => {
                form.resetFields();
                setRolls(
                  Object.fromEntries(
                    preparedDamage.map((r) => [r.group.episode_id, r]),
                  ),
                );
                setRecoveries(
                  Object.fromEntries(preparedRecovery.map((r) => [r.episode_id, r])),
                );
                setClears({});
                requests.current.clear();
                setAnchor({ membership, revision: encounterRevision });
                setError(undefined);
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
        disabled={
          disabled || pending || (changed && !applied) || !!busy || damageRoll.busy
        }
        initialValues={{ mode: "roll" }}
        onFinish={async () => {
          if (!ready || !current?.value?.complete) return;
          setError(undefined);
          try {
            const receipt = await onApply({
              boundary_key: boundaryKey,
              expected_encounter_revision: anchor.revision,
              participant_key: participant.participant_key,
              artifact_generation: generation,
              groups,
              recoveries:
                participant.persistent_damage_groups.flatMap<PersistentRecoveryInput>(
                  (g) =>
                    clears[g.episode_id]
                      ? [{ kind: "clear" as const, episode_id: g.episode_id }]
                      : recoveries[g.episode_id]
                        ? [
                            {
                              kind: "prepared" as const,
                              episode_id: g.episode_id,
                              request_id: recoveries[g.episode_id].request_id,
                            },
                          ]
                        : [],
                ),
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
        {!applied && (
          <Form.Item
            name="mode"
            style={{ marginBottom: "var(--space-control)" }}
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
              damageTypes.find((t) => t.value === group.damage_type)?.label ??
              group.damage_type;
            const active = participant.persistent_damage_groups.some(
              (g) => g.episode_id === group.episode_id,
            );
            const saved = committed?.groups.find(
              (g) => g.episode_id === group.episode_id,
            );
            const currentOutcome = current?.value?.damage.groups.find(
              (outcome) => outcome.episode_id === group.episode_id,
            );
            const rollPending = rollingEpisode === group.episode_id && damageRoll.busy;
            const rolled =
              (mode === "roll" ? currentOutcome : undefined) ??
              rolls[group.episode_id]?.group ??
              (rollingEpisode === group.episode_id
                ? damageRoll.result?.damage.groups.find(
                    (g) => g.episode_id === group.episode_id,
                  )
                : undefined);
            const selectedSource = group.sources.find(
              (source) => source.condition_id === group.selected_source_id,
            );
            const resolution = resolutions[group.episode_id];
            const protection = saved
              ? saved.protection
              : (currentOutcome?.protection ??
                (resolution?.kind === "known" ? resolution.protection : undefined) ??
                rolled?.protection);
            const note = protection?.immune
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
                {note && <p className="encounter-condition-meta">{note}</p>}
                {!saved && resolution?.kind === "table" && (
                  <p className="encounter-condition-meta">{resolution.reason}</p>
                )}
                {saved ? (
                  <div className="encounter-persistent-applied">
                    {saved.roll && <RollResult outcome={saved.roll} />}
                    <strong>Damage: {saved.amount.toString()} applied</strong>
                  </div>
                ) : !applied && active ? (
                  <div className="encounter-persistent-damage-control">
                    <span className="encounter-persistent-damage-control__label">
                      Damage
                    </span>
                    <div className="encounter-persistent-damage-control__value">
                      {mode === "table" && selectedSource?.roll_preview && (
                        <span
                          className="encounter-persistent-damage-control__sizer"
                          aria-hidden="true"
                          inert
                        >
                          <UnrolledRoll
                            expression={selectedSource.roll_preview}
                            formula={selectedSource.formula}
                          >
                            <span />
                          </UnrolledRoll>
                        </span>
                      )}
                      <div className="encounter-persistent-damage-control__content">
                        {mode === "table" ? (
                          <Form.Item
                            name={["groups", group.episode_id, "amount"]}
                            style={{ marginBottom: 0 }}
                            rules={[nonNegativeIntegerRule(true)]}
                          >
                            <InputNumber
                              aria-label={`${label} damage amount`}
                              min={0}
                              max={1_000_000_000}
                              step={1}
                            />
                          </Form.Item>
                        ) : rolled ? (
                          <>
                            {rolled.roll && (
                              <RollResult outcome={rolled.roll} pending={rollPending} />
                            )}
                            {!rollPending &&
                              (!rolled.roll ||
                                BigInt(rolled.roll.total) !== rolled.amount) && (
                                <strong>{rolled.amount.toString()} damage</strong>
                              )}
                          </>
                        ) : (
                          <UnrolledDamage
                            expression={selectedSource?.roll_preview}
                            formula={selectedSource?.formula ?? label}
                            pending={rollPending}
                          >
                            <Button
                              loading={rollPending}
                              aria-label={`Roll ${label} damage`}
                              onClick={async () => {
                                let completed: PreparedPersistentDamageView | undefined;
                                await damageRoll.run(async () => {
                                  setRollingEpisode(group.episode_id);
                                  completed = await prepare(
                                    `damage:${group.episode_id}`,
                                    [
                                      ...groups,
                                      {
                                        episode_id: group.episode_id,
                                        damage: { kind: "roll" },
                                      },
                                    ],
                                  );
                                  return completed;
                                });
                                if (!completed) return;
                                const damage = completed.damage;
                                const outcome = damage.groups.find(
                                  (g) => g.episode_id === group.episode_id,
                                );
                                if (outcome)
                                  setRolls((previous) => ({
                                    ...previous,
                                    [group.episode_id]: {
                                      request_id: damage.request_id,
                                      group: outcome,
                                    },
                                  }));
                              }}
                            >
                              Roll
                            </Button>
                          </UnrolledDamage>
                        )}
                      </div>
                    </div>
                  </div>
                ) : null}
              </>
            );
            const episode = episodes.find((e) => e.episode_id === group.episode_id);
            const staged = recoveries[group.episode_id];
            const pendingClear = !!clears[group.episode_id] || !!staged?.success;
            const hasDamage = groups.some((g) => g.episode_id === group.episode_id);
            const recovery = (
              <div
                className={`encounter-persistent-recovery-row${applied ? " encounter-persistent-recovery-row--resolved" : ""}`}
                role="group"
                aria-label={`${label} recovery`}
              >
                {active && applied && (
                  <span className="encounter-persistent-recovery-status">
                    Still active
                  </span>
                )}
                {!active && (
                  <span role="status" className="encounter-persistent-recovery-status">
                    {recoveryResults.some(
                      (r) => r.episode_id === group.episode_id && r.success,
                    )
                      ? "Effect cleared"
                      : "Effect ended"}
                  </span>
                )}
                <Space size="small" align="center" wrap>
                  {(active ||
                    recoveryResults.some(
                      (r) => r.episode_id === group.episode_id && r.dc != null,
                    )) && (
                    <EncounterPersistentRecovery
                      group={group}
                      recordedResult={
                        applied
                          ? recoveryResults.find(
                              (r) => r.episode_id === group.episode_id,
                            )
                          : staged
                      }
                      resultPending={!applied}
                      size="middle"
                      appearance="button"
                      boundaryKey={boundaryKey}
                      mode="end_turn"
                      available={
                        active &&
                        (applied ? !!episode?.end_turn_recovery_available : true)
                      }
                      rollAvailable={
                        active &&
                        (applied
                          ? !!episode?.end_turn_roll_available
                          : hasDamage && !staged)
                      }
                      rollUnavailableReason={
                        applied
                          ? episode?.end_turn_roll_unavailable_reason
                          : !hasDamage
                            ? `Enter or roll ${label.toLowerCase()} damage first.`
                            : staged
                              ? "Recovery already rolled for this turn."
                              : undefined
                      }
                      participantKey={participant.participant_key}
                      encounterRevision={applied ? encounterRevision : anchor.revision}
                      disabled={
                        disabled ||
                        (changed && !applied) ||
                        ((!!busy || damageRoll.busy) &&
                          busy !== `recovery:${group.episode_id}`)
                      }
                      pending={pending}
                      damageType={label}
                      onRecover={
                        applied
                          ? onRecover
                          : async (request) => {
                              const completed = await prepare(
                                `recovery:${group.episode_id}`,
                                groups,
                                {
                                  episode_id: group.episode_id,
                                  dc_override: request.dc_override,
                                },
                              );
                              if (!completed.recovery)
                                throw new Error("Recovery result is unavailable.");
                              const recovery = completed.recovery;
                              setRecoveries((previous) => ({
                                ...previous,
                                [group.episode_id]: recovery,
                              }));
                              return recovery;
                            }
                      }
                    />
                  )}
                  {active && (
                    <Button
                      aria-label={`${clears[group.episode_id] ? "Keep" : "Clear"} ${label.toLowerCase()} effect`}
                      icon={
                        clears[group.episode_id] ? <Undo2 size={14} /> : <X size={14} />
                      }
                      disabled={
                        disabled ||
                        pending ||
                        (changed && !applied) ||
                        !!busy ||
                        damageRoll.busy ||
                        !active ||
                        !!staged?.success
                      }
                      onClick={async () => {
                        if (!applied) {
                          setClears((previous) => ({
                            ...previous,
                            [group.episode_id]: !previous[group.episode_id],
                          }));
                          return;
                        }
                        try {
                          await onRecover({
                            resolution: "clear",
                            boundary_key: boundaryKey,
                            mode: "end_turn",
                            participant_key: participant.participant_key,
                            episode_id: group.episode_id,
                            expected_encounter_revision: encounterRevision,
                            dc_override: null,
                          });
                        } catch (cause) {
                          setError(
                            cause instanceof Error
                              ? cause.message
                              : "Could not clear effect.",
                          );
                        }
                      }}
                    >
                      {clears[group.episode_id] ? "Keep effect" : "Clear effect"}
                    </Button>
                  )}
                </Space>
                {!applied && (pendingClear || staged) && (
                  <span className="encounter-condition-meta">
                    {pendingClear ? "Clears after Apply" : "Still active"}
                  </span>
                )}
              </div>
            );
            return (
              <div key={group.episode_id}>
                {renderGroup ? (
                  renderGroup(group, damage, recovery)
                ) : (
                  <section className="encounter-persistent-type">
                    <h4>{label}</h4>
                    {damage}
                    {recovery}
                  </section>
                )}
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
        {!applied && (
          <div className="encounter-persistent-footer">
            <div aria-live="polite">
              {current?.error ? (
                <Alert
                  type="error"
                  title={current.error}
                  action={
                    <Button
                      onClick={() => setPreviewAttempt((previous) => previous + 1)}
                    >
                      Retry
                    </Button>
                  }
                />
              ) : current?.value ? (
                <span>
                  Total: <strong>{current.value.damage.total.toString()} damage</strong>
                  {current.value.damage.hardness_absorbed > 0n
                    ? ` · Hardness absorbs ${current.value.damage.hardness_absorbed}`
                    : ""}
                  {!current.value.complete
                    ? ` · ${participant.persistent_damage_groups
                        .filter(
                          (g) =>
                            !groups.some((input) => input.episode_id === g.episode_id),
                        )
                        .map(
                          (g) =>
                            damageTypes.find((type) => type.value === g.damage_type)
                              ?.label ?? g.damage_type,
                        )
                        .join(", ")} not ${mode === "roll" ? "rolled" : "entered"}`
                    : ""}
                </span>
              ) : (
                <span className="muted">Updating total…</span>
              )}
              {clearingTypes.length > 0 && <div>Clear {clearingTypes.join(", ")}</div>}
            </div>
            <Tooltip
              title="Damage applies together; effects triggered by damage happen once."
              trigger={["hover", "focus"]}
            >
              <Button
                type="primary"
                htmlType="submit"
                disabled={!ready || !current?.value?.complete}
                loading={pending}
              >
                {current?.value?.complete
                  ? `Apply ${current.value.damage.total} damage`
                  : "Apply damage"}
              </Button>
            </Tooltip>
          </div>
        )}
      </Form>
    </>
  );
}

function UnrolledDamage({
  expression,
  formula,
  pending,
  children,
}: {
  expression?: import("../../generated/atlas").RollPreviewExpression;
  formula: string;
  pending: boolean;
  children: ReactNode;
}) {
  return expression ? (
    <UnrolledRoll expression={expression} formula={formula} pending={pending}>
      {children}
    </UnrolledRoll>
  ) : (
    children
  );
}

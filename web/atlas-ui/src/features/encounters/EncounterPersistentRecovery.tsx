import { Alert, Button, Form, InputNumber, Popover, Select } from "antd";
import { useState } from "react";
import type {
  PersistentDamageGroupView,
  PersistentRecoveryResultView,
  RecoverPersistentDamageRequest,
} from "../../generated/atlas";
import { useDiceRoll } from "../../shared/dice/useDiceRoll";
import { RollResult } from "../../shared/dice/RollResult";
import { integerRule } from "./EncounterConditionEditor";

type Values = { source?: string; dc?: number | null };
export function EncounterPersistentRecovery({
  group,
  participantKey,
  encounterRevision,
  boundaryKey = null,
  disabled,
  pending,
  damageType,
  mode,
  available,
  unavailableReason,
  size = "small",
  rollAvailable = available,
  rollUnavailableReason,
  onRecover,
  onResult,
  onOpenChange,
  recordedResult,
}: {
  group: PersistentDamageGroupView;
  mode: "end_turn" | "assisted";
  available: boolean;
  size?: "small" | "middle";
  unavailableReason?: string | null;
  rollAvailable?: boolean;
  rollUnavailableReason?: string | null;
  participantKey: string;
  encounterRevision: bigint;
  boundaryKey?: string | null;
  disabled: boolean;
  pending: boolean;
  damageType: string;
  onRecover: (
    request: Omit<RecoverPersistentDamageRequest, "request_id">,
  ) => Promise<PersistentRecoveryResultView>;
  recordedResult?: PersistentRecoveryResultView;
  onResult?: (result: PersistentRecoveryResultView) => void;
  onOpenChange?: (open: boolean) => void;
}) {
  const [open, setOpen] = useState(false);
  const [revision, setRevision] = useState(encounterRevision);
  const [error, setError] = useState<string>();
  const dice = useDiceRoll<PersistentRecoveryResultView>();
  const result = dice.result ?? recordedResult;
  const [form] = Form.useForm<Values>();
  const selectedId =
    Form.useWatch("source", form) ?? group.selected_source_id?.toString();
  const source = group.sources.find(
    (source) => source.condition_id.toString() === selectedId,
  );
  const changed = revision !== encounterRevision;
  const resetFields = () => {
    form.resetFields();
    const dc = group.sources.find(
      (source) => source.condition_id === group.selected_source_id,
    )?.recovery_dc;
    form.setFieldsValue({ dc: dc == null ? null : Number(dc) });
  };
  const run = async () => {
    const outcome = "roll" as const;
    if (
      disabled ||
      pending ||
      (changed && !result) ||
      !available ||
      (outcome === "roll" && !rollAvailable)
    )
      return;
    try {
      const values = await form.validateFields(["source", "dc"]);
      const selected = values.source ?? group.selected_source_id?.toString();
      if (!selected) return;
      if (
        group.sources.find((source) => source.condition_id.toString() === selected)
          ?.recovery_dc === null &&
        (values.dc === undefined || values.dc === null)
      ) {
        form.setFields([{ name: "dc", errors: ["Enter a whole number."] }]);
        return;
      }
      setError(undefined);
      await dice.run(async () => {
        const completed = await onRecover({
          mode,
          boundary_key: boundaryKey,
          expected_encounter_revision: result ? encounterRevision : revision,
          participant_key: participantKey,
          episode_id: group.episode_id,
          selected_source_id: selected ? BigInt(selected) : null,
          outcome: { kind: outcome },
          dc_override:
            values.dc === undefined ||
            values.dc === null ||
            BigInt(values.dc) === source?.recovery_dc
              ? null
              : BigInt(values.dc),
        });
        onResult?.(completed);
        return completed;
      });
    } catch (cause) {
      if (cause instanceof Error) setError(cause.message);
    }
  };
  return (
    <>
      <Popover
        title={`${damageType} recovery`}
        open={open}
        trigger="click"
        onOpenChange={(next) => {
          if (pending) return;
          if (next) {
            resetFields();
            setRevision(encounterRevision);
            setError(undefined);
          }
          setOpen(next);
          onOpenChange?.(next);
        }}
        content={
          <div className="encounter-recovery-actions">
            {dice.error && <Alert type="error" title={dice.error} />}
            {error && (
              <Alert type="error" title="Recovery failed" description={error} />
            )}
            {changed && !result && (
              <Alert
                type="warning"
                title="Encounter changed"
                action={
                  <Button
                    onClick={() => {
                      resetFields();
                      setRevision(encounterRevision);
                    }}
                  >
                    Reload recovery
                  </Button>
                }
              />
            )}
            <Form
              form={form}
              layout="inline"
              disabled={disabled || pending || (changed && !result)}
            >
              {!result?.success && group.selected_source_id === null && (
                <Form.Item
                  name="source"
                  label="Damage source"
                  layout="vertical"
                  style={{ flexBasis: "100%", marginBottom: 12 }}
                  rules={[
                    {
                      required: true,
                      message: "Choose the source with the greater damage.",
                    },
                  ]}
                >
                  <Select
                    aria-label="Recovery damage source"
                    options={group.sources.map((source) => ({
                      value: source.condition_id.toString(),
                      label: `${source.formula}${source.multiplier === "normal" ? "" : ` · ${source.multiplier}`}`,
                    }))}
                    onChange={(id) => {
                      const dc = group.sources.find(
                        (source) => source.condition_id.toString() === id,
                      )?.recovery_dc;
                      form.setFieldsValue({ dc: dc == null ? null : Number(dc) });
                    }}
                  />
                </Form.Item>
              )}
              {(!result || !result.success) && (
                <div className="encounter-recovery-roll">
                  <Form.Item
                    name="dc"
                    label="Flat check DC"
                    style={{ marginBottom: 0, marginInlineEnd: 0 }}
                    rules={[integerRule(true)]}
                  >
                    <InputNumber
                      aria-label="Recovery DC"
                      min={0}
                      max={100}
                      precision={0}
                      style={{ width: 72 }}
                    />
                  </Form.Item>
                  <Button
                    type="primary"
                    disabled={
                      disabled ||
                      pending ||
                      (changed && !result) ||
                      !rollAvailable ||
                      dice.busy
                    }
                    loading={pending || dice.busy}
                    onClick={() => void run()}
                  >
                    {result ? "Reroll" : "Roll"}
                  </Button>
                </div>
              )}
              {!rollAvailable && rollUnavailableReason && (
                <p>{rollUnavailableReason}</p>
              )}
            </Form>
            {result && (
              <>
                {result.roll && (
                  <RollResult outcome={result.roll} pending={dice.animating} />
                )}
                {!dice.animating && (
                  <p>
                    {result.success ? "Recovered" : "Still active"}
                    {result.dc == null ? "" : ` · DC ${result.dc}`}
                  </p>
                )}
              </>
            )}
          </div>
        }
      >
        <Button
          size={size}
          aria-label={`Roll ${damageType} recovery`}
          disabled={disabled || (!available && !result)}
          onClick={(event) => event.stopPropagation()}
        >
          Roll recovery
        </Button>
      </Popover>
      {!available && unavailableReason && (
        <span className="muted">{unavailableReason}</span>
      )}
    </>
  );
}

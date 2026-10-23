import { Dice5 } from "lucide-react";
import { Alert, Button, Form, InputNumber, Popover, Tooltip } from "antd";
import { useState } from "react";
import type {
  PersistentDamageGroupView,
  PersistentRecoveryResultView,
  RecoverPersistentDamageRequest,
} from "../../generated/atlas";
import { useDiceRoll } from "../../shared/dice/useDiceRoll";
import { RollResult } from "../../shared/dice/RollResult";
import { integerRule } from "./EncounterConditionEditor";

type Values = { dc?: number | null };
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
  appearance = "button",
  rollAvailable = available,
  rollUnavailableReason,
  onRecover,
  onResult,
  onOpenChange,
  recordedResult,
}: {
  group: PersistentDamageGroupView;
  mode: "end_turn" | "explicit";
  available: boolean;
  size?: "small" | "middle";
  appearance?: "button" | "icon";
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
  const source = group.sources.find(
    (source) => source.condition_id === group.selected_source_id,
  );
  const changed = revision !== encounterRevision;
  const resetFields = () => {
    form.resetFields();
    form.setFieldsValue({
      dc: source?.recovery_dc == null ? null : Number(source.recovery_dc),
    });
  };
  const run = async () => {
    if (
      disabled ||
      pending ||
      result?.success ||
      (changed && !result) ||
      !available ||
      !rollAvailable
    )
      return;
    try {
      const values = await form.validateFields(["dc"]);
      setError(undefined);
      await dice.run(async () => {
        const completed = await onRecover({
          resolution: "roll",
          mode,
          boundary_key: boundaryKey,
          expected_encounter_revision: result ? encounterRevision : revision,
          participant_key: participantKey,
          episode_id: group.episode_id,
          selected_source_id: group.selected_source_id,
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
        placement="bottomRight"
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
              disabled={disabled || pending || result?.success || (changed && !result)}
            >
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
                  className="encounter-recovery-roll__submit"
                  disabled={
                    disabled ||
                    pending ||
                    result?.success ||
                    (changed && !result) ||
                    !rollAvailable ||
                    dice.busy
                  }
                  loading={pending || dice.busy}
                  onClick={() => void run()}
                >
                  {result && !result.success ? "Reroll" : "Roll"}
                </Button>
              </div>
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
        <Tooltip
          title={
            appearance === "icon" && !open ? `Roll ${damageType} recovery` : undefined
          }
          trigger={["hover", "focus"]}
        >
          <Button
            type={appearance === "icon" ? "text" : "default"}
            icon={
              appearance === "icon" ? <Dice5 size={14} aria-hidden="true" /> : undefined
            }
            size={size}
            aria-label={`Roll ${damageType} recovery`}
            disabled={disabled || (!available && !result)}
            onClick={(event) => event.stopPropagation()}
          >
            {appearance === "icon" ? null : "Roll recovery"}
          </Button>
        </Tooltip>
      </Popover>
      {!available && unavailableReason && (
        <span className="muted">{unavailableReason}</span>
      )}
    </>
  );
}

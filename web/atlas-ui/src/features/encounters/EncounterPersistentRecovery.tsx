import { userErrorMessage } from "../../shared/feedback/errorMessages";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { Dice5 } from "lucide-react";
import { Alert, Button, Form, InputNumber, Popover, Tooltip } from "antd";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type {
  PersistentDamageGroupView,
  PersistentRecoveryResultView,
  RecoverPersistentDamageRequest,
} from "../../generated/atlas";
import { useDiceRoll } from "../../shared/dice/useDiceRoll";
import { RollResult } from "../../shared/dice/RollResult";
import { nonNegativeIntegerRule } from "../../shared/ui/forms/nonNegativeIntegerRule";

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
  resultPending = false,
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
  resultPending?: boolean;
  onResult?: (result: PersistentRecoveryResultView) => void;
  onOpenChange?: (open: boolean) => void;
}) {
  const [open, setOpen] = useState(false);
  const [revision, setRevision] = useState(encounterRevision);
  const [error, setError] = useState<string>();
  const dice = useDiceRoll<PersistentRecoveryResultView>();
  const [tableResult, setTableResult] = useState<PersistentRecoveryResultView>();
  const [clearing, setClearing] = useState(false);
  const result = tableResult ?? dice.result ?? recordedResult;
  const check = result?.roll
    ? result
    : tableResult &&
      (dice.result?.roll
        ? dice.result
        : recordedResult?.roll
          ? recordedResult
          : undefined);
  const displayedDc = check?.dc ?? result?.dc;
  const trigger = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const resultFocus = useRef<{
    form: HTMLFormElement;
    cancelled: boolean;
  } | null>(null);
  useLayoutEffect(() => {
    if (!open) {
      resultFocus.current = null;
      return;
    }
    if (mode !== "end_turn" || !result || !resultFocus.current) return;
    if (disabled || pending || dice.busy || trigger.current?.disabled) return;
    const handoff = resultFocus.current;
    resultFocus.current = null;
    const owner = trigger.current?.ownerDocument;
    if (owner && !handoff.cancelled && owner.activeElement === owner.body)
      trigger.current?.focus({ preventScroll: true });
  }, [open, mode, result, disabled, pending, dice.busy]);
  useEffect(() => {
    const owner = open ? trigger.current?.ownerDocument : null;
    if (!owner) return;
    const moved = (event: FocusEvent) => {
      const handoff = resultFocus.current;
      if (
        handoff &&
        event.target !== owner.body &&
        !handoff.form.contains(event.target as Node)
      )
        handoff.cancelled = true;
    };
    owner.addEventListener("focusin", moved);
    return () => owner.removeEventListener("focusin", moved);
  }, [open]);
  const [form] = Form.useForm<Values>();
  const source = group.sources.find(
    (source) => source.condition_id === group.selected_source_id,
  );
  const changed = revision !== encounterRevision;
  const resetFields = () => {
    if (result && mode === "end_turn") return;
    form.resetFields();
    form.setFieldsValue({
      dc: source?.recovery_dc == null ? null : Number(source.recovery_dc),
    });
  };
  const run = async () => {
    if (
      disabled ||
      pending ||
      clearing ||
      dice.busy ||
      result?.success ||
      (changed && !result) ||
      !available ||
      !rollAvailable
    )
      return;
    try {
      const values = await form.validateFields(["dc"]);
      setError(undefined);
      const sourceForm = panel.current?.querySelector("form");
      resultFocus.current =
        mode === "end_turn" &&
        sourceForm?.contains(sourceForm.ownerDocument.activeElement)
          ? { form: sourceForm, cancelled: false }
          : null;
      let received = false;
      await dice.run(async () => {
        const completed = await onRecover({
          resolution: "roll",
          mode,
          boundary_key: boundaryKey,
          expected_encounter_revision: result ? encounterRevision : revision,
          participant_key: participantKey,
          episode_id: group.episode_id,

          dc_override:
            values.dc === undefined ||
            values.dc === null ||
            BigInt(values.dc) === source?.recovery_dc
              ? null
              : BigInt(values.dc),
        });
        received = true;
        onResult?.(completed);
        return completed;
      });
      if (!received) resultFocus.current = null;
    } catch (cause) {
      if (cause instanceof Error) setError(userErrorMessage(cause));
    }
  };
  const clear = async () => {
    if (
      disabled ||
      pending ||
      clearing ||
      dice.busy ||
      result?.success ||
      (changed && !result) ||
      !available
    )
      return;
    setClearing(true);
    setError(undefined);
    try {
      const completed = await onRecover({
        resolution: "clear",
        mode,
        boundary_key: boundaryKey,
        expected_encounter_revision: result ? encounterRevision : revision,
        participant_key: participantKey,
        episode_id: group.episode_id,
        dc_override: null,
      });
      setTableResult(completed);
      onResult?.(completed);
    } catch (cause) {
      setError(userErrorMessage(cause, "Could not clear the effect."));
    } finally {
      setClearing(false);
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
          if (pending || clearing || dice.busy) return;
          if (next) {
            resetFields();
            setRevision(encounterRevision);
            setError(undefined);
          }
          setOpen(next);
          onOpenChange?.(next);
        }}
        content={
          <div ref={panel} className="atlas-dice-popover">
            {dice.error && !tableResult?.success && (
              <ErrorNotice error={dice.error} title={dice.error} />
            )}
            {error && (
              <ErrorNotice
                error={error}
                title="Recovery could not be confirmed"
                description={error}
              />
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
            {!(result && mode === "end_turn") && (
              <Form
                form={form}
                layout="inline"
                disabled={
                  disabled || pending || result?.success || (changed && !result)
                }
              >
                <div className="encounter-recovery-roll">
                  <Form.Item
                    name="dc"
                    label="Flat check DC"
                    style={{ marginBottom: 0, marginInlineEnd: 0 }}
                    rules={[nonNegativeIntegerRule(true)]}
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
                      clearing ||
                      result?.success ||
                      (changed && !result) ||
                      !rollAvailable ||
                      dice.busy
                    }
                    loading={pending || dice.busy}
                    onClick={() => void run()}
                  >
                    {result && !result.success && mode === "explicit"
                      ? "Reroll"
                      : "Roll"}
                  </Button>
                  {mode === "explicit" && (
                    <Button
                      disabled={
                        disabled ||
                        pending ||
                        clearing ||
                        dice.busy ||
                        result?.success ||
                        (changed && !result) ||
                        !available
                      }
                      loading={clearing}
                      onClick={() => void clear()}
                    >
                      Clear effect
                    </Button>
                  )}
                </div>
                {!rollAvailable && rollUnavailableReason && (
                  <p>{rollUnavailableReason}</p>
                )}
              </Form>
            )}
            {result && (
              <>
                {check?.roll && (
                  <RollResult outcome={check.roll} pending={dice.animating} />
                )}
                {!dice.animating && (
                  <p>
                    {result.success
                      ? resultPending
                        ? "Clears after Apply"
                        : "Recovered"
                      : mode === "end_turn"
                        ? "Recovery failed"
                        : "Still active"}
                    {displayedDc == null ? "" : ` · DC ${displayedDc}`}
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
            ref={trigger}
            type={appearance === "icon" ? "text" : "default"}
            icon={
              appearance === "icon" ? <Dice5 size={14} aria-hidden="true" /> : undefined
            }
            size={size}
            aria-label={`${result && mode === "end_turn" ? "View" : "Roll"} ${damageType} recovery`}
            disabled={disabled || (!available && !result)}
            onClick={(event) => event.stopPropagation()}
          >
            {appearance === "icon" ? null : mode === "end_turn" ? (
              <span className="encounter-recovery-trigger-label">
                <span aria-hidden={!!result}>Roll recovery</span>
                <span aria-hidden={!result}>View result</span>
              </span>
            ) : (
              "Roll recovery"
            )}
          </Button>
        </Tooltip>
      </Popover>
      {!available && unavailableReason && (
        <span className="muted">{unavailableReason}</span>
      )}
    </>
  );
}

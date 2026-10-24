import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Popover, Space, Tooltip } from "antd";
import { createContext, use, useState, type ReactNode } from "react";
import { Dice5, HeartPulse } from "lucide-react";
import { RollResult } from "../../shared/dice/RollResult";
import { useDiceRoll } from "../../shared/dice/useDiceRoll";
import { getEncounterConditionLifecycleActions } from "../../api/atlasApi";
import type {
  ApplyConditionLifecycleRequest,
  ConditionLifecycleActionView,
  ConditionLifecycleOptionView,
  ConditionLifecycleResultView,
  EncounterParticipantView,
  EncounterParticipantConditionView,
  LifecycleCheckInputView,
} from "../../generated/atlas";
type RecoveryControls = {
  conditionActions: (id: bigint) => ReactNode;
  retainedCondition?: EncounterParticipantConditionView;
  notice?: ReactNode;
  turnActions?: ReactNode;
};
const RecoveryContext = createContext<RecoveryControls>({
  conditionActions: () => null,
});
export const useConditionRecoveryControls = () => use(RecoveryContext);
export function EncounterTurnRecoveryActions() {
  return useConditionRecoveryControls().turnActions;
}
const optionKey = (option: ConditionLifecycleOptionView) =>
  `${option.kind}:${"condition_id" in option ? option.condition_id : ""}:${"consequence_key" in option ? option.consequence_key : ""}`;

/** One query owner; condition rows and turn reminders place its hints and actions. */
export function EncounterConditionRecovery({
  encounterRef,
  boundaryKey = null,
  contextualTurnActions = false,
  revision,
  participant,
  disabled,
  onApply,
  children,
}: {
  encounterRef: string;
  boundaryKey?: string | null;
  contextualTurnActions?: boolean;
  revision: bigint;
  participant: EncounterParticipantView;
  disabled: boolean;
  children: ReactNode;
  onApply: (
    request: Omit<ApplyConditionLifecycleRequest, "request_id">,
  ) => Promise<ConditionLifecycleResultView>;
}) {
  const query = useQuery({
    queryKey: [
      "condition-lifecycle-actions",
      encounterRef,
      participant.participant_key,
      revision.toString(),
      boundaryKey,
    ],
    queryFn: () =>
      getEncounterConditionLifecycleActions(
        encounterRef,
        participant.participant_key,
        boundaryKey,
      ),
    placeholderData: (previous, query) =>
      query?.queryKey[1] === encounterRef &&
      query.queryKey[2] === participant.participant_key &&
      query.queryKey[4] === boundaryKey
        ? previous
        : undefined,
  });
  const [selected, setSelected] = useState<{
    anchor: string;
    option?: ConditionLifecycleOptionView;
    revision: bigint;
    condition?: EncounterParticipantConditionView;
  } | null>(null);
  const options = query.data?.actions ?? [];
  const renderActions = (
    anchor: string,
    choices: ConditionLifecycleOptionView[],
    label: string,
    icon: ReactNode,
    conditionId?: bigint,
  ) => {
    const active = selected?.anchor === anchor;
    if (!choices.length && !active) return null;
    const unavailable =
      choices.length > 0 && choices.every((choice) => !choice.available);
    return (
      <Popover
        key={anchor}
        open={active}
        trigger="click"
        placement="bottom"
        destroyOnHidden
        title={active && selected.option ? selected.option.label : label}
        onOpenChange={(open) => {
          if (disabled) return;
          setSelected(
            open
              ? {
                  anchor,
                  revision: query.data?.encounter_revision ?? revision,
                  option: choices.length === 1 ? choices[0] : undefined,
                  condition: participant.conditions.find(
                    (condition) => condition.condition_id === conditionId,
                  ),
                }
              : null,
          );
        }}
        content={
          active ? (
            <div
              className="encounter-recovery-popover"
              role="dialog"
              aria-label={selected.option?.label ?? label}
              onKeyDownCapture={(event) => {
                if (
                  event.key === "Escape" &&
                  !disabled &&
                  !event.currentTarget.querySelector(
                    '[role="combobox"][aria-expanded="true"]',
                  )
                ) {
                  event.stopPropagation();
                  setSelected(null);
                }
              }}
            >
              {selected.option ? (
                <RecoveryContent
                  key={`${optionKey(selected.option)}:${selected.revision}`}
                  option={selected.option}
                  changed={selected.revision !== revision}
                  disabled={disabled || query.isFetching}
                  onReload={() => {
                    const option = options.find(
                      (option) => optionKey(option) === optionKey(selected.option!),
                    );
                    if (option) setSelected({ ...selected, option, revision });
                    else setSelected(null);
                  }}
                  onApply={async (action) => {
                    const result = await onApply({
                      boundary_key: boundaryKey,
                      expected_encounter_revision: selected.revision,
                      participant_key: participant.participant_key,
                      action,
                    });
                    void query.refetch();
                    return result;
                  }}
                />
              ) : (
                <Space orientation="vertical" align="start">
                  {choices.map((option) => (
                    <Tooltip key={optionKey(option)} title={option.unavailable_reason}>
                      <Button
                        disabled={disabled || !option.available}
                        onClick={() => setSelected({ ...selected, option })}
                      >
                        {option.label}
                      </Button>
                    </Tooltip>
                  ))}
                </Space>
              )}
            </div>
          ) : null
        }
      >
        <Tooltip
          title={
            active ? undefined : unavailable ? choices[0]?.unavailable_reason : label
          }
          trigger={["hover", "focus"]}
        >
          <Button
            type="text"
            size="small"
            icon={icon}
            aria-label={label}
            className={
              conditionId === undefined ? undefined : "encounter-condition-recovery"
            }
            disabled={disabled || unavailable}
            onKeyDown={(event) => {
              if (event.key === "Escape" && !disabled) setSelected(null);
            }}
          />
        </Tooltip>
      </Popover>
    );
  };
  const turnOptions =
    selected?.anchor.startsWith("turn:") &&
    selected.option &&
    !options.some((option) => optionKey(option) === optionKey(selected.option!))
      ? [...options, selected.option]
      : options;
  const controls: RecoveryControls = {
    conditionActions: (id) =>
      renderActions(
        `condition:${id}`,
        options.filter((option) => option.condition_ids.includes(id)),
        `Recover ${participant.conditions.find((condition) => condition.condition_id === id)?.name ?? selected?.condition?.name ?? "condition"}`,
        <HeartPulse size={14} aria-hidden="true" />,
        id,
      ),
    retainedCondition:
      selected?.condition &&
      !participant.conditions.some(
        (condition) => condition.condition_id === selected.condition!.condition_id,
      )
        ? selected.condition
        : undefined,
    turnActions: (
      <Space wrap>
        {turnOptions.map((option) =>
          option.kind === "frightened_end_turn" ? (
            <TurnFrightenedReduction
              key={optionKey(option)}
              option={option}
              label={contextualTurnActions ? "Reduce by 1" : option.label}
              disabled={disabled || query.isFetching}
              onApply={async () => {
                await onApply({
                  boundary_key: boundaryKey,
                  expected_encounter_revision:
                    query.data?.encounter_revision ?? revision,
                  participant_key: participant.participant_key,
                  action: { kind: "frightened_end_turn" },
                });
                void query.refetch();
              }}
            />
          ) : (
            renderActions(
              `turn:${optionKey(option)}`,
              [option],
              option.label,
              <Dice5 size={14} aria-hidden="true" />,
            )
          ),
        )}
      </Space>
    ),
    notice: query.error ? (
      <Alert
        type="warning"
        title="Recovery choices unavailable"
        action={
          <Button size="small" onClick={() => void query.refetch()}>
            Retry
          </Button>
        }
      />
    ) : null,
  };
  return <RecoveryContext value={controls}>{children}</RecoveryContext>;
}
function RecoveryContent({
  option,
  changed,
  disabled,
  onReload,
  onApply,
}: {
  option: ConditionLifecycleOptionView;
  changed: boolean;
  disabled: boolean;
  onReload: () => void;
  onApply: (
    action: ConditionLifecycleActionView,
  ) => Promise<ConditionLifecycleResultView>;
}) {
  const dice = useDiceRoll<ConditionLifecycleResultView>();
  const apply = (action: ConditionLifecycleActionView) =>
    dice.run(() => onApply(action));
  const blocked = disabled || dice.busy || changed;
  const check = "check" in option ? option.check : undefined;
  const canRoll =
    check?.dc != null &&
    check.modifier != null &&
    (option.kind !== "retch" || option.issues.length === 0);
  const withCheck = (check: LifecycleCheckInputView): ConditionLifecycleActionView => {
    switch (option.kind) {
      case "dying_recovery":
        return { kind: option.kind, check };
      case "retch":
        return { kind: option.kind, condition_id: option.condition_id, check };
      case "confused_damage_recovery":
        return { kind: option.kind, consequence_key: option.consequence_key, check };
      default:
        throw new Error("This recovery does not use a check.");
    }
  };
  if (option.kind === "reminder") return <p>{option.text}</p>;
  if (dice.result) {
    const result = dice.result.check;
    return (
      <div role="status">
        {result?.roll && <RollResult outcome={result.roll} pending={dice.animating} />}
        {(!result?.roll || !dice.animating) && (
          <p>
            {result
              ? `${result.degree.replace(/_/g, " ")}${result.dc === null ? "" : ` · DC ${result.dc}`}`
              : "Saved"}
          </p>
        )}
      </div>
    );
  }
  return (
    <>
      {changed && (
        <Alert
          type="warning"
          title="Encounter changed"
          action={
            <Button disabled={disabled} onClick={onReload}>
              Reload recovery
            </Button>
          }
        />
      )}
      {dice.error && (
        <Alert type="error" title="Recovery failed" description={dice.error} />
      )}
      {option.kind === "retch" && (
        <>
          <p>Retch · 1 action. Success reduces Sickened by 1; critical success by 2.</p>
          {option.issues.map((issue, index) => (
            <p key={index}>{issue.reason}</p>
          ))}
        </>
      )}
      {option.kind === "confused_damage_recovery" && (
        <p>After damage from an attack or spell, DC 11 flat check to end Confused.</p>
      )}
      {check && (
        <>
          {canRoll && (
            <Button
              type="primary"
              loading={dice.busy}
              className="encounter-recovery-roll__submit"
              disabled={blocked}
              onClick={() => void apply(withCheck({ kind: "roll" }))}
            >
              Roll{check.dc === null ? "" : ` · DC ${check.dc}`}
            </Button>
          )}
          {option.kind !== "confused_damage_recovery" && (
            <>
              <p>Rolled at the table</p>
              <Space wrap>
                {(option.kind === "dying_recovery"
                  ? ([
                      "critical_success",
                      "success",
                      "failure",
                      "critical_failure",
                    ] as const)
                  : (["success", "critical_success"] as const)
                ).map((degree) => (
                  <Button
                    key={degree}
                    disabled={blocked}
                    onClick={() =>
                      void apply(withCheck({ kind: "table_degree", degree }))
                    }
                  >
                    {degree
                      .replace(/_/g, " ")
                      .replace(/^./, (value) => value.toUpperCase())}
                  </Button>
                ))}
              </Space>
            </>
          )}
        </>
      )}
      {option.kind === "frightened_end_turn" && (
        <Button
          type="primary"
          loading={dice.busy}
          disabled={blocked}
          onClick={() => void apply({ kind: option.kind })}
        >
          Reduce by 1
        </Button>
      )}
      {option.kind === "stunned_actions_lost" && (
        <>
          <p>{option.actions_lost?.toString()} actions lost</p>
          <Button
            type="primary"
            loading={dice.busy}
            disabled={blocked || option.actions_lost === null}
            onClick={() =>
              void apply({
                kind: option.kind,
                condition_id: option.condition_id,
                actions_lost: option.actions_lost!,
              })
            }
          >
            Apply reduction
          </Button>
        </>
      )}
    </>
  );
}

function TurnFrightenedReduction({
  option,
  label,
  disabled,
  onApply,
}: {
  option: ConditionLifecycleOptionView;
  label: string;
  disabled: boolean;
  onApply: () => Promise<void>;
}) {
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string>();
  return (
    <>
      <Tooltip title={option.unavailable_reason}>
        <Button
          disabled={disabled || pending || !option.available}
          loading={pending}
          onClick={async () => {
            setPending(true);
            setError(undefined);
            try {
              await onApply();
            } catch (cause) {
              setError(
                cause instanceof Error ? cause.message : "Could not reduce Frightened.",
              );
            } finally {
              setPending(false);
            }
          }}
        >
          {label}
        </Button>
      </Tooltip>
      {error && <Alert type="error" title={error} />}
    </>
  );
}

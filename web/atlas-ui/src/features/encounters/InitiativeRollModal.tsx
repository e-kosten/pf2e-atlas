import { EncounterModal } from "./EncounterModal";
import { RollModeControl } from "../../shared/dice/RollModeControl";
import { useDicePopoverFocus } from "../../shared/dice/useDicePopoverFocus";
import {
  isDefiniteRollRejection,
  waitForRollReveal,
} from "../../shared/dice/rollLifecycle";
import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Checkbox, Popover, Select, Spin } from "antd";
import { Dice5 } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import { AtlasApiError, getInitiativeOptions } from "../../api/atlasApi";
import type {
  EncounterParticipantView,
  InitiativeOptionsView,
  InitiativeRollView,
  RollInitiativeRequest,
  RollMode,
  RollExpression,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { DieFace } from "../../shared/dice/DieFace";
import { InitiativeValueField } from "./InitiativeValueField";
import { participantUpdate } from "./participantEdits";

type RollProps = {
  encounterKey: string;
  participants: EncounterParticipantView[];
  initialParticipant?: string;
  onClose: () => void;
  onRoll: (request: RollInitiativeRequest) => Promise<InitiativeRollView>;
  disabled: boolean;
  onSetTotal: (participantKey: string, value: number | undefined) => Promise<unknown>;
};

export function InitiativeRollModal(props: RollProps) {
  return (
    <EncounterModal
      title="Initiative"
      open
      onCancel={props.onClose}
      footer={null}
      width="max-content"
      style={{ maxWidth: "calc(100vw - 32px)" }}
      destroyOnHidden
    >
      <InitiativeEditor {...props} />
    </EncounterModal>
  );
}

export function InitiativePopover({
  participant,
  encounterKey,
  onRoll,
  onUpdate,
  disabled,
}: {
  participant: EncounterParticipantView;
  encounterKey: string;
  onRoll: RollProps["onRoll"];
  onUpdate: (request: UpdateEncounterParticipantRequest) => Promise<unknown>;
  disabled: boolean;
}) {
  const [open, setOpen] = useState(false);
  const headingId = useId();
  const heading = `${participant.display_name} · Initiative`;
  const query = useInitiativeOptions(encounterKey, participant.participant_key, open);
  const visible = open && Boolean(query.data || query.isError);
  const { trigger, content, afterOpenChange } = useDicePopoverFocus(visible);
  useEffect(() => {
    if (!open || visible) return;
    const outside = (event: PointerEvent) => {
      if (event.target instanceof Node && !trigger.current?.contains(event.target))
        setOpen(false);
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", outside, true);
    document.addEventListener("keydown", escape, true);
    return () => {
      document.removeEventListener("pointerdown", outside, true);
      document.removeEventListener("keydown", escape, true);
    };
  }, [open, visible, trigger]);
  const close = () => {
    if (!disabled) {
      setOpen(false);
    }
  };
  return (
    <Popover
      open={visible}
      trigger="click"
      placement="bottom"
      destroyOnHidden
      afterOpenChange={afterOpenChange}
      onOpenChange={(next) => {
        if (disabled) return;
        setOpen(next);
      }}
      content={
        <div
          ref={content}
          tabIndex={-1}
          className="atlas-dice-popover initiative-popover"
          role="dialog"
          aria-labelledby={headingId}
        >
          <strong id={headingId}>{heading}</strong>
          {open && (
            <InitiativeContent
              query={query}
              encounterKey={encounterKey}
              participants={[participant]}
              initialParticipant={participant.participant_key}
              onClose={close}
              onRoll={onRoll}
              onSetTotal={(_key, value) =>
                onUpdate(
                  participantUpdate(participant, {
                    initiative: value === undefined ? undefined : BigInt(value),
                  }),
                )
              }
              disabled={disabled}
            />
          )}
        </div>
      }
    >
      <Button
        ref={trigger}
        type="text"
        size="small"
        disabled={disabled}
        loading={open && !visible ? { delay: 500 } : false}
        aria-label={`${participant.display_name} initiative${participant.initiative == null ? ", not rolled" : `: ${participant.initiative}`}`}
        title="Roll or set initiative"
        icon={participant.initiative == null ? <Dice5 size={15} /> : undefined}
      >
        {participant.initiative?.toString()}
      </Button>
    </Popover>
  );
}

function useInitiativeOptions(
  encounterKey: string,
  participantKey?: string,
  enabled = true,
) {
  return useQuery({
    queryKey: ["initiative-options", encounterKey, participantKey ?? "batch"],
    queryFn: () => getInitiativeOptions(encounterKey),
    enabled,
    staleTime: 0,
    gcTime: 0,
    refetchOnWindowFocus: false,
    retry: false,
  });
}

function InitiativeEditor(props: RollProps) {
  const query = useInitiativeOptions(props.encounterKey, props.initialParticipant);
  return <InitiativeContent {...props} query={query} />;
}

function InitiativeContent({
  query,
  ...props
}: RollProps & { query: ReturnType<typeof useInitiativeOptions> }) {
  if (!query.data)
    return query.isError ? (
      <Alert
        type="error"
        title="Couldn’t load modifiers"
        action={<Button onClick={() => void query.refetch()}>Retry</Button>}
      />
    ) : (
      <Spin delay={300} />
    );
  return (
    <InitiativeForm
      {...props}
      options={query.data}
      refresh={async () => {
        const result = await query.refetch();
        return result.isError ? undefined : result.data;
      }}
    />
  );
}

function InitiativeForm({
  options,
  participants,
  initialParticipant,
  onClose,
  onRoll,
  disabled,
  refresh,
  onSetTotal,
}: RollProps & {
  options: InitiativeOptionsView;
  refresh: () => Promise<InitiativeOptionsView | undefined>;
}) {
  const [draft, setDraft] = useState(() =>
    options.participants
      .filter(
        (p) =>
          participants.some(
            (row) =>
              row.participant_key === p.participant_key &&
              row.participant_kind !== "pc",
          ) &&
          (!initialParticipant || p.participant_key === initialParticipant),
      )
      .map((p) => ({
        ...p,
        selected: true,
        statistic:
          p.statistics.find((s) => s.key === "perception")?.key ??
          p.statistics.find((s) => s.key === "hazard.stealth")?.key ??
          "manual",
        modifier_override: null as number | null,
        mode: "normal" as RollMode,
      })),
  );
  const [pending, setPending] = useState(false);
  const busy = useRef(false);
  async function saveTotal(participantKey: string, value: number | undefined) {
    if (busy.current || disabled) throw new Error("Initiative is busy");
    busy.current = true;
    setPending(true);
    try {
      await onSetTotal(participantKey, value);
      setReceipt(
        (previous) =>
          previous && {
            ...previous,
            rolls: previous.rolls.filter(
              (roll) => roll.participant_key !== participantKey,
            ),
          },
      );
      await reload();
    } finally {
      busy.current = false;
      setPending(false);
    }
  }
  const [attempt, setAttempt] = useState<RollInitiativeRequest | null>(null);
  const [receipt, setReceipt] = useState<InitiativeRollView | null>(null);
  const [error, setError] = useState<string>();
  const [conflict, setConflict] = useState(false);
  const [refreshFailed, setRefreshFailed] = useState(false);
  const valid =
    draft.some((p) => p.selected) &&
    draft
      .filter((p) => p.selected)
      .every((p) => p.statistic !== "manual" || p.modifier_override !== null);
  const locked = disabled || pending || !!attempt;
  async function reload() {
    const latest = await refresh();
    setRefreshFailed(!latest);
    if (latest)
      setDraft((rows) =>
        rows.map((row) => ({
          ...row,
          ...latest.participants.find((p) => p.participant_key === row.participant_key),
        })),
      );
  }
  async function submit() {
    if (busy.current || disabled || !valid) return;
    busy.current = true;
    setPending(true);
    setError(undefined);
    const request = attempt ?? {
      request_id: crypto.randomUUID(),
      artifact_generation: options.artifact_generation,
      expected_revision: options.expected_revision,
      participants: draft
        .filter((p) => p.selected)
        .map(
          ({
            participant_key,
            expected_revision,
            statistic,
            modifier_override,
            mode,
          }) => ({
            participant_key,
            expected_revision,
            statistic,
            modifier_override,
            mode,
          }),
        ),
    };
    setAttempt(request);
    try {
      const [result] = await Promise.all([onRoll(request), waitForRollReveal()]);
      setReceipt(result);
      setAttempt(null);
      await reload();
    } catch (cause) {
      setConflict(
        cause instanceof AtlasApiError &&
          (cause.appError?.code === "encounter_turn_conflict" ||
            cause.appError?.code === "artifact_generation_changed" ||
            cause.appError?.code === "encounter_participant_conflict"),
      );
      if (isDefiniteRollRejection(cause)) {
        setAttempt(null);
        setError(cause.message);
      } else {
        setError("Couldn’t confirm the roll. Retry to recover it.");
      }
    } finally {
      busy.current = false;
      setPending(false);
    }
  }
  const changeDraft = (rows: typeof draft) => {
    setDraft(rows);
    setReceipt(null);
  };
  const actions = (
    <div className="initiative-roll-actions">
      <Button disabled={pending} onClick={onClose}>
        {receipt ? "Done" : "Close"}
      </Button>
      {!conflict && (
        <Button
          type="primary"
          disabled={!valid || disabled || pending || refreshFailed}
          onClick={() => void submit()}
        >
          {error && attempt ? "Retry" : receipt ? "Reroll" : "Roll"}
        </Button>
      )}
    </div>
  );
  return (
    <>
      {!initialParticipant && (
        <div className="initiative-selection">
          <Checkbox
            checked={draft.every((p) => p.selected)}
            indeterminate={
              draft.some((p) => p.selected) && !draft.every((p) => p.selected)
            }
            disabled={locked}
            onChange={(e) =>
              changeDraft(draft.map((p) => ({ ...p, selected: e.target.checked })))
            }
          >
            Select all
          </Checkbox>
          <Button
            type="link"
            size="small"
            disabled={locked || !draft.some((p) => p.selected)}
            onClick={() => changeDraft(draft.map((p) => ({ ...p, selected: false })))}
          >
            Clear selection
          </Button>
        </div>
      )}
      <div
        className={
          initialParticipant ? "initiative-roll-rows--single" : "initiative-roll-rows"
        }
      >
        {draft.map((p) => {
          const name =
            participants.find((row) => row.participant_key === p.participant_key)
              ?.display_name ?? p.participant_key;
          const update = (patch: Partial<typeof p>) =>
            changeDraft(
              draft.map((row) =>
                row.participant_key === p.participant_key ? { ...row, ...patch } : row,
              ),
            );
          const result = receipt?.rolls.find(
            (r) => r.participant_key === p.participant_key,
          );
          const modifier =
            p.modifier_override ??
            p.statistics.find((s) => s.key === p.statistic)?.modifier ??
            null;
          const dice = result ? faces(result.outcome.expression) : [];
          return (
            <div
              className={`initiative-roll-row ${initialParticipant ? "initiative-roll-row--single" : ""}`}
              key={p.participant_key}
            >
              {!initialParticipant && (
                <Checkbox
                  className="initiative-roll-row__name"
                  checked={p.selected}
                  disabled={locked}
                  onChange={(e) => update({ selected: e.target.checked })}
                >
                  {name}
                </Checkbox>
              )}
              <div className="initiative-roll-controls">
                <Select
                  aria-label={`${name} initiative statistic`}
                  showSearch={{ optionFilterProp: "label" }}
                  className="initiative-roll-row__statistic"
                  disabled={locked || !p.selected}
                  value={p.statistic === "manual" ? undefined : p.statistic}
                  placeholder="Statistic"
                  optionLabelProp="shortLabel"
                  popupMatchSelectWidth={220}
                  options={[
                    ...p.statistics.map((s) => ({
                      value: s.key,
                      label: `${s.label} ${s.modifier >= 0 ? "+" : ""}${s.modifier}`,
                      shortLabel: s.label,
                    })),
                  ]}
                  onChange={(statistic) =>
                    update({ statistic, modifier_override: null })
                  }
                />
                <RollModeControl
                  label={name}
                  value={p.mode}
                  onChange={(mode) => update({ mode })}
                  disabled={locked || !p.selected}
                />
              </div>
              <div
                className="initiative-roll-row__result"
                role="status"
                aria-label={
                  pending && attempt && p.selected
                    ? "Rolling"
                    : result
                      ? `Initiative ${result.outcome.total}`
                      : "Not rolled"
                }
              >
                <div className="initiative-roll-dice">
                  {Array.from({ length: p.mode === "normal" ? 1 : 2 }, (_, i) => (
                    <DieFace
                      key={i}
                      sides={20}
                      value={pending ? undefined : dice[i]?.value}
                      kept={pending ? true : (dice[i]?.kept ?? true)}
                      pending={pending && !!attempt && p.selected}
                    />
                  ))}
                </div>
                <span className="initiative-roll-modifier">
                  {modifier === null
                    ? "?"
                    : `${modifier >= 0 ? "+" : "−"} ${Math.abs(modifier)}`}
                </span>
                <span>=</span>
                <InitiativeValueField
                  key={receipt?.request_id ?? "current"}
                  name={name}
                  value={
                    result?.outcome.total ??
                    participants.find(
                      (row) => row.participant_key === p.participant_key,
                    )?.initiative
                  }
                  disabled={locked}
                  onSave={(value) => saveTotal(p.participant_key, value)}
                />
              </div>
              {initialParticipant && actions}
            </div>
          );
        })}
      </div>
      {error && (
        <Alert
          type="warning"
          title={conflict ? "Encounter changed. Close and reopen initiative." : error}
        />
      )}
      {refreshFailed && (
        <Alert
          type="warning"
          title="Saved. Refresh before rolling again."
          action={<Button onClick={() => void reload()}>Refresh</Button>}
        />
      )}
      {!initialParticipant && actions}
    </>
  );
}

function faces(expression: RollExpression): { value: number; kept: boolean }[] {
  switch (expression.kind) {
    case "dice":
      return expression.dice;
    case "binary":
      return [...faces(expression.left), ...faces(expression.right)];
    case "negate":
      return faces(expression.inner);
    case "number":
      return [];
  }
}

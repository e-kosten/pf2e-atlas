import { recordSelection } from "../../shared/records/recordDetailQuery";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { isSharedFailure } from "../../shared/feedback/errorMessages";
import { useQuery } from "@tanstack/react-query";
import { ArrowLeft } from "lucide-react";
import { Tooltip, Alert, Button, Popover } from "antd";
import {
  Fragment,
  useLayoutEffect,
  useRef,
  useState,
  useId,
  type ReactNode,
} from "react";
import { getEncounterStrikeOptions, rollEncounterStrike } from "../../api/atlasApi";
import type {
  ActorActivityView,
  EncounterParticipantView,
  EncounterReferenceRequest,
  EncounterStrikeOptionsView,
  EncounterStrikeRollView,
  RollEncounterStrikeRequest,
  RollMode,
} from "../../generated/atlas";
import { RollModeControl } from "../../shared/dice/RollModeControl";
import { RollResult } from "../../shared/dice/RollResult";
import type { StrikeLaunch } from "./useEncounterStrike";
import { useDiceRoll } from "../../shared/dice/useDiceRoll";
import { useDicePopoverFocus } from "../../shared/dice/useDicePopoverFocus";
import { DamageComponents } from "../../shared/records/ActorReference";

const signed = (value: number) => `${value >= 0 ? "+" : ""}${value}`;

export function EncounterStrikeControl({
  encounterKey,
  generation,
  participant,
  activity,
  disabled,
  onBeginStrike,
  onPrepareStrike,
  renderStrikeContext,
}: {
  encounterKey: string;
  generation: string;
  participant: EncounterParticipantView;
  activity: ActorActivityView;
  disabled: boolean;
  onBeginStrike: (request: StrikeLaunch) => void;
  onPrepareStrike?: (request: StrikeLaunch) => void;
  renderStrikeContext?: (requestId: string) => ReactNode;
}) {
  const nav = activity.navigation;
  const reference: EncounterReferenceRequest = {
    artifact_generation: generation,
    selection: recordSelection({
      ...nav,
      field: null,
      passage: null,
      spell_rank: undefined,
    }),
  };
  const options = useQuery({
    meta: referenceQueryMeta,
    queryKey: [
      "strike-options",
      encounterKey,
      participant.participant_key,
      participant.revision.toString(),
      nav,
      generation,
    ],
    queryFn: () =>
      getEncounterStrikeOptions(encounterKey, {
        participant_key: participant.participant_key,
        reference,
      }),
    enabled: false,
    staleTime: Infinity,
    retry: false,
  });
  const common = {
    encounterKey,
    participantKey: participant.participant_key,
    reference,
    title: activity.title,
    options: options.data,
    disabled,
    refresh: async () => {
      const result = await options.refetch();
      if (result.error) throw result.error;
      if (!result.data) throw Error("Strike unavailable");
      return result.data;
    },
  };
  const damageText = activity.damage.value
    ?.filter((c) => c.formula.value?.trim())
    .map((c) =>
      [c.formula.value, c.damage_type.value?.label, c.category.value]
        .filter(Boolean)
        .join(" "),
    )
    .join(" + ");
  return (
    <div className="strike-inline-rolls">
      <div>
        <strong>Attack </strong>
        {activity.attack_options.length ? (
          activity.attack_options.map((choice, index) => (
            <Fragment key={choice.attack_number}>
              {index > 0 && <span aria-hidden="true"> / </span>}
              <StrikeRollPopover
                {...common}
                label={signed(choice.modifier)}
                attackNumber={choice.attack_number}
                onBeginStrike={onBeginStrike}
                onPrepareStrike={onPrepareStrike}
                renderStrikeContext={renderStrikeContext}
              />
            </Fragment>
          ))
        ) : (
          <span>
            {activity.attack.value != null
              ? signed(activity.attack.value)
              : "Unavailable"}
          </span>
        )}
      </div>
      {damageText ? (
        <div>
          <strong>Damage </strong>
          <StrikeRollPopover {...common} label={damageText} />
        </div>
      ) : (
        <DamageComponents damage={activity.damage} />
      )}
      {options.isError && !isSharedFailure(options.error) && (
        <Button type="link" size="small" onClick={() => void options.refetch()}>
          Retry roll controls
        </Button>
      )}
    </div>
  );
}

function StrikeRollPopover({
  encounterKey,
  participantKey,
  reference,
  title,
  options,
  disabled,
  refresh,
  label,
  attackNumber,
  onBeginStrike,
  onPrepareStrike,
  renderStrikeContext,
}: {
  encounterKey: string;
  participantKey: string;
  reference: EncounterReferenceRequest;
  title: string;
  options: EncounterStrikeOptionsView | undefined;
  disabled: boolean;
  refresh: () => Promise<EncounterStrikeOptionsView>;
  label: string;
  attackNumber?: number;
  onBeginStrike?: (request: StrikeLaunch) => void;
  onPrepareStrike?: (request: StrikeLaunch) => void;
  renderStrikeContext?: (requestId: string) => ReactNode;
}) {
  const [open, setOpen] = useState(false);
  const headingId = useId();
  const [mode, setMode] = useState<RollMode>("normal");
  const [contextRequested, setContextRequested] = useState(false);
  const contextButton = useRef<HTMLButtonElement>(null);
  const contextOrigin = useRef<HTMLElement | null>(null);
  const contextDialog = useRef<HTMLElement | null>(null);
  const contextWasOpen = useRef(false);
  const popover = useRef<import("antd").TooltipRef>(null);
  const { trigger, content, afterOpenChange } = useDicePopoverFocus(open);
  const roll = useDiceRoll<{
    receipt: EncounterStrikeRollView;
    request: RollEncounterStrikeRequest;
    options: EncounterStrikeOptionsView;
  }>({ retainResultOnError: true });
  const result = roll.result?.receipt.result;
  const attackResult = result?.kind === "attack" ? result : undefined;
  const damageResult = result?.kind === "damage" ? result : undefined;
  const original = roll.result?.request;
  const resultOptions = roll.result?.options ?? options;
  const strikeContext =
    contextRequested && original?.action.kind === "attack" && attackResult
      ? renderStrikeContext?.(original.request_id)
      : null;
  const contextOpen = contextRequested && Boolean(strikeContext);
  useLayoutEffect(() => {
    if (!open) return;
    const origin = contextOrigin.current;
    const document = origin?.ownerDocument;
    if (document && document.activeElement === document.body) {
      // Expansion removes its initiating button. Recover only abandoned focus;
      // a user can choose Close or another control while preparation runs.
      if (contextOpen && origin && !origin.isConnected)
        contextDialog.current?.focus({ preventScroll: true });
      else if (contextWasOpen.current && !contextOpen)
        contextButton.current?.focus({ preventScroll: true });
    }
    contextWasOpen.current = contextOpen;
    const frame = requestAnimationFrame(() => popover.current?.forceAlign());
    return () => cancelAnimationFrame(frame);
  }, [open, contextOpen]);
  const attack = attackNumber !== undefined;
  const actionLabel = `${title} ${attack ? "attack" : "damage"}`;
  const act = () =>
    void roll.run(async () => {
      const current = await refresh();
      const request: RollEncounterStrikeRequest = {
        request_id: crypto.randomUUID(),
        expected_revision: current.expected_revision,
        expected_participant_revision: current.expected_participant_revision,
        participant_key: participantKey,
        reference,
        action:
          attackNumber === undefined
            ? { kind: "damage" }
            : {
                kind: "attack",
                attack_number: attackNumber,
                mode,
                modifier_override: null,
              },
      };
      setContextRequested(false);
      const result = await rollEncounterStrike(encounterKey, request);
      if (
        result.request_id !== request.request_id ||
        result.result.kind !== request.action.kind
      )
        throw new Error("Strike response did not match the roll request.");
      if (request.action.kind === "attack" && result.result.kind === "attack")
        onPrepareStrike?.({
          strike: { participant_key: participantKey, reference },
          attackNumber: request.action.attack_number,
          rollId: result.request_id,
          rolledModifier: result.result.attack_modifier,
        });
      return { receipt: result, request, options: current };
    });
  return (
    <Popover
      ref={popover}
      trigger="click"
      placement="bottom"
      align={{
        offset: [0, 0],
        overflow: { adjustX: true, adjustY: true, shiftX: true, shiftY: true },
      }}
      open={open}
      afterOpenChange={afterOpenChange}
      onOpenChange={(next) => {
        setOpen(next);
        if (next) {
          setContextRequested(false);
          if (!disabled && !result && !roll.error) act();
        }
      }}
      content={
        <div
          ref={content}
          tabIndex={-1}
          className={`atlas-dice-popover strike-roll-control${contextOpen ? " strike-roll-control--expanded" : ""}`}
          role="dialog"
          aria-labelledby={headingId}
        >
          <div className="strike-roll-control__toolbar">
            {contextOpen && (
              <Tooltip title="Back to roll" trigger={["hover", "focus"]}>
                <Button
                  type="text"
                  icon={<ArrowLeft size={16} />}
                  aria-label="Back to roll"
                  onClick={() => setContextRequested(false)}
                />
              </Tooltip>
            )}
            <strong id={headingId}>
              {title} · {contextOpen ? "Target & checks" : attack ? "Attack" : "Damage"}
            </strong>
            {attack && (
              <RollModeControl
                label={title}
                value={mode}
                onChange={setMode}
                disabled={disabled || roll.busy}
              />
            )}
          </div>
          {!result && <div>{label}</div>}
          {attackResult && (
            <RollResult outcome={attackResult.attack} pending={roll.animating} />
          )}
          {attack &&
            resultOptions?.applied?.map((hint) => (
              <div key={hint.source}>
                Applied · {hint.label} · {hint.reason}
              </div>
            ))}
          {attack &&
            resultOptions?.conditional?.map((hint) => (
              <div key={hint.source}>
                Context · {hint.label} · {hint.reason}
              </div>
            ))}
          {attack && (
            <>
              {!contextOpen && (
                <Button
                  ref={contextButton}
                  disabled={
                    disabled ||
                    roll.busy ||
                    original?.action.kind !== "attack" ||
                    !attackResult
                  }
                  loading={contextRequested ? { delay: 500 } : false}
                  onClick={(event) => {
                    if (original?.action.kind !== "attack" || !attackResult) return;
                    contextOrigin.current = event.currentTarget;
                    contextDialog.current =
                      event.currentTarget.closest("[role=dialog]");
                    setContextRequested(true);
                    onBeginStrike?.({
                      strike: { participant_key: participantKey, reference },
                      attackNumber: original.action.attack_number,
                      rollId: original.request_id,
                      rolledModifier: attackResult.attack_modifier,
                    });
                  }}
                >
                  Target &amp; checks
                </Button>
              )}
              {contextOpen && original && (
                <div className="strike-roll-control__context">{strikeContext}</div>
              )}
            </>
          )}
          {damageResult?.damage.map((component) => (
            <div key={component.id}>
              <strong>{component.label}</strong>{" "}
              {component.outcome ? (
                <RollResult outcome={component.outcome} pending={roll.animating} />
              ) : (
                component.unavailable
              )}
            </div>
          ))}
          {damageResult?.damage_total != null && !roll.animating && (
            <strong>Damage: {damageResult.damage_total}</strong>
          )}
          {damageResult?.persistent_total != null && !roll.animating && (
            <strong>Persistent damage: {damageResult.persistent_total}</strong>
          )}
          {!!damageResult &&
            !roll.animating &&
            (damageResult.damage_minimum_adjustment > 0 ||
              damageResult.persistent_minimum_adjustment > 0 ||
              damageResult.damage_totals.some((t) => t.raw_total !== t.total)) && (
              <small>Minimum damage applied.</small>
            )}
          {(attack ? options?.unavailable : options?.damage_unavailable) && (
            <Alert
              type="info"
              title={
                (attack ? options?.unavailable : options?.damage_unavailable) ??
                undefined
              }
            />
          )}
          {roll.error && (
            <Alert
              type="warning"
              title={roll.error}
              description={
                result
                  ? "Previous result shown. Roll again to make a new roll."
                  : "Roll again to make a new roll."
              }
            />
          )}
          <div className="strike-roll-control__toolbar">
            <Button onClick={() => setOpen(false)}>Close</Button>
            <Button
              type="primary"
              disabled={
                disabled ||
                roll.busy ||
                (attack ? !!options?.unavailable : options?.damage_available === false)
              }
              loading={roll.busy && !roll.animating ? { delay: 300 } : false}
              onClick={act}
            >
              {roll.error ? "Roll again" : result ? "Reroll" : "Roll"}
            </Button>
          </div>
        </div>
      }
    >
      <Button
        ref={trigger}
        type="link"
        size="small"
        style={{ padding: 0, height: "auto", whiteSpace: "normal", textAlign: "left" }}
        disabled={disabled}
        aria-label={attack ? `Strike ${title} attack ${label}` : `Roll ${actionLabel}`}
      >
        {label}
      </Button>
    </Popover>
  );
}

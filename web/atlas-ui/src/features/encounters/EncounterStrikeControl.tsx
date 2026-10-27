import { useQuery } from "@tanstack/react-query";
import { ArrowLeft } from "lucide-react";
import { Alert, Button, Popover } from "antd";
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
import { isDefiniteRollRejection } from "../../shared/dice/rollLifecycle";
import { DamageComponents } from "../../shared/records/ActorReference";

const signed = (value: number) => `${value >= 0 ? "+" : ""}${value}`;

export function EncounterStrikeControl({
  encounterKey,
  generation,
  participant,
  activity,
  disabled,
  onBeginStrike,
  renderStrikeContext,
}: {
  encounterKey: string;
  generation: string;
  participant: EncounterParticipantView;
  activity: ActorActivityView;
  disabled: boolean;
  onBeginStrike: (request: StrikeLaunch) => void;
  renderStrikeContext?: (requestId: string) => ReactNode;
}) {
  const nav = activity.navigation;
  const reference: EncounterReferenceRequest = {
    artifact_generation: generation,
    selection: {
      record_key: nav.record_key,
      embedded_path: nav.embedded_path,
      fields: [],
      passage: null,
      source_fingerprint: nav.source_fingerprint,
    },
  };
  const options = useQuery({
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
    options: options.isFetching ? undefined : options.data,
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
      [c.formula.value, c.damage_type.value, c.category.value]
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
      {options.isError && (
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
  renderStrikeContext?: (requestId: string) => ReactNode;
}) {
  const [open, setOpen] = useState(false);
  const headingId = useId();
  const [mode, setMode] = useState<RollMode>("normal");
  const [original, setOriginal] = useState<RollEncounterStrikeRequest>();
  const [contextOpen, setContextOpen] = useState(false);
  const popover = useRef<import("antd").TooltipRef>(null);
  useLayoutEffect(() => {
    if (!open) return;
    const frame = requestAnimationFrame(() => popover.current?.forceAlign());
    return () => cancelAnimationFrame(frame);
  }, [open, contextOpen]);
  const { trigger, content, afterOpenChange } = useDicePopoverFocus(open);
  const [retry, setRetry] = useState<RollEncounterStrikeRequest>();
  const roll = useDiceRoll<EncounterStrikeRollView>();
  const attack = attackNumber !== undefined;
  const actionLabel = `${title} ${attack ? "attack" : "damage"}`;
  const act = () =>
    void roll.run(async () => {
      const current = retry ? undefined : await refresh();
      const request = retry ?? {
        request_id: crypto.randomUUID(),
        expected_revision: current!.expected_revision,
        expected_participant_revision: current!.expected_participant_revision,
        participant_key: participantKey,
        reference,
        attack_number: attackNumber ?? 1,
        mode,
        modifier_override: null,
        roll_kind: attack ? ("attack" as const) : ("damage" as const),
      };
      if (!retry) setContextOpen(false);
      setRetry(request);
      try {
        const result = await rollEncounterStrike(encounterKey, request);
        setOriginal(request);
        setRetry(undefined);
        return result;
      } catch (cause) {
        if (isDefiniteRollRejection(cause)) {
          setRetry(undefined);
        }
        throw cause;
      }
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
        if (next && !disabled) act();
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
              <Button
                type="text"
                icon={<ArrowLeft size={16} />}
                aria-label="Back to roll"
                title="Back to roll"
                onClick={() => setContextOpen(false)}
              />
            )}
            <strong id={headingId}>
              {title} · {contextOpen ? "Target & checks" : attack ? "Attack" : "Damage"}
            </strong>
            {attack && (
              <RollModeControl
                label={title}
                value={mode}
                onChange={setMode}
                disabled={disabled || roll.busy || !!retry}
              />
            )}
          </div>
          {!roll.result && <div>{label}</div>}
          {roll.result?.attack && (
            <RollResult outcome={roll.result.attack} pending={roll.animating} />
          )}
          {attack &&
            options?.applied?.map((hint) => (
              <div key={hint.source}>
                Applied · {hint.label} · {hint.reason}
              </div>
            ))}
          {attack &&
            options?.conditional?.map((hint) => (
              <div key={hint.source}>
                Context · {hint.label} · {hint.reason}
              </div>
            ))}
          {attack && (
            <>
              {!contextOpen && (
                <Button
                  disabled={disabled || roll.busy || !original || !roll.result?.attack}
                  onClick={() => {
                    if (!original || !roll.result?.attack) return;
                    setContextOpen(!contextOpen);
                    if (!contextOpen)
                      onBeginStrike?.({
                        strike: { participant_key: participantKey, reference },
                        attackNumber: attackNumber!,
                        rollId: original.request_id,
                        rolledModifier: roll.result?.attack_modifier,
                      });
                  }}
                >
                  Target &amp; checks
                </Button>
              )}
              {contextOpen && original && (
                <div className="strike-roll-control__context">
                  {renderStrikeContext?.(original.request_id)}
                </div>
              )}
            </>
          )}
          {roll.result?.damage.map((component) => (
            <div key={component.id}>
              <strong>{component.label}</strong>{" "}
              {component.outcome ? (
                <RollResult outcome={component.outcome} pending={roll.animating} />
              ) : (
                component.unavailable
              )}
            </div>
          ))}
          {roll.result?.damage_total != null && !roll.animating && (
            <strong>Damage: {roll.result.damage_total}</strong>
          )}
          {roll.result?.persistent_total != null && !roll.animating && (
            <strong>Persistent damage: {roll.result.persistent_total}</strong>
          )}
          {!!roll.result &&
            !roll.animating &&
            (roll.result.damage_minimum_adjustment > 0 ||
              roll.result.persistent_minimum_adjustment > 0 ||
              roll.result.damage_totals.some((t) => t.raw_total !== t.total)) && (
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
              title={
                retry ? "Couldn’t confirm the roll. Retry to recover it." : roll.error
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
              {retry && roll.error ? "Retry" : roll.result ? "Reroll" : "Roll"}
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

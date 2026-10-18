import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Popover } from "antd";
import { Fragment, useState } from "react";
import { getEncounterStrikeOptions, rollEncounterStrike } from "../../api/atlasApi";
import type {
  ActorActivityView,
  EncounterParticipantView,
  EncounterReferenceRequest,
  EncounterStrikeOptionsView,
  EncounterStrikeRollView,
  RollEncounterStrikeRequest,
  RollMode,
  StrikeRollKind,
} from "../../generated/atlas";
import { RollResult } from "../../shared/dice/RollResult";
import { RollModeControl } from "../../shared/dice/RollModeControl";
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
}: {
  encounterKey: string;
  generation: string;
  participant: EncounterParticipantView;
  activity: ActorActivityView;
  disabled: boolean;
}) {
  const nav = activity.navigation;
  const reference: EncounterReferenceRequest = {
    artifact_generation: generation,
    selection: {
      record_key: nav.record_key,
      owners: nav.owners,
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
                kind="attack"
                attackNumber={choice.attack_number}
                label={signed(choice.modifier)}
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
          <StrikeRollPopover
            {...common}
            kind="damage"
            attackNumber={1}
            label={damageText}
          />
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
  kind,
  attackNumber,
  label,
}: {
  encounterKey: string;
  participantKey: string;
  reference: EncounterReferenceRequest;
  title: string;
  options: EncounterStrikeOptionsView | undefined;
  disabled: boolean;
  refresh: () => Promise<EncounterStrikeOptionsView>;
  kind: StrikeRollKind;
  attackNumber: number;
  label: string;
}) {
  const [open, setOpen] = useState(false);
  const { trigger, content, afterOpenChange } = useDicePopoverFocus(open);
  const [mode, setMode] = useState<RollMode>("normal");
  const [retry, setRetry] = useState<RollEncounterStrikeRequest>();
  const roll = useDiceRoll<EncounterStrikeRollView>();
  const actionLabel =
    kind === "attack" ? `${title} attack ${label}` : `${title} damage`;
  const act = () =>
    void roll.run(async () => {
      const current = retry ? undefined : await refresh();
      const request = retry ?? {
        request_id: crypto.randomUUID(),
        expected_revision: current!.expected_revision,
        expected_participant_revision: current!.expected_participant_revision,
        participant_key: participantKey,
        reference,
        attack_number: attackNumber,
        mode,
        modifier_override: null,
        roll_kind: kind,
      };
      setRetry(request);
      try {
        const result = await rollEncounterStrike(encounterKey, request);
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
      trigger="click"
      placement="bottom"
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
          className="strike-roll-control"
          role="dialog"
          aria-label={actionLabel}
        >
          <div className="strike-roll-control__toolbar">
            <strong>
              {title} · {kind === "attack" ? `Attack ${label}` : "Damage"}
            </strong>
            {kind === "attack" && (
              <RollModeControl
                label={title}
                value={mode}
                onChange={setMode}
                disabled={disabled || roll.busy || !!retry}
              />
            )}
          </div>
          {!roll.result && (
            <div>
              {kind === "attack"
                ? `${mode === "normal" ? "1d20" : mode === "fortune" ? "2d20 (highest)" : "2d20 (lowest)"} ${label}`
                : label}
            </div>
          )}
          {roll.result?.attack && (
            <RollResult outcome={roll.result.attack} pending={roll.animating} />
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
          {kind === "damage" && options?.damage_unavailable && (
            <Alert type="info" title={options.damage_unavailable} />
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
                (kind === "damage" && options?.damage_available === false)
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
        aria-label={`Roll ${actionLabel}`}
      >
        {label}
      </Button>
    </Popover>
  );
}

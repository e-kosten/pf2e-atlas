import { Typography } from "antd";
import { useId } from "react";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import type { EncounterAbilityStateView } from "../../generated/atlas";
import type { RecordReferenceHandler } from "../../shared/records/PreparedContent";
import {
  EncounterAbilityControls,
  type AbilityStateHandler,
} from "./EncounterAbilityControls";

export function EncounterAbilityStates({
  participantKey,
  abilities,
  disabled,
  onChange,
  onReference,
}: {
  participantKey: string;
  abilities?: EncounterAbilityStateView[];
  disabled: boolean;
  onChange?: AbilityStateHandler;
  onReference: RecordReferenceHandler;
}) {
  const headingId = useId();
  const configurable = abilities?.filter(
    ({ state }) =>
      state.kind === "option" && (!state.always_active || state.choices.length > 0),
  );
  if (!configurable?.length) return null;
  return (
    <section className="encounter-ability-states" aria-labelledby={headingId}>
      <h3 id={headingId}>Ability states</h3>
      {configurable.map((ability) => (
        <div
          key={JSON.stringify([
            participantKey,
            ability.navigation.embedded_path,
            ability.rule_ordinal,
          ])}
          className="encounter-ability-states__row"
        >
          <Typography.Link
            className="encounter-ability-states__name"
            href={destinationHref(
              recordDestination({
                recordKey: ability.navigation.record_key,
                selection: ability.navigation,
              }),
            )}
            onClick={(event) => {
              if (!isPrimaryActivation(event)) return;
              event.preventDefault();
              onReference(
                ability.navigation.record_key,
                event.currentTarget,
                ability.navigation,
              );
            }}
          >
            {ability.label}
          </Typography.Link>
          <EncounterAbilityControls
            ability={ability}
            disabled={disabled}
            onChange={onChange}
          />
        </div>
      ))}
    </section>
  );
}

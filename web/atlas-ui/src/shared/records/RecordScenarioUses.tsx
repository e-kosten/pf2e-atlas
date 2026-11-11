import { Fragment } from "react";
import type { CreatureScenarioUseView } from "../../generated/atlas";

export function RecordScenarioUses({
  uses,
  qualifiers,
}: {
  uses: CreatureScenarioUseView[];
  qualifiers?: string[];
}) {
  return (
    <span className="record-scenario-uses">
      {qualifiers?.map((qualifier) => (
        <Fragment key={qualifier}>
          <span className="record-scenario-uses__qualifier">{qualifier}</span>{" "}
        </Fragment>
      ))}
      {uses.map(({ id, label }) => (
        <Fragment key={id}>
          <span>{label}</span>{" "}
        </Fragment>
      ))}
    </span>
  );
}

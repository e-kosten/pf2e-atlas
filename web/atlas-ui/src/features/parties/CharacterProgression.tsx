import { Switch } from "antd";
import { useId, useState } from "react";
import type {
  SheetFactView,
  SheetGroupView,
  SheetRowView,
} from "../../generated/atlas";
import { SheetFacts, SheetLabel, type SheetPresentation } from "./CharacterSheetParts";

function isSourceFact(fact: SheetFactView) {
  return fact.role === undefined;
}

export function CharacterProgression({
  groups,
  ...presentation
}: SheetPresentation & { groups: SheetGroupView[] }) {
  const levels = groups.filter((group) => group.kind === "advancement");
  const supporting = groups.filter((group) => group.kind !== "advancement");
  const [showSources, setShowSources] = useState(false);
  const switchId = useId();
  const hasSources = levels.some((group) =>
    group.rows.some((row) => row.facts.some(isSourceFact)),
  );
  return (
    <div className="character-progression">
      {levels.length > 0 && (
        <div className="character-progression__advancement">
          {hasSources && (
            <label className="character-progression__source-control" htmlFor={switchId}>
              <Switch id={switchId} checked={showSources} onChange={setShowSources} />
              Show feat sources
            </label>
          )}
          <ol className="character-progression__levels">
            {levels.map((group, index) => (
              <li key={index}>
                <section className="character-progression__level">
                  <h3>{group.label}</h3>
                  <ul className="character-progression__choices">
                    {group.rows.map((row, index) => (
                      <FeatRow
                        key={index}
                        row={row}
                        showSources={showSources}
                        {...presentation}
                      />
                    ))}
                  </ul>
                </section>
              </li>
            ))}
          </ol>
        </div>
      )}
      {supporting.length > 0 && (
        <div className="character-progression__support">
          {supporting.map((group, index) => (
            <section key={index}>
              <h3>{group.label}</h3>
              <ul className="character-progression__support-rows">
                {group.rows.map((row, index) => (
                  <li key={index}>
                    <span>
                      <SheetLabel row={row} {...presentation} />
                    </span>
                    {row.summary_value !== undefined && (
                      <strong>{row.summary_value}</strong>
                    )}
                    <SheetFacts facts={row.facts} />
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      )}
    </div>
  );
}

function FeatRow({
  row,
  showSources,
  ...presentation
}: SheetPresentation & { row: SheetRowView; showSources: boolean }) {
  const choice = row.facts.find((fact) => fact.role === "choice");
  const category = row.facts.find((fact) => fact.role === "category");
  return (
    <li>
      <div className="character-progression__choice-heading">
        <h4>
          <SheetLabel row={row} {...presentation} />
          {choice && (
            <span className="character-progression__selected"> ({choice.value})</span>
          )}
        </h4>
        {category && (
          <span className="character-progression__category">{category.value}</span>
        )}
      </div>
      {showSources && <SheetFacts facts={row.facts.filter(isSourceFact)} />}
    </li>
  );
}

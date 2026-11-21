import type { SheetGroupView } from "../../generated/atlas";
import { SheetFacts, SheetLabel, type SheetPresentation } from "./CharacterSheetParts";

export function CharacterProgression({
  groups,
  ...presentation
}: SheetPresentation & { groups: SheetGroupView[] }) {
  const levels = groups.filter((group) => group.kind === "advancement");
  const supporting = groups.filter((group) => group.kind !== "advancement");
  return (
    <div className="character-progression">
      {levels.length > 0 && (
        <ol className="character-progression__levels">
          {levels.map((group, index) => (
            <li key={index}>
              <section className="character-progression__level">
                <h3>{group.label}</h3>
                <ul className="character-progression__choices">
                  {group.rows.map((row, index) => (
                    <li key={index}>
                      <div className="character-progression__choice-heading">
                        <h4>
                          <SheetLabel row={row} {...presentation} />
                        </h4>
                        <SheetFacts
                          facts={row.facts.filter((fact) => fact.role === "category")}
                          className="character-progression__category"
                        />
                      </div>
                      <SheetFacts
                        facts={row.facts.filter((fact) => fact.role === "choice")}
                        className="character-progression__selected"
                      />
                      <SheetFacts
                        facts={row.facts.filter(
                          (fact) =>
                            fact.role !== "acquired_level" &&
                            fact.role !== "category" &&
                            fact.role !== "choice",
                        )}
                      />
                    </li>
                  ))}
                </ul>
              </section>
            </li>
          ))}
        </ol>
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

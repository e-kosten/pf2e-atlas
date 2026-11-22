import { Fragment } from "react";
import type { SheetGroupView, SheetRowView } from "../../generated/atlas";
import { SheetFacts, SheetLabel, type SheetPresentation } from "./CharacterSheetParts";

export function CharacterInventory({
  groups,
  ...presentation
}: SheetPresentation & { groups: SheetGroupView[] }) {
  const coins = groups.filter((group) => group.kind === "coins");
  const loadout = groups.filter(
    (group) => group.kind === "weapons" || group.kind === "armor",
  );
  const storage = groups.filter(
    (group) =>
      !coins.includes(group) && !loadout.includes(group) && group.kind !== "formulas",
  );
  const formulas = groups.filter((group) => group.kind === "formulas");
  return (
    <div className="character-inventory">
      {loadout.length > 0 && (
        <div className="character-inventory__loadout">
          {loadout.map((group) => (
            <section key={group.kind} className="character-inventory__section">
              <h3>{group.label}</h3>
              <ul className="character-inventory__equipment">
                {group.rows.map((row, index) => (
                  <EquipmentRow key={index} row={row} {...presentation} />
                ))}
              </ul>
            </section>
          ))}
        </div>
      )}
      {coins.map((group, index) => (
        <p className="character-inventory__coins" key={index}>
          <strong>{group.label}</strong>{" "}
          {group.rows.map((row, index) => (
            <Fragment key={row.label}>
              {index > 0 && ", "}
              <span>
                {row.summary_value} {row.label}
              </span>
            </Fragment>
          ))}
        </p>
      ))}
      {storage.length > 0 && (
        <div className="character-inventory__storage">
          {storage.map((group, index) => (
            <GearGroup
              key={
                group.container_id === undefined
                  ? `group-${index}`
                  : `container-${group.container_id}`
              }
              group={group}
              {...presentation}
            />
          ))}
        </div>
      )}
      {formulas.length > 0 && (
        <div className="character-inventory__formulas">
          {formulas.map((group, index) => (
            <GearGroup key={index} group={group} {...presentation} />
          ))}
        </div>
      )}
    </div>
  );
}

function EquipmentRow({
  row,
  ...presentation
}: SheetPresentation & { row: SheetRowView }) {
  const quantity = row.facts.filter((fact) => fact.role === "quantity");
  const primary = row.facts.filter((fact) => fact.role === "primary");
  const supporting = row.facts.filter(
    (fact) => fact.role !== "quantity" && fact.role !== "primary",
  );
  return (
    <li>
      <div className="character-inventory__item-heading">
        <h4>
          <SheetLabel row={row} {...presentation} />
        </h4>
        <SheetFacts facts={quantity} />
      </div>
      <SheetFacts facts={primary} className="character-inventory__readouts" />
      <SheetFacts facts={supporting} className="character-inventory__supporting" />
    </li>
  );
}

function GearGroup({
  group,
  ...presentation
}: SheetPresentation & { group: SheetGroupView }) {
  const metadata = group.rows.filter((row) => row.kind === "container_metadata");
  const items = group.rows.filter((row) => row.kind !== "container_metadata");
  const formulas = group.kind === "formulas";
  return (
    <section className="character-inventory__section">
      <h3>{group.label}</h3>
      {metadata.map((row, index) => (
        <SheetFacts
          key={index}
          facts={row.facts}
          className="character-inventory__container-facts"
        />
      ))}
      {items.length > 0 && (
        <>
          {!formulas && (
            <div className="character-inventory__gear-labels" aria-hidden="true">
              <span>Item</span>
              <span>Quantity</span>
              <span>State</span>
            </div>
          )}
          <ul
            className={`character-inventory__gear${formulas ? " character-inventory__gear--formulas" : ""}`}
          >
            {items.map((row, index) => (
              <li key={index}>
                <strong>
                  <SheetLabel row={row} {...presentation} />
                </strong>
                <SheetFacts
                  facts={row.facts.filter((fact) => fact.role === "quantity")}
                  className="character-inventory__quantity"
                />
                <SheetFacts
                  facts={row.facts.filter((fact) => fact.role !== "quantity")}
                  className="character-inventory__state"
                />
              </li>
            ))}
          </ul>
        </>
      )}
    </section>
  );
}

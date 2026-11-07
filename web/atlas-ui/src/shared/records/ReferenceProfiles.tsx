import { Fragment } from "react";
import { ReferenceValue, ReferenceValues } from "./ReferenceHelp";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Table, Typography } from "antd";
import { Children, cloneElement, isValidElement, type ReactNode } from "react";
import type {
  RecordBodyView,
  ClassReferenceView,
  FactView,
  NumberFactView,
  ReferenceGrantView,
  AttributeChoiceView,
} from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
import { FactRow, NumberFact, scalarFactText } from "./presentationFacts";
import { renderIwr } from "./ActorReference";
import "../../styles/ui/reference-profiles.css";
export function renderReferenceFact<T>({
  label,
  fact,
  format = scalarFactText,
  unit = "",
  optional = false,
  emptyLabel,
  renderValue,
}: {
  label: string;
  fact: FactView<T> | NumberFactView;
  format?: (value: T) => string;
  unit?: string;
  optional?: boolean;
  emptyLabel?: string;
  renderValue?: (value: T) => ReactNode;
}) {
  if (
    fact.state === "not_applicable" ||
    (optional && ["missing", "null"].includes(fact.state))
  )
    return null;
  const value = fact.value;
  if (
    fact.state === "value" &&
    (value === "" || (Array.isArray(value) && value.length === 0))
  )
    return emptyLabel ? <FactRow label={label}>{emptyLabel}</FactRow> : null;
  return (
    <FactRow label={label}>
      {fact.state === "value" && value !== null ? (
        "adjustment" in fact ? (
          <NumberFact
            fact={fact}
            label={label}
            format={(number) => format(number as T)}
            unit={unit}
          />
        ) : (
          <>{renderValue ? renderValue(value as T) : `${format(value as T)}${unit}`}</>
        )
      ) : (
        <span className="muted">Unavailable</span>
      )}
    </FactRow>
  );
}
export const ReferenceFact = renderReferenceFact;

const alternatives = (values: unknown[]) => values.map(scalarFactText).join(" or ");
const choices = (slots: AttributeChoiceView[]) =>
  slots.map((slot, index) => (
    <Fragment key={index}>
      {index > 0 && ", "}
      {slot.choices.state !== "value" || !slot.choices.value ? (
        "Unavailable"
      ) : slot.free_choice ? (
        "Free"
      ) : (
        <ReferenceValues values={slot.choices.value} separator=" or " />
      )}
    </Fragment>
  ));
export function referenceRows(rows: ReactNode[]) {
  return Children.toArray(rows).map((row) =>
    isValidElement(row) ? cloneElement(row, { key: row.key }) : row,
  );
}

function renderFactGroup(rows: ReactNode[], stacked = false) {
  const visibleRows = referenceRows(rows);
  return visibleRows.length ? (
    <div className={stacked ? "reference-profile__groups" : "reference-profile__facts"}>
      {visibleRows}
    </div>
  ) : null;
}

function Group({
  title,
  rows,
  stacked,
}: {
  title?: string;
  rows: ReactNode[];
  stacked?: boolean;
}) {
  const content = renderFactGroup(rows, stacked);
  return content ? (
    <section className="reference-profile__section">
      {title && <h3>{title}</h3>}
      {content}
    </section>
  ) : null;
}
function Grants({
  fact,
  onReference,
  spells = false,
  showLevels = true,
}: {
  fact: FactView<ReferenceGrantView[]>;
  onReference: RecordReferenceHandler;
  spells?: boolean;
  showLevels?: boolean;
}) {
  if (fact.state !== "value")
    return (
      <ReferenceFact
        label={spells ? "Granted spells" : "Granted features"}
        fact={fact}
        optional
      />
    );
  if (!fact.value?.length) return null;
  const grants = showLevels
    ? [...fact.value].sort(
        (a, b) =>
          (a.level.value ?? Number.MAX_SAFE_INTEGER) -
          (b.level.value ?? Number.MAX_SAFE_INTEGER),
      )
    : fact.value;
  return (
    <section className="reference-profile__section">
      <h3>{spells ? "Granted spells" : "Granted features"}</h3>
      <ul className="reference-profile__grants">
        {grants.map((grant, index) => (
          <li key={`${grant.target.value}-${index}`}>
            {showLevels && (
              <span className="reference-profile__grant-level">
                {spells ? "Rank" : "Level"}{" "}
                {grant.level.state === "value" ? grant.level.value : "unavailable"}
              </span>
            )}
            {grant.navigation ? (
              <Typography.Link
                href={destinationHref(
                  recordDestination({
                    recordKey: grant.navigation.record_key,
                    selection: grant.navigation,
                  }),
                )}
                onClick={(event) => {
                  if (!isPrimaryActivation(event)) return;
                  event.preventDefault();
                  onReference(
                    grant.navigation!.record_key,
                    event.currentTarget,
                    grant.navigation!,
                  );
                }}
              >
                {grant.name.value ?? "Unnamed reference"}
              </Typography.Link>
            ) : (
              <span>{grant.name.value ?? "Reference unavailable"}</span>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}
function ClassAdvancement({ value }: { value: ClassReferenceView }) {
  const groups = [
    ["Ancestry feat", value.ancestry_feat_levels],
    ["Class feat", value.class_feat_levels],
    ["General feat", value.general_feat_levels],
    ["Skill feat", value.skill_feat_levels],
    ["Skill increase", value.skill_increase_levels],
  ] as const;
  const levels = [
    ...new Set(
      groups.flatMap(([, fact]) => (fact.state === "value" ? (fact.value ?? []) : [])),
    ),
  ].sort((a, b) => a - b);
  if (
    !levels.length &&
    groups.every(
      ([, fact]) => fact.state === "value" || fact.state === "not_applicable",
    )
  )
    return null;
  return (
    <section className="reference-profile__section">
      <h3>Feat and skill advancement</h3>
      {levels.length > 0 && (
        <Table
          aria-label="Feat and skill advancement"
          pagination={false}
          size="small"
          rowKey="level"
          dataSource={levels.map((level) => ({
            level,
            advances: groups
              .filter(
                ([, fact]) => fact.state === "value" && fact.value?.includes(level),
              )
              .map(([label]) => label),
          }))}
          columns={[
            {
              title: "Level",
              dataIndex: "level",
              width: 72,
              rowScope: "row",
              onCell: () => ({ style: { verticalAlign: "top" } }),
            },
            {
              title: "Advancement",
              dataIndex: "advances",
              render: (advances: string[]) => advances.join(", "),
            },
          ]}
        />
      )}
      {groups
        .filter(([, fact]) => fact.state !== "value")
        .map(([label, fact]) => (
          <ReferenceFact key={label} label={label} fact={fact} />
        ))}
    </section>
  );
}
export function ReferenceProfiles({
  body,
  onReference,
  children,
}: {
  body: RecordBodyView;
  onReference: RecordReferenceHandler;
  children: ReactNode;
}) {
  if (body.kind === "heritage") {
    const ancestry = renderReferenceFact({
      label: "Ancestry",
      fact: body.value.ancestry,
      optional: true,
    });
    return ancestry || children ? (
      <div className="reference-profile">
        {ancestry}
        {children}
      </div>
    ) : null;
  }
  if (body.kind === "ancestry") {
    const v = body.value;
    return (
      <div className="reference-profile">
        <Group
          rows={[
            renderReferenceFact({ label: "HP", fact: v.hit_points }),
            renderReferenceFact({ label: "Speed", fact: v.speed, unit: " feet" }),
            renderReferenceFact({
              label: "Size",
              fact: v.size,
              renderValue: (value) => <ReferenceValue value={value} />,
            }),
            renderReferenceFact({
              label: "Vision",
              fact: v.vision,
              renderValue: (value) => <ReferenceValue value={value} />,
            }),
            renderReferenceFact({ label: "Hands", fact: v.hands, optional: true }),
            renderReferenceFact({
              label: "Reach",
              fact: v.reach,
              unit: " feet",
              optional: true,
            }),
          ]}
        />
        <Group
          rows={[
            renderReferenceFact({
              label: "Attribute boosts",
              fact: v.boosts,
              renderValue: choices,
            }),
            renderReferenceFact({
              label: "Attribute flaws",
              fact: v.flaws,
              renderValue: choices,
            }),
          ]}
        />
        <Group
          rows={[
            renderReferenceFact({
              label: "Languages",
              fact: v.languages,
              renderValue: (values) => <ReferenceValues values={values} />,
              emptyLabel: "None listed",
            }),
            renderReferenceFact({
              label: "Languages",
              fact: v.language_details,
              optional: true,
            }),
            renderReferenceFact({
              label: "Additional language choices",
              fact: v.additional_languages,
              renderValue: (values) => <ReferenceValues values={values} />,
            }),
            renderReferenceFact({
              label: "Additional languages",
              fact: v.additional_language_details,
              optional: true,
            }),
            v.additional_language_count.value !== 0
              ? renderReferenceFact({
                  label: "Additional language slots",
                  fact: v.additional_language_count,
                  optional: true,
                })
              : null,
          ]}
        />
        {children}
        <Grants fact={v.grants} onReference={onReference} showLevels={false} />
      </div>
    );
  }
  if (body.kind === "class") {
    const v = body.value;
    return (
      <div className="reference-profile">
        <Group
          rows={[
            renderReferenceFact({ label: "Class HP", fact: v.hit_points }),
            renderReferenceFact({
              label: "Key attribute",
              fact: v.key_attributes,
              renderValue: (values) => (
                <ReferenceValues values={values} separator=" or " />
              ),
            }),
          ]}
        />
        {children}
        <Group
          title="Initial proficiencies"
          stacked
          rows={[
            ...(
              [
                [
                  ["Perception", v.perception],
                  ["Fortitude", v.fortitude],
                  ["Reflex", v.reflex],
                  ["Will", v.will],
                ],
                [
                  ["Simple weapons", v.simple_weapons],
                  ["Martial weapons", v.martial_weapons],
                  ["Advanced weapons", v.advanced_weapons],
                  ["Unarmed attacks", v.unarmed],
                ],
                [
                  ["Unarmored defense", v.unarmored],
                  ["Light armor", v.light_armor],
                  ["Medium armor", v.medium_armor],
                  ["Heavy armor", v.heavy_armor],
                ],
              ] as const
            ).map((group) =>
              renderFactGroup(
                group.map(([label, fact]) => renderReferenceFact({ label, fact })),
              ),
            ),
            v.other_weapons.value
              ? renderReferenceFact({
                  label: v.other_weapons.value,
                  fact: v.other_weapon_rank,
                })
              : null,
            renderFactGroup([
              renderReferenceFact({ label: "Spellcasting", fact: v.spellcasting }),
              renderReferenceFact({
                label: "Listed trained skills",
                fact: v.trained_skills,
                renderValue: (values) => <ReferenceValues values={values} />,
              }),
              renderReferenceFact({
                label: "Additional skills",
                fact: v.additional_skills,
                unit: " + Intelligence modifier",
              }),
            ]),
          ]}
        />
        <Grants fact={v.grants} onReference={onReference} />
        <ClassAdvancement value={v} />
      </div>
    );
  }
  if (body.kind === "deity") {
    const v = body.value;
    return (
      <div className="reference-profile">
        <Group
          title="Devotee benefits"
          rows={[
            renderReferenceFact({
              label: "Divine attributes",
              fact: v.attributes,
              renderValue: (values) => (
                <ReferenceValues values={values} separator=" or " />
              ),
            }),
            renderReferenceFact({
              label: "Divine font",
              fact: v.fonts,
              format: alternatives,
              emptyLabel: "None listed",
            }),
            renderReferenceFact({
              label: "Divine skill",
              fact: v.skills,
              renderValue: (values) => <ReferenceValues values={values} />,
            }),
            renderReferenceFact({
              label: "Favored weapon",
              fact: v.favored_weapons,
              format: alternatives,
            }),
            renderReferenceFact({
              label: "Domains",
              fact: v.primary_domains,
              renderValue: (values) => <ReferenceValues values={values} />,
            }),
            renderReferenceFact({
              label: "Alternate domains",
              fact: v.alternate_domains,
              renderValue: (values) => <ReferenceValues values={values} />,
            }),
            renderReferenceFact({
              label: "Sanctification",
              fact: v.sanctification,
              optional: true,
              format: (s) =>
                `${s.requirement.value === "must" ? "Must choose" : s.requirement.value === "can" ? "Can choose" : "Requirement unavailable"} ${s.choices.value?.join(" or ") ?? "Unavailable"}`,
            }),
          ]}
        />
        <Grants fact={v.spells} onReference={onReference} spells />
        {children}
      </div>
    );
  }
  if (body.kind === "vehicle") {
    const v = body.value;
    return (
      <div className="reference-profile">
        <Group
          title="Operation"
          rows={[
            renderReferenceFact({ label: "Crew", fact: v.crew }),
            renderReferenceFact({ label: "Passengers", fact: v.passengers }),
            renderReferenceFact({ label: "Speed", fact: v.speed }),
            renderReferenceFact({ label: "Piloting", fact: v.piloting }),
          ]}
        />
        {children}
        <Group
          title="Defenses"
          stacked
          rows={[
            renderFactGroup([
              renderReferenceFact({ label: "AC", fact: v.defenses.armor_class }),
              renderReferenceFact({ label: "HP", fact: v.defenses.hit_points }),
              renderReferenceFact({ label: "Hardness", fact: v.defenses.hardness }),
            ]),
            renderReferenceFact({
              label: "Fortitude",
              fact: v.fortitude,
              format: (n) => `${Number(n) >= 0 ? "+" : ""}${n}`,
            }),
            renderIwr({ label: "Immunities", fact: v.defenses.immunities }),
            renderIwr({ label: "Weaknesses", fact: v.defenses.weaknesses }),
            renderIwr({ label: "Resistances", fact: v.defenses.resistances }),
          ]}
        />
        <Group
          title="Collision"
          rows={[
            renderReferenceFact({ label: "Damage", fact: v.collision_damage }),
            renderReferenceFact({ label: "DC", fact: v.collision_dc }),
          ]}
        />
        <Group
          rows={[
            renderReferenceFact({ label: "Price", fact: v.price_gp, unit: " gp" }),
            renderReferenceFact({
              label: "Size",
              fact: v.size,
              renderValue: (value) => <ReferenceValue value={value} />,
            }),
            renderReferenceFact({ label: "Length", fact: v.length, unit: " feet" }),
            renderReferenceFact({ label: "Width", fact: v.width, unit: " feet" }),
            renderReferenceFact({ label: "Height", fact: v.height, unit: " feet" }),
          ]}
        />
      </div>
    );
  }
  if (body.kind === "army") {
    const v = body.value;
    return (
      <div className="reference-profile">
        <Group
          rows={[renderReferenceFact({ label: "Army type", fact: v.army_type })]}
        />
        {children}
        <Group
          title="Defenses"
          rows={[
            ...(
              [
                ["AC", v.armor_class],
                ["HP", v.hit_points],
                ["Rout threshold", v.rout_threshold],
                ["Maneuver", v.maneuver],
                ["Morale", v.morale],
              ] as const
            ).map(([label, fact]) => renderReferenceFact({ label, fact })),
            renderIwr({ label: "Immunities", fact: v.immunities }),
            renderIwr({ label: "Weaknesses", fact: v.weaknesses }),
            renderIwr({ label: "Resistances", fact: v.resistances }),
          ]}
        />
        <Group
          title="Scouting and logistics"
          rows={[
            ...(
              [
                ["Scouting", v.scouting],
                ["Consumption", v.consumption],
                ["Recruitment DC", v.recruitment_dc],
                ["Starting ammunition", v.ammunition],
                ["Starting potions", v.potions],
              ] as const
            ).map(([label, fact]) => renderReferenceFact({ label, fact })),
            renderReferenceFact({
              label: "Melee weapon",
              fact: v.melee_weapon,
              optional: true,
            }),
            renderReferenceFact({
              label: "Ranged weapon",
              fact: v.ranged_weapon,
              optional: true,
            }),
          ]}
        />
      </div>
    );
  }
  return children;
}

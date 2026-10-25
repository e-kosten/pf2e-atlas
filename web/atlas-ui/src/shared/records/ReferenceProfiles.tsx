import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { Table, Typography } from "antd";
import type { ReactNode } from "react";
import type {
  RecordBodyView,
  ClassReferenceView,
  FactView,
  NumberFactView,
  ReferenceGrantView,
  AttributeChoiceView,
} from "../../generated/atlas";
import type { RecordReferenceHandler } from "./PreparedContent";
import { FactRow, NumberFact } from "./presentationFacts";
import { Iwr } from "./ActorReference";
import "../../styles/ui/reference-profiles.css";
export function ReferenceFact<T>({
  label,
  fact,
  format = String,
  unit = "",
  optional = false,
  emptyLabel,
}: {
  label: string;
  fact: FactView<T> | NumberFactView;
  format?: (value: T) => string;
  unit?: string;
  optional?: boolean;
  emptyLabel?: string;
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
          `${format(value as T)}${unit}`
        )
      ) : (
        <span className="muted">Unavailable</span>
      )}
    </FactRow>
  );
}
const list = (values: string[]) => values.join(", ");
const alternatives = (values: string[]) => values.join(" or ");
const choices = (slots: AttributeChoiceView[]) =>
  slots
    .map((slot) =>
      slot.choices.state !== "value" || !slot.choices.value
        ? "Unavailable"
        : slot.free_choice
          ? "Free"
          : alternatives(slot.choices.value),
    )
    .join(", ");
function Group({
  title,
  children,
  stacked = false,
}: {
  title?: string;
  children: ReactNode;
  stacked?: boolean;
}) {
  return (
    <section className="reference-profile__section">
      {title && <h3>{title}</h3>}
      <div
        className={stacked ? "reference-profile__groups" : "reference-profile__facts"}
      >
        {children}
      </div>
    </section>
  );
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
  if (body.kind === "heritage")
    return (
      <div className="reference-profile">
        <ReferenceFact label="Ancestry" fact={body.value.ancestry} optional />
        {children}
      </div>
    );
  if (body.kind === "ancestry") {
    const v = body.value;
    return (
      <div className="reference-profile">
        <Group>
          <ReferenceFact label="HP" fact={v.hit_points} />
          <ReferenceFact label="Speed" fact={v.speed} unit=" feet" />
          <ReferenceFact label="Size" fact={v.size} />
          <ReferenceFact label="Vision" fact={v.vision} />
          <ReferenceFact label="Hands" fact={v.hands} optional />
          <ReferenceFact label="Reach" fact={v.reach} unit=" feet" optional />
        </Group>
        <Group>
          <ReferenceFact label="Attribute boosts" fact={v.boosts} format={choices} />
          <ReferenceFact label="Attribute flaws" fact={v.flaws} format={choices} />
        </Group>
        <Group>
          <ReferenceFact
            label="Languages"
            fact={v.languages}
            format={list}
            emptyLabel="None listed"
          />
          <ReferenceFact label="Languages" fact={v.language_details} optional />
          <ReferenceFact
            label="Additional language choices"
            fact={v.additional_languages}
            format={list}
          />
          <ReferenceFact
            label="Additional languages"
            fact={v.additional_language_details}
            optional
          />
          {v.additional_language_count.value !== 0 && (
            <ReferenceFact
              label="Additional language slots"
              fact={v.additional_language_count}
              optional
            />
          )}
        </Group>
        {children}
        <Grants fact={v.grants} onReference={onReference} showLevels={false} />
      </div>
    );
  }
  if (body.kind === "class") {
    const v = body.value;
    return (
      <div className="reference-profile">
        <Group>
          <ReferenceFact label="Class HP" fact={v.hit_points} />
          <ReferenceFact
            label="Key attribute"
            fact={v.key_attributes}
            format={alternatives}
          />
        </Group>
        {children}
        <section className="reference-profile__section">
          <h3>Initial proficiencies</h3>
          <div className="reference-profile__groups">
            {(
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
            ).map((group, index) => (
              <div className="reference-profile__facts" key={index}>
                {group.map(([label, fact]) => (
                  <ReferenceFact key={label} label={label} fact={fact} />
                ))}
              </div>
            ))}
            {v.other_weapons.value && (
              <ReferenceFact label={v.other_weapons.value} fact={v.other_weapon_rank} />
            )}
            <div className="reference-profile__facts">
              <ReferenceFact label="Spellcasting" fact={v.spellcasting} />
              <ReferenceFact
                label="Listed trained skills"
                fact={v.trained_skills}
                format={list}
              />
              <ReferenceFact
                label="Additional skills"
                fact={v.additional_skills}
                unit=" + Intelligence modifier"
              />
            </div>
          </div>
        </section>
        <Grants fact={v.grants} onReference={onReference} />
        <ClassAdvancement value={v} />
      </div>
    );
  }
  if (body.kind === "deity") {
    const v = body.value;
    return (
      <div className="reference-profile">
        <Group title="Devotee benefits">
          <ReferenceFact
            label="Divine attributes"
            fact={v.attributes}
            format={alternatives}
          />
          <ReferenceFact
            label="Divine font"
            fact={v.fonts}
            format={alternatives}
            emptyLabel="None listed"
          />
          <ReferenceFact label="Divine skill" fact={v.skills} format={list} />
          <ReferenceFact
            label="Favored weapon"
            fact={v.favored_weapons}
            format={alternatives}
          />
          <ReferenceFact label="Domains" fact={v.primary_domains} format={list} />
          <ReferenceFact
            label="Alternate domains"
            fact={v.alternate_domains}
            format={list}
          />
          <ReferenceFact
            label="Sanctification"
            fact={v.sanctification}
            optional
            format={(s) =>
              `${s.requirement.value === "must" ? "Must choose" : s.requirement.value === "can" ? "Can choose" : "Requirement unavailable"} ${s.choices.value?.join(" or ") ?? "Unavailable"}`
            }
          />
        </Group>
        <Grants fact={v.spells} onReference={onReference} spells />
        {children}
      </div>
    );
  }
  if (body.kind === "vehicle") {
    const v = body.value;
    return (
      <div className="reference-profile">
        <Group title="Operation">
          <ReferenceFact label="Crew" fact={v.crew} />
          <ReferenceFact label="Passengers" fact={v.passengers} />
          <ReferenceFact label="Speed" fact={v.speed} />
          <ReferenceFact label="Piloting" fact={v.piloting} />
        </Group>
        {children}
        <Group title="Defenses" stacked>
          <div className="reference-profile__facts">
            <ReferenceFact label="AC" fact={v.defenses.armor_class} />

            <ReferenceFact label="HP" fact={v.defenses.hit_points} />
            <ReferenceFact label="Hardness" fact={v.defenses.hardness} />
          </div>
          <ReferenceFact
            label="Fortitude"
            fact={v.fortitude}
            format={(n) => `${Number(n) >= 0 ? "+" : ""}${n}`}
          />
          <Iwr label="Immunities" fact={v.defenses.immunities} />
          <Iwr label="Weaknesses" fact={v.defenses.weaknesses} />
          <Iwr label="Resistances" fact={v.defenses.resistances} />
        </Group>
        <Group title="Collision">
          <ReferenceFact label="Damage" fact={v.collision_damage} />
          <ReferenceFact label="DC" fact={v.collision_dc} />
        </Group>
        <Group>
          <ReferenceFact label="Price" fact={v.price_gp} unit=" gp" />
          <ReferenceFact label="Size" fact={v.size} />
          <ReferenceFact label="Length" fact={v.length} unit=" feet" />
          <ReferenceFact label="Width" fact={v.width} unit=" feet" />
          <ReferenceFact label="Height" fact={v.height} unit=" feet" />
        </Group>
      </div>
    );
  }
  if (body.kind === "army") {
    const v = body.value;
    return (
      <div className="reference-profile">
        <Group>
          <ReferenceFact label="Army type" fact={v.army_type} />
        </Group>
        {children}
        <Group title="Defenses">
          {(
            [
              ["AC", v.armor_class],
              ["HP", v.hit_points],
              ["Rout threshold", v.rout_threshold],
              ["Maneuver", v.maneuver],
              ["Morale", v.morale],
            ] as const
          ).map(([label, fact]) => (
            <ReferenceFact key={label} label={label} fact={fact} />
          ))}
          <Iwr label="Immunities" fact={v.immunities} />
          <Iwr label="Weaknesses" fact={v.weaknesses} />
          <Iwr label="Resistances" fact={v.resistances} />
        </Group>
        <Group title="Scouting and logistics">
          {(
            [
              ["Scouting", v.scouting],
              ["Consumption", v.consumption],
              ["Recruitment DC", v.recruitment_dc],
              ["Starting ammunition", v.ammunition],
              ["Starting potions", v.potions],
            ] as const
          ).map(([label, fact]) => (
            <ReferenceFact key={label} label={label} fact={fact} />
          ))}
          <ReferenceFact label="Melee weapon" fact={v.melee_weapon} optional />
          <ReferenceFact label="Ranged weapon" fact={v.ranged_weapon} optional />
        </Group>
      </div>
    );
  }
  return children;
}

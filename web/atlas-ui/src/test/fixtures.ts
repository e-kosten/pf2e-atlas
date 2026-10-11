import type {
  RecordDetailView,
  RecordSummaryView,
  FilterEditorView,
  QueryFieldDefinition,
  QueryPredicate,
  FilterValueListView,
  ActorPresentationView,
  ActorActivityView,
  FactView,
  NumberFactView,
  SpellPresentationView,
} from "../generated/atlas";
export const knownFact = <T>(value: T): FactView<T> => ({ state: "value", value });
export const unavailableFact = <T>(
  state: FactView<T>["state"] = "missing",
): FactView<T> => ({ state, value: null });
export const numberFact = (value: number): NumberFactView => ({
  state: "value",
  value,
  adjustment: null,
});
export function actorFixture(): ActorPresentationView {
  return {
    level: numberFact(1),
    armor_class: numberFact(16),
    maximum_hp: numberFact(20),
    perception: numberFact(7),
    saves: { fortitude: numberFact(7), reflex: numberFact(5), will: numberFact(0) },
    abilities: knownFact([{ key: "str", label: "Strength", modifier: numberFact(3) }]),
    skills: knownFact([]),
    land_speed: numberFact(25),
    movement: knownFact([]),
    immunities: knownFact([]),
    weaknesses: knownFact([]),
    resistances: knownFact([]),
    senses: knownFact([]),
    perception_details: knownFact(""),
    languages: knownFact(["Common"]),
    language_details: knownFact(""),
    activities: knownFact([]),
    runtime: null,
  };
}
export function activityFixture(
  title = "Jaws",
  recordKey = "actors:ghoul",
): ActorActivityView {
  const na: NumberFactView = { state: "not_applicable", value: null, adjustment: null };
  return {
    title,
    family: "melee",
    kind: "strike",
    navigation: {
      record_key: recordKey,
      owners: [{ collection: "items", identity: { SnapshotLocal: { index: 0 } } }],
      field: null,
      passage: null,
      source_fingerprint: "snapshot",
    },
    usage: "1 action",
    traits: [],
    attack: numberFact(9),
    lore_modifier: na,
    difficulty_class: na,
    casting_tradition: unavailableFact("not_applicable"),
    preparation: unavailableFact("not_applicable"),
    damage: knownFact([]),
    casting_entry: null,
    association: unavailableFact("not_applicable"),
    notes: [],
  };
}
export function spellFixture(): SpellPresentationView {
  return {
    rank: numberFact(3),
    traditions: knownFact(["arcane", "primal"]),
    cast: knownFact("2 actions"),
    requirements: knownFact(""),
    cost: knownFact(""),
    range: knownFact("500 feet"),
    target: knownFact(""),
    area: knownFact({
      shape: knownFact("burst"),
      size: knownFact("20 feet"),
      details: knownFact(""),
    }),
    defense: knownFact({
      statistic: knownFact("Reflex"),
      basic: knownFact(true),
      passive: knownFact(""),
    }),
    duration: knownFact(""),
    sustained: knownFact(false),
    damage: knownFact([]),
    heightening: unavailableFact("not_applicable"),
    forms: knownFact([]),
    casting_entry: unavailableFact("not_applicable"),
    authored_cast_rank: { state: "not_applicable", value: null, adjustment: null },
  };
}
export function summaryFixture(
  record_key: string,
  title = record_key.split(":")[1] || record_key,
): RecordSummaryView {
  return {
    record_key,
    title,
    kind: "spell",
    kind_label: "Spell",
    source_type: "spell",
    level_label: null,
    level_basis: null,
    rarity: null,
    traits: [],
    publication: null,
    pack: null,
  };
}
export function detailFixture(
  record_key: string,
  title?: string,
  target?: string,
): RecordDetailView {
  const record = summaryFixture(record_key, title);
  const locator = { record: record_key, owners: [], field: "system.description.value" };
  return {
    record,
    selected: {
      record_key,
      owners: [],
      field: null,
      passage: null,
      source_fingerprint: null,
    },
    relationships_truncated: false,
    relationships: target
      ? [
          {
            source: locator,
            ordinal: 0,
            origin: "prepared_content",
            kind: "uuid",
            authored_target: target,
            occurrence_path: null,
            audiences: [],
            availability: null,
            status: "resolved_root",
            target: {
              record_key: target,
              owners: [],
              field: null,
              passage: null,
              source_fingerprint: null,
            },
            url: null,
          },
        ]
      : [],
    presentation: {
      identity: record,
      owned: [],
      body: { kind: "content" },
      content: target
        ? [
            {
              locator,
              role: "description",
              source_fingerprint: null,
              body: {
                kind: "html",
                html: '<p><a data-atlas-reference="0" href="#">Nested Rule</a></p>',
                controls: [],
              },
            },
          ]
        : [],
    },
  };
}
export function fieldFixture(
  id: string,
  field_type: QueryFieldDefinition["field_type"] = "string",
  label = id,
): QueryFieldDefinition {
  return {
    id,
    path: id,
    scope: null,
    field_type,
    label,
    family_types: [],
    units: null,
    basis: "authored",
    choices: [],
    operators:
      field_type === "number"
        ? ["eq", "gt", "gte", "lt", "lte", "between", "state"]
        : field_type === "set"
          ? ["includes", "includes_all", "includes_any", "excludes_any", "state"]
          : field_type === "collection"
            ? ["exists", "state"]
            : ["eq", "neq", "in", "state"],
    examples: [],
    value_discovery:
      field_type === "number"
        ? "numeric_statistics"
        : field_type === "collection"
          ? "collection_states"
          : "open_values",
  };
}
export function editorFixture(fields: QueryFieldDefinition[] = []): FilterEditorView {
  return {
    catalog_version: 1,
    limits: { source_bytes: 16384, nodes: 256, depth: 32, literal_list: 128 },
    groups: [
      {
        id: "common",
        label: "Common",
        fields: fields.map((definition) => ({
          definition,
          control:
            definition.field_type === "number"
              ? "numeric"
              : definition.field_type === "set"
                ? "set"
                : "text",
          placement: "initially_visible",
        })),
      },
    ],
  };
}
export function valuesFixture(
  field: string,
  values: string[] = [],
): FilterValueListView {
  return {
    values: {
      field,
      options: values.map((value) => ({ value, distinct_roots: 1n, selected: false })),
      total_values: BigInt(values.length),
      exhaustive: true,
      count_basis: "distinct roots",
    },
  };
}
export const truePredicate: QueryPredicate = { kind: "boolean_constant", value: true };

import type {
  AppReadinessView,
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
export function readinessFixture(): AppReadinessView {
  return {
    default_search_profile: "bge-base",
    artifact_generation: "generation-a",
    status: "ready",
    message: "Ready",
    search_profiles: [
      {
        id: "bge-base",
        label: "BGE Base",
        stored_vectors: true,
        assets_ready: true,
        usable: true,
      },
      {
        id: "embeddinggemma",
        label: "EmbeddingGemma",
        stored_vectors: true,
        assets_ready: true,
        usable: true,
      },
    ],
  };
}
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
    casting_ranks: [],
    title,
    family: "melee",
    kind: "strike",
    has_reference_details: true,
    navigation: {
      record_key: recordKey,
      owners: [{ collection: "items", identity: { SnapshotLocal: { index: 0 } } }],
      field: null,
      passage: null,
      source_fingerprint: "snapshot",
    },
    usage: null,
    action_cost: knownFact({ glyph: "one_action", label: "1 action" }),
    traits: [],
    attack: numberFact(9),
    attack_options: [
      { attack_number: 1, label: "Attack", modifier: 9 },
      { attack_number: 2, label: "Second attack", modifier: 4 },
      { attack_number: 3, label: "Third attack", modifier: -1 },
    ],
    lore_modifier: na,
    difficulty_class: na,
    casting_tradition: unavailableFact("not_applicable"),
    preparation: unavailableFact("not_applicable"),
    damage: knownFact([]),
    casting_entry: null,
    association: unavailableFact("not_applicable"),
    base_rank: na,
    authored_cast_rank: na,
    casting_context: null,
    notes: [],
  };
}
export function spellFixture(): SpellPresentationView {
  return {
    base_rank: numberFact(1),
    rank: numberFact(3),
    traditions: knownFact(["arcane", "primal"]),
    cast: knownFact({ glyph: "two_actions", label: "2 actions" }),
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
    artifact_generation: "test-generation",
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
            effect_label: null,
            source_navigation: {
              record_key,
              owners: [],
              field: locator.field,
              passage: null,
              source_fingerprint: null,
            },
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
      classification: record.kind_label,
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
        ? ["eq", "neq", "gt", "gte", "lt", "lte", "between", "state"]
        : field_type === "set"
          ? ["includes", "includes_all", "includes_any", "excludes_any", "state"]
          : field_type === "collection"
            ? ["exists", "state"]
            : field_type === "boolean"
              ? ["eq", "neq", "state"]
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
    catalog_version: 4,
    common_field_ids: ["kind", "level_or_rank", "rarity", "traits"].filter((id) =>
      fields.some((field) => field.id === id),
    ),
    limits: { source_bytes: 16384, nodes: 256, depth: 32, literal_list: 128 },
    groups: [
      {
        id: "common",
        label: "Common",
        fields: fields.map((definition) => ({
          definition,
          label: definition.label,
          help: null,
          choices: definition.choices.map((value) => ({ value, label: value })),
          operators: definition.operators.map((operator) => ({
            operator,
            label: operator,
            operand:
              operator === "exists"
                ? ("predicate" as const)
                : operator === "state"
                  ? ("state" as const)
                  : operator === "between"
                    ? ("range" as const)
                    : ["in", "includes_any", "includes_all", "excludes_any"].includes(
                          operator,
                        )
                      ? ("list" as const)
                      : ("scalar" as const),
          })),
          control:
            definition.field_type === "number"
              ? "numeric"
              : definition.field_type === "set"
                ? "set"
                : "text",
          placement: "initially_visible",
          applicable: true,
          unit_fields: [],
          short_choices:
            definition.choices.length > 0 || definition.field_type === "boolean",
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
    kind: "contextual",
    search_profile: "bge-base",
    artifact_generation: "generation-a",
    outcome: {
      status: "available",
      value: {
        field,
        options: values.map((value) => ({
          value,
          distinct_roots: 1n,
          selected: false,
        })),
        total_values: BigInt(values.length),
        exhaustive: true,
        count_basis: "distinct roots",
      },
    },
  };
}
export const truePredicate: QueryPredicate = { kind: "boolean_constant", value: true };

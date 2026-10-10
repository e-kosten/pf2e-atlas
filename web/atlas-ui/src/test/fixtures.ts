import type {
  RecordDetailView,
  RecordSummaryView,
  FilterEditorView,
  QueryFieldDefinition,
  QueryPredicate,
  FilterValueListView,
} from "../generated/atlas";
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
    surface: {
      record_key,
      title: record.title,
      kind: record.kind,
      profile: "record_detail",
      header: { traits: [] },
      sections: target
        ? [
            {
              kind: "description",
              title: "Description",
              collapsed_by_default: false,
              content: {
                locator,
                role: "description",
                source_fingerprint: null,
                body: {
                  kind: "html",
                  html: '<p><a data-atlas-reference="0" href="#">Nested Rule</a></p>',
                  controls: [],
                },
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

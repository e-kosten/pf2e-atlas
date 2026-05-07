import { inferActorMetricValueType } from "../../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../../domain/item-metrics.js";
import { getMetricDiscoveryGroupLabel } from "../../../domain/metric-discovery-group-label.js";
import type { MetadataFieldName } from "../../../domain/metadata-field-types.js";
import {
  describeMetadataFieldType,
  formatMetadataFieldLabel,
} from "../../../domain/presentation-vocabulary.js";
import { isSearchPromotedFieldDomainKey } from "../../../domain/search-field-domains.js";
import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import { isMetadataFieldName } from "../../../domain/metadata-field-catalog.js";
import {
  getMetadataFilterSemantics,
  type MetadataFieldSemantics,
} from "../../../domain/metadata-field-catalog.js";
import { getSearchFilterNodeAtPath, isSearchFilterBooleanGroup } from "../../search/query-core.js";
import type {
  Pf2eTerminalQueryField,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import { getQueryFieldEditor } from "../../search/discoverable-fields.js";
import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import { getSearchQueryCategory } from "../../search/query-state.js";
import { getContainingBooleanGroupPath } from "./structured-draft-host-mutations.js";

export type StructuredDraftMetricFieldFamily = "actorMetric" | "itemMetric";

export type StructuredDraftMetricFieldRefFamily = "actor" | "item";

export type StructuredDraftFieldRef =
  | { kind: "metadata"; field: MetadataFieldName }
  | { kind: "metric"; family: StructuredDraftMetricFieldRefFamily }
  | { kind: "pack" };

export type StructuredDraftLeafAddKind =
  | "scope"
  | "level"
  | "price"
  | "metric"
  | "metricCompare"
  | "linksTo"
  | "linkedFrom";

export type StructuredDraftAddIntent =
  | { kind: "field"; field: StructuredDraftFieldRef; groupPath: number[] }
  | { kind: "leaf"; leaf: StructuredDraftLeafAddKind; groupPath: number[] };

export type StructuredDraftRouteCatalog = {
  getMetadataFieldSemantics: (field: MetadataFieldName) => MetadataFieldSemantics | null;
  isMetadataFieldAvailable: (field: MetadataFieldName, query: Pf2eTerminalSearchQuery) => boolean;
  isMetricFieldAvailable: (family: StructuredDraftMetricFieldRefFamily, query: Pf2eTerminalSearchQuery) => boolean;
  isPromotedGroupedField: (field: MetadataFieldName) => boolean;
};

export type StructuredDraftLeafKind =
  | "scope"
  | "level"
  | "price"
  | "metadataScalar"
  | "metadataBoolean"
  | "metadataText"
  | "metric"
  | "metricCompare"
  | "linksTo"
  | "linkedFrom";

export type StructuredDraftPromptAddClauseKind =
  | "field"
  | "metric"
  | "metricCompare"
  | "pack"
  | "scope"
  | "level"
  | "price"
  | "rarity"
  | "actionCost";

export type StructuredDraftEditRoute =
  | {
      kind: "groupField";
      field: Pf2eTerminalQueryFieldOption["value"];
      fieldOption: Pf2eTerminalQueryFieldOption;
      groupPath: number[];
      memberPaths: number[][];
      fieldMemberPaths: number[][];
      source: "bucket" | "member" | "add";
    }
  | {
      kind: "leaf";
      leafKind: StructuredDraftLeafKind;
      path: number[] | null;
      groupPath: number[] | null;
      placement: "inGroup" | "rootSingleton";
      fieldOption?: Pf2eTerminalQueryFieldOption;
    }
  | {
      kind: "unsupported";
      reason: string;
    };

const METADATA_FIELD_SEMANTICS_BY_NAME = new Map<MetadataFieldName, MetadataFieldSemantics>(
  getMetadataFilterSemantics().metadataFields.map((field) => [field.field, field]),
);

function metricRefFamilyToFieldValue(family: StructuredDraftMetricFieldRefFamily): StructuredDraftMetricFieldFamily {
  return family === "actor" ? "actorMetric" : "itemMetric";
}

function metricFieldValueToRefFamily(
  value: StructuredDraftMetricFieldFamily,
): StructuredDraftMetricFieldRefFamily {
  return value === "actorMetric" ? "actor" : "item";
}

export function createStructuredDraftRouteCatalog(
  availableFields: readonly Pf2eTerminalQueryField[],
): StructuredDraftRouteCatalog {
  const availableMetadataFields = new Set<MetadataFieldName>();
  const availableMetricFamilies = new Set<StructuredDraftMetricFieldRefFamily>();

  for (const field of availableFields) {
    if (field === "actorMetric" || field === "itemMetric") {
      availableMetricFamilies.add(metricFieldValueToRefFamily(field));
      continue;
    }
    if (isMetadataFieldName(field)) {
      availableMetadataFields.add(field);
    }
  }

  return {
    getMetadataFieldSemantics: (field) => METADATA_FIELD_SEMANTICS_BY_NAME.get(field) ?? null,
    isMetadataFieldAvailable: (field) =>
      availableMetadataFields.has(field) || isSearchPromotedFieldDomainKey(field),
    isMetricFieldAvailable: (family) => availableMetricFamilies.has(family),
    isPromotedGroupedField: (field) => isSearchPromotedFieldDomainKey(field),
  };
}

export function getStructuredDraftFieldRefForQueryFieldValue(
  value: Pf2eTerminalQueryField,
): StructuredDraftFieldRef | null {
  if (value === "pack") {
    return { kind: "pack" };
  }
  if (value === "actorMetric" || value === "itemMetric") {
    return { kind: "metric", family: metricFieldValueToRefFamily(value) };
  }
  if (isMetadataFieldName(value)) {
    return { kind: "metadata", field: value };
  }
  return null;
}

export function getStructuredDraftAddIntentForClauseKind(
  clauseKind: StructuredDraftPromptAddClauseKind,
  groupPath: number[],
): StructuredDraftAddIntent | null {
  switch (clauseKind) {
    case "pack":
      return { kind: "field", field: { kind: "pack" }, groupPath };
    case "rarity":
      return { kind: "field", field: { kind: "metadata", field: "rarity" }, groupPath };
    case "actionCost":
      return { kind: "field", field: { kind: "metadata", field: "actionCost" }, groupPath };
    case "scope":
    case "level":
    case "price":
    case "metric":
    case "metricCompare":
      return { kind: "leaf", leaf: clauseKind, groupPath };
    case "field":
      return null;
  }
}

export function buildStructuredDraftExplorerOnlyFieldOption(
  field: Pf2eTerminalQueryFieldOption["value"],
  label: string,
  description: string,
  fieldType: Pf2eTerminalQueryFieldOption["fieldType"],
): Pf2eTerminalQueryFieldOption {
  return {
    value: field,
    label,
    description,
    fieldType,
    editor: "sharedExplorer",
  };
}

export function getStructuredDraftPromotedFieldOption(
  field: Pf2eTerminalQueryFieldOption["value"],
): Pf2eTerminalQueryFieldOption | null {
  if (field === "pack") {
    return buildStructuredDraftExplorerOnlyFieldOption(
      "pack",
      "Pack",
      "Browse live packs for the current group and stage canonical pack clauses.",
      "enumString",
    );
  }
  if (field === "rarity") {
    return buildStructuredDraftExplorerOnlyFieldOption(
      "rarity",
      "Rarity",
      "Browse live rarities for the current group and stage canonical rarity clauses.",
      "enumString",
    );
  }
  if (field === "actionCost") {
    return buildStructuredDraftExplorerOnlyFieldOption(
      "actionCost",
      "Action Cost",
      "Browse live action costs for the current group and stage canonical action-cost clauses.",
      "number",
    );
  }
  return null;
}

function buildStructuredDraftMetadataFieldOption(
  semantics: MetadataFieldSemantics,
  options: { forceSharedExplorer?: boolean } = {},
): Pf2eTerminalQueryFieldOption {
  return {
    value: semantics.field,
    label: formatMetadataFieldLabel(semantics.field),
    description:
      semantics.notes ??
      (semantics.field === "derivedTags"
        ? "Derived-tag field with hierarchy-capable ontology browsing."
        : `${describeMetadataFieldType(semantics.fieldType)} query field for the current browse scope.`),
    fieldType: semantics.fieldType,
    editor: options.forceSharedExplorer ? "sharedExplorer" : getQueryFieldEditor(semantics),
  };
}

function buildStructuredDraftMetricFieldOption(
  family: StructuredDraftMetricFieldRefFamily,
  query: Pf2eTerminalSearchQuery,
): Pf2eTerminalQueryFieldOption {
  const category = getSearchQueryCategory(query);
  const metricField = family === "actor" ? "actorMetrics" : "itemMetrics";
  return {
    value: metricRefFamilyToFieldValue(family),
    label: category
      ? getMetricDiscoveryGroupLabel(category, metricField)
      : family === "actor"
        ? "Creature Statistics"
        : "Item Statistics",
    description:
      family === "actor"
        ? "Browse live statistic keys and author exact or numeric literal filters for the current scope."
        : "Browse live item property keys and author exact or numeric literal filters for the current scope.",
    fieldType: "enumString",
    editor: "sharedExplorer",
  };
}

function resolveStructuredDraftFieldOptionForRef({
  catalog,
  field,
  query,
}: {
  catalog: StructuredDraftRouteCatalog;
  field: StructuredDraftFieldRef;
  query: Pf2eTerminalSearchQuery;
}): Pf2eTerminalQueryFieldOption | null {
  switch (field.kind) {
    case "pack":
      return getStructuredDraftPromotedFieldOption("pack");
    case "metric":
      return catalog.isMetricFieldAvailable(field.family, query)
        ? buildStructuredDraftMetricFieldOption(field.family, query)
        : null;
    case "metadata": {
      if (!catalog.isMetadataFieldAvailable(field.field, query)) {
        return null;
      }
      const semantics = catalog.getMetadataFieldSemantics(field.field);
      if (!semantics) {
        return null;
      }
      return buildStructuredDraftMetadataFieldOption(semantics, {
        forceSharedExplorer: catalog.isPromotedGroupedField(field.field),
      });
    }
  }
}

export function isStructuredDraftMetricFieldOptionValue(
  value: Pf2eTerminalQueryFieldOption["value"],
): value is StructuredDraftMetricFieldFamily {
  return value === "actorMetric" || value === "itemMetric";
}

export function inferStructuredDraftMetricFieldFamily(
  metric: string,
  category: ReturnType<typeof getSearchQueryCategory> = null,
): StructuredDraftMetricFieldFamily {
  const actorValueType = inferActorMetricValueType(metric);
  const itemValueType = inferItemMetricValueType(metric);

  if (actorValueType && !itemValueType) {
    return "actorMetric";
  }
  if (itemValueType && !actorValueType) {
    return "itemMetric";
  }

  return category === "equipment" ? "itemMetric" : "actorMetric";
}

export function getStructuredDraftQueryFieldValueForNode(
  node: SearchFilterNode,
): Pf2eTerminalQueryFieldOption["value"] | null {
  switch (node.kind) {
    case "metadataPredicate":
      return node.predicate.field;
    case "metric":
      return inferStructuredDraftMetricFieldFamily(node.metric);
    case "pack":
      return "pack";
    case "rarity":
      return "rarity";
    case "actionCost":
      return "actionCost";
    case "scope":
    case "level":
    case "price":
    case "linksTo":
    case "linkedFrom":
    case "metricCompare":
    case "anyOf":
    case "allOf":
    case "not":
      return null;
  }
}

export function structuredDraftSearchFilterNodeContainsFieldValue(
  node: SearchFilterNode | null | undefined,
  field: Pf2eTerminalQueryFieldOption["value"],
): boolean {
  if (!node) {
    return false;
  }

  if (node.kind === "metadataPredicate") {
    return node.predicate.field === field;
  }

  if (node.kind === "rarity" || node.kind === "actionCost" || node.kind === "pack") {
    return node.kind === field;
  }

  if (node.kind === "metric") {
    return inferStructuredDraftMetricFieldFamily(node.metric) === field;
  }

  if (node.kind === "allOf" || node.kind === "anyOf") {
    return node.children.some((child) => structuredDraftSearchFilterNodeContainsFieldValue(child, field));
  }

  if (node.kind === "not") {
    return structuredDraftSearchFilterNodeContainsFieldValue(node.child, field);
  }

  return false;
}

export function isStructuredDraftGroupFieldRef(
  field: StructuredDraftFieldRef,
  catalog: StructuredDraftRouteCatalog,
): boolean {
  if (field.kind === "pack") {
    return true;
  }
  if (field.kind === "metric") {
    return false;
  }
  if (catalog.isPromotedGroupedField(field.field)) {
    return true;
  }
  const semantics = catalog.getMetadataFieldSemantics(field.field);
  return semantics?.fieldType === "set" || semantics?.fieldType === "enumString";
}

export function isStructuredDraftGroupFieldOption(fieldOption: Pf2eTerminalQueryFieldOption): boolean {
  const fieldRef = getStructuredDraftFieldRefForQueryFieldValue(fieldOption.value);
  return fieldRef ? isStructuredDraftGroupFieldRef(fieldRef, createStructuredDraftRouteCatalog([fieldOption.value])) : false;
}

export function isStructuredDraftGroupFieldRoute(
  route: StructuredDraftEditRoute,
): route is Extract<StructuredDraftEditRoute, { kind: "groupField" }> {
  return route.kind === "groupField";
}

function getLeafKindForFieldOption(fieldOption: Pf2eTerminalQueryFieldOption): StructuredDraftLeafKind {
  if (isStructuredDraftMetricFieldOptionValue(fieldOption.value)) {
    return "metric";
  }
  if (fieldOption.fieldType === "boolean") {
    return "metadataBoolean";
  }
  if (fieldOption.fieldType === "text") {
    return "metadataText";
  }
  return "metadataScalar";
}

function getLeafKindForNode(
  node: SearchFilterNode,
  fieldOption?: Pf2eTerminalQueryFieldOption,
): StructuredDraftLeafKind | null {
  switch (node.kind) {
    case "scope":
      return "scope";
    case "level":
      return "level";
    case "price":
      return "price";
    case "linksTo":
      return "linksTo";
    case "linkedFrom":
      return "linkedFrom";
    case "metric":
      return "metric";
    case "metricCompare":
      return "metricCompare";
    case "metadataPredicate":
      return fieldOption ? getLeafKindForFieldOption(fieldOption) : "metadataScalar";
    case "pack":
    case "rarity":
    case "actionCost":
    case "allOf":
    case "anyOf":
    case "not":
      return null;
  }
}

function collectGroupMemberPathsForField(
  filter: SearchFilterNode | undefined,
  groupPath: number[],
  field: Pf2eTerminalQueryFieldOption["value"],
): number[][] {
  const groupNode = groupPath.length === 0 ? filter : getSearchFilterNodeAtPath(filter, groupPath);
  if (!groupNode) {
    return [];
  }
  if (!isSearchFilterBooleanGroup(groupNode)) {
    return structuredDraftSearchFilterNodeContainsFieldValue(groupNode, field) ? [groupPath] : [];
  }
  return groupNode.children.flatMap((child, childIndex) =>
    structuredDraftSearchFilterNodeContainsFieldValue(child, field) ? [[...groupPath, childIndex]] : [],
  );
}

export function classifyStructuredDraftAddIntentRoute({
  catalog,
  intent,
  query,
  structuralWrapper,
}: {
  catalog: StructuredDraftRouteCatalog;
  intent: StructuredDraftAddIntent;
  query: Pf2eTerminalSearchQuery;
  structuralWrapper?: "allOf" | "anyOf" | "not";
}): StructuredDraftEditRoute {
  if (intent.kind === "leaf") {
    return {
      kind: "leaf",
      leafKind: intent.leaf,
      path: null,
      groupPath: intent.groupPath,
      placement: intent.leaf === "scope" ? "rootSingleton" : "inGroup",
    };
  }

  const fieldOption = resolveStructuredDraftFieldOptionForRef({ catalog, field: intent.field, query });
  if (!fieldOption) {
    return { kind: "unsupported", reason: "That field is not available in the current query scope." };
  }

  if (isStructuredDraftGroupFieldRef(intent.field, catalog)) {
    if (structuralWrapper !== undefined) {
      return {
        kind: "unsupported",
        reason: "Grouped query fields must be added directly to an existing group before structural wrapping.",
      };
    }
    const memberPaths = collectGroupMemberPathsForField(query.filter, intent.groupPath, fieldOption.value);
    return {
      kind: "groupField",
      field: fieldOption.value,
      fieldOption,
      groupPath: intent.groupPath,
      memberPaths,
      fieldMemberPaths: memberPaths,
      source: "add",
    };
  }

  return {
    kind: "leaf",
    leafKind: getLeafKindForFieldOption(fieldOption),
    path: null,
    groupPath: intent.groupPath,
    placement: "inGroup",
    fieldOption,
  };
}

export function classifyStructuredDraftBucketEditRoute({
  catalog,
  entry,
  query,
}: {
  catalog: StructuredDraftRouteCatalog;
  entry: SearchStructuredDraftEntry;
  query: Pf2eTerminalSearchQuery;
}): StructuredDraftEditRoute {
  if (entry.kind !== "queryFieldBucket") {
    return { kind: "unsupported", reason: "That row is not a grouped field bucket." };
  }

  const field = entry.field as Pf2eTerminalQueryFieldOption["value"] | undefined;
  if (!field) {
    return { kind: "unsupported", reason: "That grouped row is missing its query field." };
  }

  const fieldRef = getStructuredDraftFieldRefForQueryFieldValue(field);
  const fieldOption = fieldRef
    ? resolveStructuredDraftFieldOptionForRef({ catalog, field: fieldRef, query })
    : null;
  if (!fieldRef || !fieldOption || !isStructuredDraftGroupFieldRef(fieldRef, catalog)) {
    return { kind: "unsupported", reason: "That grouped row cannot be edited through the shared explorer." };
  }

  const memberPaths = entry.memberPaths ?? [];
  const fieldMemberPaths = entry.fieldMemberPaths ?? memberPaths;

  return {
    kind: "groupField",
    field: fieldOption.value,
    fieldOption,
    groupPath: entry.groupPath ?? [],
    memberPaths,
    fieldMemberPaths,
    source: "bucket",
  };
}

export function classifyStructuredDraftNodeEditRoute({
  catalog,
  path,
  query,
}: {
  catalog: StructuredDraftRouteCatalog;
  path: number[];
  query: Pf2eTerminalSearchQuery;
}): StructuredDraftEditRoute {
  const node = getSearchFilterNodeAtPath(query.filter, path);
  if (!node) {
    return { kind: "unsupported", reason: "That clause is no longer present in the query tree." };
  }

  const fieldValue = getStructuredDraftQueryFieldValueForNode(node);
  const fieldRef = fieldValue ? getStructuredDraftFieldRefForQueryFieldValue(fieldValue) : null;
  const fieldOption = fieldRef
    ? resolveStructuredDraftFieldOptionForRef({ catalog, field: fieldRef, query })
    : null;
  if (fieldRef && fieldOption && isStructuredDraftGroupFieldRef(fieldRef, catalog)) {
    const groupPath = getContainingBooleanGroupPath(query.filter, path);
    const memberPaths = collectGroupMemberPathsForField(query.filter, groupPath, fieldOption.value);
    return {
      kind: "groupField",
      field: fieldOption.value,
      fieldOption,
      groupPath,
      memberPaths,
      fieldMemberPaths: memberPaths,
      source: "member",
    };
  }

  const leafKind = getLeafKindForNode(node, fieldOption ?? undefined);
  if (!leafKind) {
    return { kind: "unsupported", reason: "That clause cannot be edited through the current canonical editor set." };
  }

  return {
    kind: "leaf",
    leafKind,
    path,
    groupPath: null,
    placement: leafKind === "scope" ? "rootSingleton" : "inGroup",
    fieldOption: fieldOption ?? undefined,
  };
}

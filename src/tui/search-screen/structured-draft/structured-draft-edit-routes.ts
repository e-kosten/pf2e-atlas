import { inferActorMetricValueType } from "../../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../../domain/item-metrics.js";
import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import { canonicalFilterToMetadataNode } from "../../search/query-parts.js";
import { getSearchFilterNodeAtPath, isSearchFilterBooleanGroup } from "../../search/query-core.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import { getSearchQueryCategory } from "../../search/query-state.js";
import { getContainingBooleanGroupPath } from "./structured-draft-host-mutations.js";

export type StructuredDraftMetricFieldFamily = "actorMetric" | "itemMetric";

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
      placement: "inGroup" | "rootSingleton";
      fieldOption?: Pf2eTerminalQueryFieldOption;
    }
  | {
      kind: "unsupported";
      reason: string;
    };

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

export function getStructuredDraftSyntheticFieldOption(
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

export function isStructuredDraftGroupFieldOption(fieldOption: Pf2eTerminalQueryFieldOption): boolean {
  if (fieldOption.value === "pack" || fieldOption.value === "rarity" || fieldOption.value === "actionCost") {
    return true;
  }
  if (isStructuredDraftMetricFieldOptionValue(fieldOption.value)) {
    return false;
  }
  return fieldOption.editor === "sharedExplorer" && (fieldOption.fieldType === "set" || fieldOption.fieldType === "enumString");
}

export function isStructuredDraftGroupFieldRoute(
  route: StructuredDraftEditRoute,
): route is Extract<StructuredDraftEditRoute, { kind: "groupField" }> {
  return route.kind === "groupField";
}

export function resolveStructuredDraftFieldOption(
  field: Pf2eTerminalQueryFieldOption["value"],
  fieldOptions: readonly Pf2eTerminalQueryFieldOption[],
): Pf2eTerminalQueryFieldOption | null {
  return getStructuredDraftSyntheticFieldOption(field) ?? fieldOptions.find((candidate) => candidate.value === field) ?? null;
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

function getLeafKindForNode(node: SearchFilterNode, fieldOption?: Pf2eTerminalQueryFieldOption): StructuredDraftLeafKind | null {
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

export function classifyStructuredDraftAddFieldRoute({
  fieldOption,
  groupPath,
  query,
}: {
  fieldOption: Pf2eTerminalQueryFieldOption;
  groupPath: number[];
  query: Pf2eTerminalSearchQuery;
}): StructuredDraftEditRoute {
  if (isStructuredDraftGroupFieldOption(fieldOption)) {
    const memberPaths = collectGroupMemberPathsForField(query.filter, groupPath, fieldOption.value);
    return {
      kind: "groupField",
      field: fieldOption.value,
      fieldOption,
      groupPath,
      memberPaths,
      fieldMemberPaths: memberPaths,
      source: "add",
    };
  }

  return {
    kind: "leaf",
    leafKind: getLeafKindForFieldOption(fieldOption),
    path: null,
    placement: "inGroup",
    fieldOption,
  };
}

export function classifyStructuredDraftBucketEditRoute({
  entry,
  fieldOptions,
}: {
  entry: SearchStructuredDraftEntry;
  fieldOptions: readonly Pf2eTerminalQueryFieldOption[];
}): StructuredDraftEditRoute {
  if (entry.kind !== "queryFieldBucket") {
    return { kind: "unsupported", reason: "That row is not a grouped field bucket." };
  }

  const field = entry.field as Pf2eTerminalQueryFieldOption["value"] | undefined;
  if (!field) {
    return { kind: "unsupported", reason: "That grouped row is missing its query field." };
  }

  const fieldOption = resolveStructuredDraftFieldOption(field, fieldOptions);
  if (!fieldOption || !isStructuredDraftGroupFieldOption(fieldOption)) {
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
  fieldOptions,
  path,
  query,
}: {
  fieldOptions: readonly Pf2eTerminalQueryFieldOption[];
  path: number[];
  query: Pf2eTerminalSearchQuery;
}): StructuredDraftEditRoute {
  const node = getSearchFilterNodeAtPath(query.filter, path);
  if (!node) {
    return { kind: "unsupported", reason: "That clause is no longer present in the query tree." };
  }

  const fieldValue = getStructuredDraftQueryFieldValueForNode(node);
  const fieldOption = fieldValue ? resolveStructuredDraftFieldOption(fieldValue, fieldOptions) : null;
  if (fieldOption && isStructuredDraftGroupFieldOption(fieldOption)) {
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

  if (node.kind === "metadataPredicate" && !canonicalFilterToMetadataNode(node)) {
    return { kind: "unsupported", reason: "That metadata clause cannot be edited through the current canonical editor set." };
  }

  return {
    kind: "leaf",
    leafKind,
    path,
    placement: leafKind === "scope" ? "rootSingleton" : "inGroup",
    fieldOption: fieldOption ?? undefined,
  };
}

import type { RuleGroupType, RuleType } from "react-querybuilder";
import type {
  QueryPredicate,
  QueryFieldDefinition,
  QueryLiteral,
  QueryCompare,
  QuerySetMatch,
  QueryFieldState,
} from "../../generated/atlas";
// A structural builder condition, never a backend catalog field.
export const BOOLEAN_CONDITION = "atlas:boolean-constant";
type PredicateGroup = RuleGroupType & {
  atlasSinglePredicate?: boolean;
  atlasNotClauseId?: string;
};
export function numericLiteral(value: unknown): number {
  if (typeof value !== "string" && typeof value !== "number")
    throw new Error("Enter a numeric value");
  if (String(value).trim() === "") throw new Error("Enter a numeric value");
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) throw new Error("Enter a finite numeric value");
  if (Number.isInteger(parsed) && !Number.isSafeInteger(parsed))
    throw new Error("Integer exceeds JavaScript's exact numeric range");
  return parsed;
}
const states: QueryFieldState[] = [
  "value",
  "missing",
  "null",
  "invalid",
  "not_applicable",
];
export const emptyGroup = (): RuleGroupType => ({ combinator: "and", rules: [] });
function literal(value: unknown, field: QueryFieldDefinition): QueryLiteral {
  if (field.field_type === "number") return numericLiteral(value);
  if (field.field_type === "boolean") {
    if (value === true || value === "true") return true;
    if (value === false || value === "false") return false;
    throw new Error("Choose true or false");
  }
  if (typeof value !== "string" || !value.trim()) throw new Error("Enter a value");
  return value;
}
function values(value: unknown): unknown[] {
  return Array.isArray(value)
    ? value
    : typeof value === "string"
      ? value
          .split(",")
          .map((v) => v.trim())
          .filter(Boolean)
      : [];
}
export function builderPredicate(
  group: RuleGroupType,
  fields: QueryFieldDefinition[],
  scope: string | null = null,
): QueryPredicate | null {
  const children: QueryPredicate[] = [];
  for (const node of group.rules) {
    if ("rules" in node) {
      const child = builderPredicate(node as RuleGroupType, fields, scope);
      if (child) children.push(child);
      continue;
    }
    if (node.field === BOOLEAN_CONDITION) {
      if (
        node.operator !== "constant" ||
        ![true, false, "true", "false"].includes(node.value)
      )
        throw new Error("Choose All records or No records");
      children.push({
        kind: "boolean_constant",
        clause_id: node.id,
        value: node.value === true || node.value === "true",
      });
      continue;
    }
    const field = fields.find((f) => f.id === node.field && f.scope === scope);
    if (!field) throw new Error("Choose an available filter field");
    if (!field.operators.includes(node.operator))
      throw new Error("Choose an allowed operator");
    const clause_id = node.id;
    if (node.operator === "state") {
      if (!states.includes(node.value)) throw new Error("Choose a field state");
      children.push({
        kind: "state_match",
        clause_id,
        field: field.path,
        state: node.value,
      });
    } else if (node.operator === "exists") {
      if (scope !== null) throw new Error("Nested collection filters are unavailable");
      const child = builderPredicate(
        node.value || emptyGroup(),
        fields,
        field.path,
      ) || { kind: "boolean_constant", value: true };
      children.push({
        kind: "exists",
        clause_id,
        collection: field.path,
        scope_id:
          (node as RuleType & { atlasScopeId?: string }).atlasScopeId ||
          node.id ||
          crypto.randomUUID(),
        predicate: child,
      });
    } else if (node.operator === "in") {
      const parsed = values(node.value).map((v) => literal(v, field));
      if (!parsed.length) throw new Error("Choose at least one value");
      children.push({ kind: "in", clause_id, field: field.path, values: parsed });
    } else if (field.field_type === "set") {
      const parsed = values(node.value).map((v) => String(v));
      if (!parsed.length) throw new Error("Choose at least one value");
      children.push({
        kind: "set_match",
        clause_id,
        field: field.path,
        op: node.operator as QuerySetMatch,
        values: parsed,
      });
    } else if (node.operator === "between") {
      const range = values(node.value);
      if (range.length !== 2) throw new Error("Enter minimum and maximum");
      const min = numericLiteral(range[0]),
        max = numericLiteral(range[1]);
      if (min > max) throw new Error("Minimum exceeds maximum");
      children.push({
        kind: "all_of",
        clause_id,
        children: [
          { kind: "compare", field: field.path, op: "gte", value: min },
          { kind: "compare", field: field.path, op: "lte", value: max },
        ],
      });
    } else {
      children.push({
        kind: "compare",
        clause_id,
        field: field.path,
        op: node.operator as QueryCompare,
        value: literal(node.value, field),
      });
    }
  }
  if (!children.length) return null;
  const predicate: QueryPredicate =
    (group as PredicateGroup).atlasSinglePredicate &&
    children.length === 1 &&
    group.combinator === "and"
      ? children[0]
      : {
          kind: group.combinator === "or" ? "any_of" : "all_of",
          clause_id: group.id,
          children,
        };
  return group.not
    ? {
        kind: "not",
        clause_id: (group as RuleGroupType & { atlasNotClauseId?: string })
          .atlasNotClauseId,
        predicate,
      }
    : predicate;
}
export function predicateBuilder(
  predicate: QueryPredicate | null,
  fields: QueryFieldDefinition[],
  scope: string | null = null,
): RuleGroupType {
  if (!predicate) return emptyGroup();
  if (predicate.kind === "all_of" || predicate.kind === "any_of") {
    return {
      id: predicate.clause_id,
      combinator: predicate.kind === "all_of" ? "and" : "or",
      rules: predicate.children.map((p) => predicateNode(p, fields, scope)),
    };
  }
  if (predicate.kind === "not")
    return {
      combinator: "and",
      rules: [predicateNode(predicate.predicate, fields, scope)],
      not: true,
      atlasSinglePredicate: true,
      atlasNotClauseId: predicate.clause_id,
    } as PredicateGroup;
  return {
    combinator: "and",
    rules: [predicateNode(predicate, fields, scope)],
    atlasSinglePredicate: true,
  } as PredicateGroup;
}
function predicateNode(
  predicate: QueryPredicate,
  fields: QueryFieldDefinition[],
  scope: string | null,
): RuleType | RuleGroupType {
  if (
    predicate.kind === "all_of" ||
    predicate.kind === "any_of" ||
    predicate.kind === "not"
  )
    return predicateBuilder(predicate, fields, scope);
  if (predicate.kind === "boolean_constant")
    return {
      id: predicate.clause_id,
      field: BOOLEAN_CONDITION,
      operator: "constant",
      value: predicate.value,
    };
  const fieldPath =
    predicate.kind === "exists" ? predicate.collection : predicate.field;
  const field = fields.find((f) => f.path === fieldPath && f.scope === scope);
  if (!field) throw new Error("Filter URL contains an unavailable field");
  const base = { id: predicate.clause_id, field: field.id };
  switch (predicate.kind) {
    case "compare":
      return { ...base, operator: predicate.op, value: predicate.value };
    case "in":
      return { ...base, operator: "in", value: predicate.values };
    case "set_match":
      return { ...base, operator: predicate.op, value: predicate.values };
    case "state_match":
      return { ...base, operator: "state", value: predicate.state };
    case "exists":
      return {
        ...base,
        atlasScopeId: predicate.scope_id,
        operator: "exists",
        value: predicateBuilder(predicate.predicate, fields, predicate.collection),
      } as RuleType & { atlasScopeId: string };
  }
}

import type { RuleGroupType, RuleType } from "react-querybuilder";
import type {
  QueryPredicate,
  QueryFieldDefinition,
  QueryLiteral,
  QueryCompare,
  QuerySetMatch,
  QueryFieldState,
} from "../../generated/atlas";
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
        scope_id: node.id || crypto.randomUUID(),
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
  const predicate: QueryPredicate = {
    kind: group.combinator === "or" ? "any_of" : "all_of",
    clause_id: group.id,
    children,
  };
  return group.not ? { kind: "not", predicate } : predicate;
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
    return { ...predicateBuilder(predicate.predicate, fields, scope), not: true };
  if (predicate.kind === "boolean_constant") {
    if (predicate.value) return emptyGroup();
    throw new Error("A false constant cannot be edited as a filter rule");
  }
  return { combinator: "and", rules: [predicateNode(predicate, fields, scope)] };
}
function predicateNode(
  predicate: QueryPredicate,
  fields: QueryFieldDefinition[],
  scope: string | null,
): RuleType | RuleGroupType {
  if (
    predicate.kind === "all_of" ||
    predicate.kind === "any_of" ||
    predicate.kind === "not" ||
    predicate.kind === "boolean_constant"
  )
    return predicateBuilder(predicate, fields, scope);
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
        id: predicate.scope_id,
        operator: "exists",
        value: predicateBuilder(predicate.predicate, fields, predicate.collection),
      };
  }
}

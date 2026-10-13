import type { RuleGroupType, RuleType } from "react-querybuilder";
import { commonCondition, type CommonCondition } from "./filterControls";
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
export type PredicateRule = RuleType & {
  meta?: {
    atlasExcluded?: boolean;
    atlasNotClauseId?: string;
  };
  atlasScopeId?: string;
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
const draftMentionsField = (node: RuleType | RuleGroupType, field: string): boolean =>
  "rules" in node
    ? node.rules.some((child) => draftMentionsField(child, field))
    : node.field === field;
function commonDraftNodes(
  query: RuleGroupType,
  field: QueryFieldDefinition,
): Array<RuleType | RuleGroupType> | null {
  const matching: Array<RuleType | RuleGroupType> = [];
  const onlyField = (node: RuleType | RuleGroupType): boolean =>
    "rules" in node
      ? node.rules.length > 0 && node.rules.every(onlyField)
      : node.field === field.id;
  function collect(group: RuleGroupType): boolean {
    if (!draftMentionsField(group, field.id)) return true;
    if (group.not || group.combinator !== "and") return false;
    if (onlyField(group)) {
      matching.push(group);
      return true;
    }
    return group.rules.every((node) => {
      if ("rules" in node) return collect(node);
      if (node.field === field.id) matching.push(node);
      return true;
    });
  }
  return collect(query) ? matching : null;
}
export function commonDraftCondition(
  query: RuleGroupType,
  field: QueryFieldDefinition,
): CommonCondition {
  const matching = commonDraftNodes(query, field);
  if (!matching) return { status: "advanced", clause: null };
  if (!matching.length) return { status: "empty", clause: null };
  try {
    return commonCondition(
      builderPredicate({ combinator: "and", rules: matching }, [field]),
      field,
    );
  } catch {
    return { status: "advanced", clause: null };
  }
}
/** Raw numeric text remains editable only in an exact positive range shape. */
export function commonDraftRange(
  query: RuleGroupType,
  field: QueryFieldDefinition,
): [string, string] | null {
  if (field.field_type !== "number") return null;
  const matching = commonDraftNodes(query, field);
  if (!matching || !matching.length) return null;
  const leaves: PredicateRule[] = [];
  function collect(node: RuleType | RuleGroupType): boolean {
    if ("rules" in node)
      return !node.not && node.combinator === "and" && node.rules.every(collect);
    leaves.push(node);
    return !node.meta?.atlasExcluded;
  }
  if (!matching.every(collect)) return null;
  const text = (value: unknown) =>
    typeof value === "string" || typeof value === "number" ? String(value) : null;
  if (leaves.length === 1) {
    const { operator, value } = leaves[0];
    if (operator === "between" && Array.isArray(value) && value.length === 2) {
      const min = text(value[0]);
      const max = text(value[1]);
      return min !== null && max !== null ? [min, max] : null;
    }
    const valueText = text(value);
    if (valueText === null) return null;
    if (operator === "eq") return [valueText, valueText];
    if (operator === "gte") return [valueText, ""];
    if (operator === "lte") return ["", valueText];
  }
  if (leaves.length === 2) {
    const min = leaves.find((node) => node.operator === "gte");
    const max = leaves.find((node) => node.operator === "lte");
    const minText = min ? text(min.value) : null;
    const maxText = max ? text(max.value) : null;
    if (minText !== null && maxText !== null) return [minText, maxText];
  }
  return null;
}
export function canReplaceCommonDraft(
  query: RuleGroupType,
  field: QueryFieldDefinition,
): boolean {
  return (
    commonDraftCondition(query, field).status !== "advanced" ||
    commonDraftRange(query, field) !== null
  );
}
/** Refresh a quick-control field without discarding unfinished unrelated rules. */
export function replaceCommonDraft(
  query: RuleGroupType,
  field: QueryFieldDefinition,
  replacement: QueryPredicate | null,
): RuleGroupType {
  const next = predicateBuilder(replacement, [field]);
  const existingGroups = new Map<string, PredicateGroup>();
  function remember(group: PredicateGroup) {
    if (group.id) existingGroups.set(group.id, group);
    for (const node of group.rules) if ("rules" in node) remember(node);
  }
  remember(query);
  function commonGroup(node: RuleType | RuleGroupType): RuleType | RuleGroupType {
    if (!("rules" in node)) return node;
    const existing = node.id ? existingGroups.get(node.id) : undefined;
    return {
      ...node,
      // Quick controls synthesize conjunctions; unlike authored groups, those
      // wrappers have no independent editing meaning when only one leaf remains.
      ...(!node.not &&
      node.combinator === "and" &&
      (!existing || existing.atlasSinglePredicate)
        ? { atlasSinglePredicate: true }
        : {}),
      rules: node.rules.map(commonGroup),
    };
  }
  const nextNode = replacement
    ? commonGroup(
        (next as PredicateGroup).atlasSinglePredicate && !next.not
          ? next.rules[0]
          : next,
      )
    : null;
  if (!canReplaceCommonDraft(query, field))
    throw new Error("Edit this condition in advanced filters");
  return replaceCommonNodes(query, field.id, nextNode);
}
/** Keep partially typed numeric operands in the same shared editor tree. */
export function replaceCommonRangeDraft(
  query: RuleGroupType,
  field: QueryFieldDefinition,
  range: [string, string],
): RuleGroupType {
  if (field.field_type !== "number" || !canReplaceCommonDraft(query, field))
    throw new Error("Edit this condition in advanced filters");
  const matching = commonDraftNodes(query, field) ?? [];
  function firstRule(node: RuleType | RuleGroupType): RuleType | undefined {
    return "rules" in node ? node.rules.map(firstRule).find(Boolean) : node;
  }
  const existing = matching.map(firstRule).find(Boolean);
  const hasMin = !!range[0].trim();
  const hasMax = !!range[1].trim();
  const node: RuleType | null =
    hasMin || hasMax
      ? {
          ...existing,
          id: existing?.id ?? crypto.randomUUID(),
          field: field.id,
          operator: hasMin && hasMax ? "between" : hasMin ? "gte" : "lte",
          value: hasMin && hasMax ? range : hasMin ? range[0] : range[1],
        }
      : null;
  return replaceCommonNodes(query, field.id, node);
}
function replaceCommonNodes(
  query: RuleGroupType,
  fieldId: string,
  nextNode: RuleType | RuleGroupType | null,
): RuleGroupType {
  if (query.not || query.combinator !== "and")
    return nextNode ? { combinator: "and", rules: [query, nextNode] } : query;
  let inserted = false;
  function update(group: RuleGroupType): RuleGroupType {
    return {
      ...group,
      rules: group.rules.flatMap((node): Array<RuleType | RuleGroupType> => {
        if (!("rules" in node) && node.field === fieldId) {
          if (inserted || !nextNode) return [];
          inserted = true;
          // A common include/exclude pair can already own this authored AND.
          // Replace its leaves in place rather than nesting a second copy of
          // the same group identity inside itself.
          return "rules" in nextNode &&
            nextNode.id &&
            nextNode.id === group.id &&
            nextNode.combinator === group.combinator &&
            !nextNode.not
            ? nextNode.rules
            : [nextNode];
        }
        if ("rules" in node && node.combinator === "and" && !node.not) {
          return [update(node)];
        }
        return [node];
      }),
    };
  }
  const updated = update(query);
  if (!inserted && nextNode) updated.rules.push(nextNode);
  return updated;
}
function literal(value: unknown, field: QueryFieldDefinition): QueryLiteral {
  if (field.field_type === "number") return numericLiteral(value);
  if (field.field_type === "boolean") {
    if (value === true || value === "true") return true;
    if (value === false || value === "false") return false;
    throw new Error("Choose true or false");
  }
  if (typeof value !== "string" || !value.trim()) throw new Error("Enter a value");
  if (field.choices.length && !field.choices.includes(value))
    throw new Error("Choose an available value");
  return value;
}
function values(value: unknown): unknown[] {
  return Array.isArray(value)
    ? value
    : typeof value === "string"
      ? value.trim()
        ? [value]
        : []
      : [];
}
export function builderPredicate(
  group: RuleGroupType,
  fields: QueryFieldDefinition[],
  scope: string | null = null,
): QueryPredicate | null {
  if (
    !group ||
    !Array.isArray(group.rules) ||
    !["and", "or"].includes(group.combinator)
  )
    throw new Error("Choose a valid condition group");
  const children: QueryPredicate[] = [];
  for (const node of group.rules) {
    if ("rules" in node) {
      const child = builderPredicate(node as RuleGroupType, fields, scope);
      if (child) children.push(child);
      continue;
    }
    const rule = node as PredicateRule;
    const append = (predicate: QueryPredicate) =>
      children.push(
        rule.meta?.atlasExcluded
          ? { kind: "not", clause_id: rule.meta.atlasNotClauseId, predicate }
          : predicate,
      );
    if (node.field === BOOLEAN_CONDITION) {
      if (
        node.operator !== "constant" ||
        ![true, false, "true", "false"].includes(node.value)
      )
        throw new Error("Choose All records or No records");
      append({
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
      append({
        kind: "state_match",
        clause_id,
        field: field.path,
        state: node.value,
      });
    } else if (node.operator === "exists") {
      if (scope !== null) throw new Error("Nested collection filters are unavailable");
      const child = builderPredicate(
        node.value ?? emptyGroup(),
        fields,
        field.path,
      ) || { kind: "boolean_constant", value: true };
      append({
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
      append({ kind: "in", clause_id, field: field.path, values: parsed });
    } else if (field.field_type === "set") {
      const parsed = values(node.value).map((v) => String(literal(v, field)));
      if (!parsed.length) throw new Error("Choose at least one value");
      if (node.operator === "includes" && parsed.length !== 1)
        throw new Error("Choose one value");
      append({
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
      append({
        kind: "all_of",
        clause_id,
        children: [
          { kind: "compare", field: field.path, op: "gte", value: min },
          { kind: "compare", field: field.path, op: "lte", value: max },
        ],
      });
    } else {
      append({
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
  if (!predicate)
    return { ...emptyGroup(), atlasSinglePredicate: true } as PredicateGroup;
  if (predicate.kind === "all_of" || predicate.kind === "any_of") {
    return {
      id: predicate.clause_id,
      combinator: predicate.kind === "all_of" ? "and" : "or",
      rules: predicate.children.map((p) => predicateNode(p, fields, scope)),
    };
  }
  if (predicate.kind === "not") {
    // QueryBuilder represents a group's negation on the group itself. Wrapping
    // it in another group adds an editor level after every controlled update.
    if (predicate.predicate.kind === "all_of" || predicate.predicate.kind === "any_of")
      return {
        ...predicateBuilder(predicate.predicate, fields, scope),
        not: true,
        atlasNotClauseId: predicate.clause_id,
      } as PredicateGroup;
    if (predicate.predicate.kind === "not")
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
    predicate.kind === "not" &&
    !["all_of", "any_of", "not"].includes(predicate.predicate.kind)
  )
    return {
      ...predicateNode(predicate.predicate, fields, scope),
      meta: {
        atlasExcluded: true,
        atlasNotClauseId: predicate.clause_id,
      },
    } as PredicateRule;
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
  const base = { id: predicate.clause_id, field: field?.id ?? fieldPath };
  switch (predicate.kind) {
    case "compare":
      return { ...base, operator: predicate.op, value: predicate.value };
    case "in":
      return { ...base, operator: "in", value: predicate.values };
    case "set_match":
      return {
        ...base,
        operator: predicate.op,
        value:
          predicate.op === "includes" && predicate.values.length === 1
            ? predicate.values[0]
            : predicate.values,
      };
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

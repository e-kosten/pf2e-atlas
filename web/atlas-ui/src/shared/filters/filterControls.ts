import type {
  FilterCountsView,
  FilterDiscoveryContext,
  FilterEditorView,
  FilterVocabularyScope,
  QueryFieldDefinition,
  QueryPredicate,
} from "../../generated/atlas";
import type { SearchFormState } from "./searchState";
export type FilterPanelState = {
  search: SearchFormState;
  setSearch: (next: SearchFormState) => void;
  filterEditor: FilterEditorView | undefined;
  filterCountsByField?: Record<string, FilterCountsView | undefined>;
  filterDiscoveryContext?: FilterDiscoveryContext;
  vocabularyScope?: FilterVocabularyScope;
  filterDiscoveryLoading?: boolean;
  filterDiscoveryError?: string | null;
};
export function clearAllFilters(search: SearchFormState): SearchFormState {
  return {
    ...search,
    filter: null,
    filterError: null,
  };
}

export function predicateMentionsField(
  predicate: QueryPredicate,
  path: string,
): boolean {
  if (!predicate || typeof predicate !== "object") return false;
  if ("field" in predicate) return predicate.field === path;
  if (predicate.kind === "all_of" || predicate.kind === "any_of")
    return (
      Array.isArray(predicate.children) &&
      predicate.children.some((child) => predicateMentionsField(child, path))
    );
  if (predicate.kind === "not" || predicate.kind === "exists")
    return predicateMentionsField(predicate.predicate, path);
  return false;
}
function commonShape(predicate: QueryPredicate, field: QueryFieldDefinition): boolean {
  if (field.field_type === "number") {
    if (predicate.kind === "compare")
      return (
        predicate.field === field.path && ["eq", "gte", "lte"].includes(predicate.op)
      );
    return (
      predicate.kind === "all_of" &&
      predicate.children.length === 2 &&
      predicate.children.every(
        (child) =>
          child.kind === "compare" &&
          child.field === field.path &&
          typeof child.value === "number",
      ) &&
      predicate.children.some(
        (child) => child.kind === "compare" && child.op === "gte",
      ) &&
      predicate.children.some((child) => child.kind === "compare" && child.op === "lte")
    );
  }
  if (predicate.kind === "all_of")
    return (
      predicate.children.length === 2 &&
      predicate.children.every(
        (child) => child.kind !== "all_of" && commonShape(child, field),
      ) &&
      isExcluded(predicate.children[0]) !== isExcluded(predicate.children[1]) &&
      !conditionValues(predicate.children[0]).some((value) =>
        conditionValues(predicate.children[1]).includes(value),
      )
    );
  if (field.field_type === "set")
    return (
      predicate.kind === "set_match" &&
      predicate.field === field.path &&
      ["includes_any", "excludes_any"].includes(predicate.op)
    );
  return (
    (predicate.kind === "in" && predicate.field === field.path) ||
    (predicate.kind === "compare" &&
      predicate.field === field.path &&
      predicate.op === "eq") ||
    (predicate.kind === "not" &&
      predicate.predicate.kind === "in" &&
      predicate.predicate.field === field.path)
  );
}
export type CommonCondition =
  | { status: "empty"; clause: null }
  | { status: "editable"; clause: QueryPredicate }
  | { status: "advanced"; clause: null };
/** Recognize a bounded common-control representation, never simplify a tree. */
export function commonCondition(
  predicate: QueryPredicate | null,
  field: QueryFieldDefinition,
): CommonCondition {
  if (!predicate || !predicateMentionsField(predicate, field.path))
    return { status: "empty", clause: null };
  if (commonShape(predicate, field)) return { status: "editable", clause: predicate };
  if (predicate.kind !== "all_of") return { status: "advanced", clause: null };
  const matches = predicate.children.filter((child) =>
    predicateMentionsField(child, field.path),
  );
  return matches.length === 1 && commonShape(matches[0], field)
    ? { status: "editable", clause: matches[0] }
    : field.field_type !== "number" &&
        matches.length === 2 &&
        commonShape({ kind: "all_of", children: matches }, field)
      ? { status: "editable", clause: { kind: "all_of", children: matches } }
      : { status: "advanced", clause: null };
}
export function replaceCommonCondition(
  predicate: QueryPredicate | null,
  field: QueryFieldDefinition,
  replacement: QueryPredicate | null,
): QueryPredicate | null {
  const current = commonCondition(predicate, field);
  if (current.status === "advanced")
    throw new Error("Edit this condition in advanced filters");
  if (current.clause === predicate) return replacement;
  if (predicate?.kind === "all_of") {
    const removed =
      current.clause?.kind === "all_of" && !predicate.children.includes(current.clause)
        ? current.clause.children
        : current.clause
          ? [current.clause]
          : [];
    let inserted = false;
    const children = predicate.children.flatMap((child) =>
      removed.includes(child)
        ? !inserted && replacement
          ? ((inserted = true), [replacement])
          : []
        : [child],
    );
    if (!current.clause && replacement) children.push(replacement);
    return children.length ? { ...predicate, children } : null;
  }
  if (!replacement) return predicate;
  return predicate
    ? { kind: "all_of", children: [predicate, replacement] }
    : replacement;
}
export function isExcluded(clause: QueryPredicate): boolean {
  return (
    clause.kind === "not" ||
    (clause.kind === "set_match" && clause.op === "excludes_any")
  );
}
export function choiceClauses(clause: QueryPredicate | null): QueryPredicate[] {
  return clause?.kind === "all_of" ? clause.children : clause ? [clause] : [];
}
export type ChoiceState = "neutral" | "include" | "exclude";
export function choiceValueState(
  clause: QueryPredicate | null,
  value: string,
): ChoiceState {
  const match = choiceClauses(clause).find((child) =>
    conditionValues(child).includes(value),
  );
  return match ? (isExcluded(match) ? "exclude" : "include") : "neutral";
}
/** Change one literal while retaining the other literals and each surviving leaf ID. */
export function changeChoiceValue(
  field: QueryFieldDefinition,
  current: QueryPredicate | null,
  value: string,
  state: ChoiceState,
): QueryPredicate | null {
  const clauses = choiceClauses(current);
  const source = clauses.find((child) => conditionValues(child).includes(value));
  const children = [false, true].flatMap((exclude) => {
    const previous = clauses.find((child) => isExcluded(child) === exclude);
    const values = conditionValues(previous ?? null).filter((entry) => entry !== value);
    if (state === (exclude ? "exclude" : "include")) values.push(value);
    const identity =
      previous ?? (source && conditionValues(source).length === 1 ? source : null);
    const next = choiceCondition(field, identity, values, exclude);
    return next ? [next] : [];
  });
  return children.length > 1
    ? {
        kind: "all_of",
        clause_id: current?.kind === "all_of" ? current.clause_id : crypto.randomUUID(),
        children,
      }
    : (children[0] ?? null);
}

/** Only remove recognized common clauses from a positive root conjunction. */
export function advancedResidual(
  predicate: QueryPredicate | null,
  fields: QueryFieldDefinition[],
): { common: QueryPredicate[]; residual: QueryPredicate | null } {
  const common = fields.flatMap((field) => {
    const recognized = commonCondition(predicate, field);
    if (!recognized.clause) return [];
    if (
      recognized.clause === predicate ||
      predicate?.kind !== "all_of" ||
      predicate.children.includes(recognized.clause)
    )
      return [recognized.clause];
    return choiceClauses(recognized.clause);
  });
  if (common.includes(predicate!)) return { common, residual: null };
  if (predicate?.kind !== "all_of") return { common, residual: predicate };
  const children = predicate.children.filter((child) => !common.includes(child));
  return { common, residual: children.length ? { ...predicate, children } : null };
}
export function replaceAdvancedResidual(
  predicate: QueryPredicate | null,
  fields: QueryFieldDefinition[],
  replacement: QueryPredicate | null,
): QueryPredicate | null {
  const { common, residual } = advancedResidual(predicate, fields);
  if (!common.length) return replacement;
  if (predicate?.kind !== "all_of" || common.includes(predicate))
    return replacement
      ? { kind: "all_of", children: [...common, replacement] }
      : predicate;
  const replacements =
    replacement?.kind === "all_of" && replacement.clause_id === residual?.clause_id
      ? replacement.children
      : replacement
        ? [replacement]
        : [];
  let inserted = false;
  const children = predicate.children.flatMap((child) => {
    if (common.includes(child)) return [child];
    if (inserted) return [];
    inserted = true;
    return replacements;
  });
  if (!inserted) children.push(...replacements);
  return children.length ? { ...predicate, children } : null;
}
export function conditionLeaf(clause: QueryPredicate | null): QueryPredicate | null {
  return clause?.kind === "not" ? clause.predicate : clause;
}
export function choiceCondition(
  field: QueryFieldDefinition,
  current: QueryPredicate | null,
  values: string[],
  exclude: boolean,
): QueryPredicate | null {
  if (!values.length) return null;
  const leaf = conditionLeaf(current);
  const clause_id = leaf?.clause_id ?? crypto.randomUUID();
  if (field.field_type === "set")
    return {
      kind: "set_match",
      clause_id,
      field: field.path,
      op: exclude ? "excludes_any" : "includes_any",
      values,
    };
  const predicate: QueryPredicate = {
    kind: "in",
    clause_id,
    field: field.path,
    values,
  };
  return exclude
    ? {
        kind: "not",
        ...(current?.kind === "not" && current.clause_id
          ? { clause_id: current.clause_id }
          : {}),
        predicate,
      }
    : predicate;
}
export function conditionValues(clause: QueryPredicate | null): string[] {
  const leaf = conditionLeaf(clause);
  return leaf && "values" in leaf
    ? leaf.values.map(String)
    : leaf?.kind === "compare"
      ? [String(leaf.value)]
      : [];
}
export function conditionRange(clause: QueryPredicate | null): [string, string] {
  const children = clause?.kind === "all_of" ? clause.children : clause ? [clause] : [];
  let min = "",
    max = "";
  for (const child of children)
    if (child.kind === "compare") {
      if (child.op === "gte" || child.op === "eq") min = String(child.value);
      if (child.op === "lte" || child.op === "eq") max = String(child.value);
    }
  return [min, max];
}

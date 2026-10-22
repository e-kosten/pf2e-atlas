import { useMemo, useState } from "react";
import type { RuleGroupType, RuleType } from "react-querybuilder";
import type {
  FilterEditorFieldView,
  QueryFieldDefinition,
  QueryRelationship,
  QueryPredicate,
} from "../../generated/atlas";
import type { SearchFormState } from "./searchState";
import { encodeFilterUrl } from "./filterUrl";
import {
  builderPredicate,
  predicateBuilder,
  commonDraftCondition,
  LINK_CONDITION,
} from "./queryBuilder";
import { appendLink, linkLeaf, sameRelationship } from "./linkPredicates";
import { commonCondition, replaceCommonCondition } from "./filterControls";

type EditorDraft = {
  baseline: string;
  query: RuleGroupType;
  // Committed occurrence path → retained draft rule ID, scoped to baseline.
  linkRules: ReadonlyMap<string, string>;
};

/** Committed syntax paths identify occurrences even when URL decoding replaces IDs. */
function linkClausePaths(
  predicate: QueryPredicate | null,
  path = "root",
): Array<[string, string]> {
  if (!predicate) return [];
  if (predicate.kind === "relationship")
    return predicate.clause_id ? [[path, predicate.clause_id]] : [];
  if (predicate.kind === "all_of" || predicate.kind === "any_of")
    return predicate.children.flatMap((child, index) =>
      linkClausePaths(child, `${path}/children/${index}`),
    );
  if (predicate.kind === "not" || predicate.kind === "exists")
    return linkClausePaths(predicate.predicate, `${path}/predicate`);
  return [];
}
function draftLinkIds(query: RuleGroupType): Set<string> {
  return new Set(
    query.rules.flatMap((node): string[] =>
      "rules" in node
        ? [...draftLinkIds(node)]
        : node.field === LINK_CONDITION && node.id
          ? [node.id]
          : [],
    ),
  );
}
const criteriaIdentity = (search: SearchFormState) =>
  JSON.stringify([
    search.filter ? encodeFilterUrl(search.filter) : "",
    search.filterError,
  ]);

/** The execution predicate is a projection, not a replacement for editing state. */
export function useFilterEditor(
  search: SearchFormState,
  fields: FilterEditorFieldView[],
  commit: (next: SearchFormState) => void,
) {
  const definitions = useMemo(() => fields.map((field) => field.definition), [fields]);
  // Record-preview navigation restores IDs from the URL. Only changed criteria
  // invalidate a draft; ephemeral clause IDs are not query meaning.
  const baseline = criteriaIdentity(search);
  const [draft, setDraft] = useState<EditorDraft | null>(null);
  if (draft && draft.baseline !== baseline) setDraft(null);
  const prepared = useMemo(() => {
    try {
      const query =
        draft?.baseline === baseline
          ? draft.query
          : predicateBuilder(search.filter, definitions);
      let error = search.filterError;
      try {
        builderPredicate(query, definitions);
      } catch (failure) {
        error ||= failure instanceof Error ? failure.message : String(failure);
      }
      const linkRuleIds = new Map(
        linkClausePaths(search.filter).flatMap(
          ([path, id]): Array<[string, string]> => {
            const draftId =
              draft?.baseline === baseline ? draft.linkRules.get(path) : id;
            return draftId ? [[id, draftId]] : [];
          },
        ),
      );
      return { query, error, linkRuleIds };
    } catch (error) {
      return {
        query: null,
        linkRuleIds: new Map<string, string>(),
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }, [baseline, definitions, draft, search.filter, search.filterError]);

  function retain(
    query: RuleGroupType,
    next: SearchFormState = search,
    committedFromDraft = false,
  ) {
    // A valid projection carries draft IDs; partial edits keep the existing
    // clause binding even when navigation has assigned fresh committed IDs.
    const ids = draftLinkIds(query);
    const linkRules = new Map<string, string>();
    for (const [path, id] of linkClausePaths(next.filter)) {
      const draftId = committedFromDraft ? id : (prepared.linkRuleIds.get(id) ?? id);
      if (ids.has(draftId)) linkRules.set(path, draftId);
    }
    setDraft({ baseline: criteriaIdentity(next), query, linkRules });
  }
  function change(query: RuleGroupType) {
    try {
      const filter = builderPredicate(query, definitions);
      const next = { ...search, filter, filterError: null };
      retain(query, next, true);
      commit(next);
    } catch {
      retain(query);
    }
  }
  function changeCommon(query: RuleGroupType, field: QueryFieldDefinition) {
    // A completed common edit updates its own committed condition while an
    // unfinished unrelated rule retains its last executable value.
    try {
      builderPredicate(query, definitions);
      change(query);
      return;
    } catch {
      const condition = commonDraftCondition(query, field);
      if (
        condition.status === "advanced" ||
        commonCondition(search.filter, field).status === "advanced"
      ) {
        retain(query);
        return;
      }
      const next = {
        ...search,
        filter: replaceCommonCondition(search.filter, field, condition.clause),
        filterError: null,
      };
      retain(query, next);
      commit(next);
    }
  }
  function addLink(
    relationship: QueryRelationship,
    clause_id: string = crypto.randomUUID(),
  ): string | null {
    if (!prepared.query) return "Reset the filters before adding a link condition.";
    const leaf = linkLeaf(relationship, clause_id);
    const filter = appendLink(search.filter, leaf);
    if (filter === search.filter) return "This link condition is already added.";
    const rule = {
      id: leaf.clause_id,
      field: LINK_CONDITION,
      operator: relationship.direction,
      value: relationship,
    };
    const query =
      prepared.query.combinator === "and" && !prepared.query.not
        ? { ...prepared.query, rules: [...prepared.query.rules, rule] }
        : { id: crypto.randomUUID(), combinator: "and", rules: [prepared.query, rule] };
    const next = { ...search, filter, filterError: null };
    retain(query, next);
    commit(next);
    return null;
  }
  function editLink(
    clause: Extract<QueryPredicate, { kind: "relationship" }>,
    relationship: QueryRelationship | null,
  ): string | null {
    if (relationship && sameRelationship(clause.relationship, relationship))
      return null;
    const direct =
      search.filter?.kind === "all_of"
        ? search.filter.children
        : search.filter
          ? [search.filter]
          : [];
    if (
      relationship &&
      direct.some(
        (node) =>
          node.kind === "relationship" &&
          node.clause_id !== clause.clause_id &&
          sameRelationship(node.relationship, relationship),
      )
    )
      return "This link condition is already added.";
    function replace(node: QueryPredicate): QueryPredicate | null {
      if (node.kind === "relationship" && node.clause_id === clause.clause_id)
        return relationship ? { ...node, relationship } : null;
      if (node.kind !== "all_of") return node;
      const children = node.children.flatMap((child) => {
        const next = replace(child);
        return next ? [next] : [];
      });
      return children.length ? { ...node, children } : null;
    }
    const draftClauseId = clause.clause_id
      ? prepared.linkRuleIds.get(clause.clause_id)
      : undefined;
    function replaceDraft(group: RuleGroupType): RuleGroupType {
      return {
        ...group,
        rules: group.rules.flatMap((node): Array<RuleType | RuleGroupType> => {
          if ("rules" in node) return [replaceDraft(node)];
          if (
            node.field !== LINK_CONDITION ||
            !draftClauseId ||
            node.id !== draftClauseId
          )
            return [node];
          return relationship
            ? [{ ...node, operator: relationship.direction, value: relationship }]
            : [];
        }),
      };
    }
    const next = {
      ...search,
      filter: search.filter ? replace(search.filter) : null,
      filterError: null,
    };
    if (prepared.query) retain(replaceDraft(prepared.query), next);
    commit(next);
    return null;
  }
  return {
    query: prepared.query,
    error: prepared.error,
    change,
    changeCommon,
    addLink,
    editLink,
    clear: () => setDraft(null),
  };
}

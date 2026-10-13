import { useMemo, useState } from "react";
import type { RuleGroupType } from "react-querybuilder";
import type {
  FilterEditorFieldView,
  QueryFieldDefinition,
} from "../../generated/atlas";
import type { SearchFormState } from "./searchState";
import { encodeFilterUrl } from "./filterUrl";
import {
  builderPredicate,
  predicateBuilder,
  commonDraftCondition,
} from "./queryBuilder";
import { commonCondition, replaceCommonCondition } from "./filterControls";

type EditorDraft = {
  baseline: string;
  query: RuleGroupType;
};
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
      return { query, error };
    } catch (error) {
      return {
        query: null,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }, [baseline, definitions, draft, search.filter, search.filterError]);

  function retain(query: RuleGroupType, next: SearchFormState = search) {
    setDraft({
      baseline: criteriaIdentity(next),
      query,
    });
  }
  function change(query: RuleGroupType) {
    try {
      const filter = builderPredicate(query, definitions);
      const next = { ...search, filter, filterError: null };
      retain(query, next);
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
  return { ...prepared, change, changeCommon, clear: () => setDraft(null) };
}

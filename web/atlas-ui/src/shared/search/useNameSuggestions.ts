import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { searchExact, searchPage } from "../../api/atlasApi";
import type { QueryPredicate } from "../../generated/atlas";
import { referenceQueryMeta } from "../queries/referenceQueries";
import { useReferenceQueryRetirement } from "../queries/useReferenceQueryRetirement";
import { assertKeywordSearch, assertSearchIdentity } from "./searchIdentity";

export type NameSuggestionContext = {
  enabled: boolean;
  filter: QueryPredicate | null;
  generation: string | null;
  epoch: number;
  onSelect: (key: string, generation: string) => void;
};

export function useNameSuggestions(
  context: NameSuggestionContext | undefined,
  draft: string,
) {
  const [debounced, setDebounced] = useState("");
  useEffect(() => {
    if (!draft) return;
    const timer = window.setTimeout(() => setDebounced(draft), 140);
    return () => window.clearTimeout(timer);
  }, [draft]);
  const enabled =
    context?.enabled === true &&
    draft === debounced &&
    Array.from(draft.trim()).length >= 2;
  const filter = context?.filter ?? null;
  const generation = context?.generation ?? null;
  const key = ["search-name-suggestions", draft, filter, generation, context?.epoch];
  const result = useQuery({
    queryKey: key,
    meta: referenceQueryMeta,
    enabled,
    retry: false,
    queryFn: async ({ signal }) => {
      const page = await searchPage(
        {
          criteria: {
            kind: "text_search",
            mode: "lexical",
            query: draft,
            filter,
            lexical: { fields: "record_names", match: "last_token_prefix" },
          },
          page: { number: 1, size: 5 },
          result_detail: "records",
          expected_artifact_generation: generation,
        },
        signal,
      );
      assertKeywordSearch(page);
      assertSearchIdentity(page, undefined, generation);
      const { artifact_generation } = page;
      if (!page.page.has_more) return { artifact_generation, rows: page.rows };

      const exact = await searchExact(
        { query: draft, filter, expected_artifact_generation: artifact_generation },
        signal,
      );
      assertSearchIdentity(exact, undefined, artifact_generation);
      return exact.total === 1 ? { artifact_generation, rows: exact.matches } : null;
    },
  });
  useReferenceQueryRetirement(key);
  return enabled && result.isSuccess && !result.isFetching ? result.data : undefined;
}

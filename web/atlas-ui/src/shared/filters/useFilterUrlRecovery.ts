import { atlasRouter } from "../../app/router";
import { useEffect, useRef } from "react";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "./searchState";

export const FILTER_URL_RESET_NOTICE =
  "This link couldn’t be restored. Search has been reset.";

/** Recover only failed transport restoration, never a backend or retrieval error. */
export function useFilterUrlRecovery(
  search: SearchFormState,
  onRecovered?: (search: SearchFormState) => void,
) {
  const handled = useRef(false);
  useEffect(() => {
    if (!search.filterWasReset) {
      handled.current = false;
      return;
    }
    if (handled.current) return;
    handled.current = true;
    void atlasRouter
      .navigate({
        to: "/search",
        search: {},
        replace: true,
        resetScroll: false,
        state: (state) => ({
          ...state,
          atlasSearchReset: crypto.randomUUID(),
          atlasFilterResetNotice: true,
        }),
      })
      .then(() => onRecovered?.({ ...DEFAULT_SEARCH_STATE }));
  }, [search, onRecovered]);
}

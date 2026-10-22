import { App } from "antd";
import { useEffect, useRef } from "react";
import type { SearchFormState } from "./searchState";

export const FILTER_URL_RESET_NOTICE =
  "Filters in this link couldn’t be restored and were cleared.";

/** Recover only failed transport restoration, never a backend or retrieval error. */
export function useFilterUrlRecovery(
  search: SearchFormState,
  onRecovered: (search: SearchFormState) => void,
) {
  const { message } = App.useApp();
  const handled = useRef(false);
  useEffect(() => {
    if (!search.filterWasReset) {
      handled.current = false;
      return;
    }
    if (handled.current) return;
    handled.current = true;
    const url = new URL(window.location.href);
    url.searchParams.delete("filter");
    url.searchParams.delete("relationships");
    history.replaceState(
      { ...history.state, atlasSearchPage: 1 },
      "",
      url.pathname + url.search + url.hash,
    );
    const { filterWasReset: _reset, ...restored } = search;
    onRecovered(restored);
    void message.info({ key: "filter-url-reset", content: FILTER_URL_RESET_NOTICE });
  }, [search, message, onRecovered]);
}

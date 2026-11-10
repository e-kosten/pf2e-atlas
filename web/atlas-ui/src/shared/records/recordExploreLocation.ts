import { useCallback, useRef } from "react";
import { useRouter, useRouterState } from "@tanstack/react-router";
import { atlasRouter } from "../../app/router";
import { readingView, type RecordExploreTab } from "../../app/urlState";

/** Reading mode and the active exploration task travel with this history entry. */
export function useRecordExploreLocation() {
  const contextRouter = useRouter({ warn: false });
  const router = contextRouter ?? atlasRouter;
  const search = useRouterState({ router, select: (state) => state.location.search });
  const { view: mode = "read", tab } = readingView(search);
  const resetPending = useRef(false);
  const update = useCallback(
    (mode: "read" | "explore", tab: RecordExploreTab | undefined, replace = false) => {
      const next = { ...router.latestLocation.search };
      if (mode === "explore") next.record_view = "explore";
      else delete next.record_view;
      if (tab) next.record_tab = tab;
      else delete next.record_tab;
      return router.navigate({
        to: ".",
        search: next,
        replace,
        hash: true,
        resetScroll: false,
        state: true,
      });
    },
    [router],
  );
  const resetUnavailableVersions = useCallback(() => {
    const view = readingView(router.latestLocation.search);
    if (view.view !== "explore" || view.tab !== "versions" || resetPending.current)
      return;
    resetPending.current = true;
    void update("read", undefined, true).finally(() => {
      resetPending.current = false;
    });
  }, [router, update]);
  return {
    mode,
    tab,
    resetUnavailableVersions,
    setMode: (mode: "read" | "explore") =>
      void update(mode, readingView(router.latestLocation.search).tab),
    setTab: (tab: string) =>
      void update("explore", readingView({ record_tab: tab }).tab, true),
  };
}

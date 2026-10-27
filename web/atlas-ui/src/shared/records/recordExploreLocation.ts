import { useRouter, useRouterState } from "@tanstack/react-router";
import { atlasRouter } from "../../app/router";
import { readingView, type RecordExploreTab } from "../../app/urlState";

/** Reading mode and the active exploration task travel with this history entry. */
export function useRecordExploreLocation() {
  const contextRouter = useRouter({ warn: false });
  const router = contextRouter ?? atlasRouter;
  const search = useRouterState({ router, select: (state) => state.location.search });
  const { view: mode = "read", tab = "incoming" } = readingView(search);
  function update(
    mode: "read" | "explore",
    tab: RecordExploreTab = readingView(router.latestLocation.search).tab ?? "incoming",
    replace = false,
  ) {
    const next = { ...router.latestLocation.search };
    if (mode === "explore") next.record_view = "explore";
    else delete next.record_view;
    if (tab !== "incoming") next.record_tab = tab;
    else delete next.record_tab;
    void router.navigate({
      to: ".",
      search: next,
      replace,
      resetScroll: false,
      state: true,
    });
  }
  return {
    mode,
    tab,
    setMode: (mode: "read" | "explore") => update(mode),
    setTab: (tab: string) =>
      update("explore", readingView({ record_tab: tab }).tab, true),
  };
}

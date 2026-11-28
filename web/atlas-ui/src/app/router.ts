import { useRouter } from "@tanstack/react-router";
import { createRouter, type RouterHistory } from "@tanstack/react-router";
import { routeTree } from "../routeTree.gen";
import { connectMacHistory } from "./desktopMacHistory";
import { parseSearchParams, stringifySearchParams } from "./urlState";

export function createAtlasRouter(
  history?: RouterHistory,
  scrollRestoration: boolean | (() => boolean) = true,
) {
  const router = createRouter({
    history,
    routeTree,
    parseSearch: parseSearchParams,
    stringifySearch: stringifySearchParams,
    pathParamsAllowedCharacters: [":"],
    defaultPreload: false,
    defaultPendingMs: 500,
    scrollRestoration: scrollRestoration === true,
  });
  if (typeof scrollRestoration === "function") {
    // The per-tab predicate gates render-time scrolling. Initializing with it
    // would also register document listeners that keep closed routers alive.
    router.update({ ...router.options, scrollRestoration });
  }
  return router;
}

export const atlasRouter = createAtlasRouter();
connectMacHistory(atlasRouter.history);
export function useAtlasRouter() {
  return useRouter({ warn: false }) ?? atlasRouter;
}

export type AtlasSection =
  "search" | "lists" | "encounters" | "characters" | "reference";
export type AtlasRouter = ReturnType<typeof createAtlasRouter>;

declare module "@tanstack/react-router" {
  interface StaticDataRouteOption {
    section?: AtlasSection;
  }
  interface Register {
    router: AtlasRouter;
  }
  interface HistoryState {
    atlasSearchReset?: string;
    atlasComparisonControl?: {
      seed: import("../generated/atlas").SimilaritySeed;
      open: import("../shared/search/SimilarityFailure").ComparisonControl;
    };
    atlasReadingDismissal?: string;
    atlasCharacterOrigin?: { href: string; index: number };
    atlasFilterResetNotice?: boolean;
    atlasOwnerOrigin?: { recordKey: string; href: string };
  }
}

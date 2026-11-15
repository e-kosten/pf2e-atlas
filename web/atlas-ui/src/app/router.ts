import { useRouter } from "@tanstack/react-router";
import { createRouter, type RouterHistory } from "@tanstack/react-router";
import { routeTree } from "../routeTree.gen";
import { parseSearchParams, stringifySearchParams } from "./urlState";

export function createAtlasRouter(
  history?: RouterHistory,
  scrollRestoration: boolean | (() => boolean) = true,
) {
  return createRouter({
    history,
    routeTree,
    parseSearch: parseSearchParams,
    stringifySearch: stringifySearchParams,
    pathParamsAllowedCharacters: [":"],
    defaultPreload: false,
    defaultPendingMs: 500,
    scrollRestoration,
  });
}

export const atlasRouter = createAtlasRouter();
export function useAtlasRouter() {
  return useRouter({ warn: false }) ?? atlasRouter;
}

export type AtlasRouter = ReturnType<typeof createAtlasRouter>;

declare module "@tanstack/react-router" {
  interface StaticDataRouteOption {
    section?: "search" | "lists" | "encounters" | "reference";
  }
  interface Register {
    router: AtlasRouter;
  }
  interface HistoryState {
    atlasSearchReset?: string;
    atlasReadingDismissal?: string;
    atlasFilterResetNotice?: boolean;
    atlasOwnerOrigin?: { recordKey: string; href: string };
  }
}

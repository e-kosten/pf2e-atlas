import { createRouter, type RouterHistory } from "@tanstack/react-router";
import { routeTree } from "../routeTree.gen";
import { parseSearchParams, stringifySearchParams } from "./urlState";

export function createAtlasRouter(history?: RouterHistory) {
  return createRouter({
    history,
    routeTree,
    parseSearch: parseSearchParams,
    stringifySearch: stringifySearchParams,
    pathParamsAllowedCharacters: [":"],
    defaultPreload: false,
    defaultPendingMs: 500,
    scrollRestoration: true,
  });
}

export const atlasRouter = createAtlasRouter();
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
    atlasFilterResetNotice?: boolean;
    atlasOwnerOrigin?: { recordKey: string; href: string };
  }
}

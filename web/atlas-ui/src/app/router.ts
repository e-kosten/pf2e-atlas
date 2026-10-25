import { createRouter } from "@tanstack/react-router";
import { routeTree } from "../routeTree.gen";
import { parseSearchParams, stringifySearchParams } from "./urlState";

export const atlasRouter = createRouter({
  routeTree,
  parseSearch: parseSearchParams,
  stringifySearch: stringifySearchParams,
  pathParamsAllowedCharacters: [":"],
  defaultPreload: false,
  defaultPendingMs: 500,
  scrollRestoration: true,
});

export type AtlasRouter = typeof atlasRouter;

declare module "@tanstack/react-router" {
  interface StaticDataRouteOption {
    section?: "search" | "lists" | "encounters" | "reference";
  }
  interface Register {
    router: AtlasRouter;
  }
  interface HistoryState {
    atlasSearchPage?: number;
    atlasOwnerOrigin?: { recordKey: string; href: string };
  }
}

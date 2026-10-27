import { redirect, type ParsedLocation } from "@tanstack/react-router";
import { normalizeTags } from "../features/lists/listUtils";
import {
  decodeSearchStateFromParams,
  parseRandomSeed,
} from "../shared/filters/searchState";
import { encodeReadingAddress } from "./readingUrl";
import {
  readingView,
  selectedNavigation,
  stringifySearchParams,
  validateReadingSearch,
  type AtlasSearchParams,
} from "./urlState";

export type RouteSearchKind =
  | "search"
  | "search_record"
  | "list"
  | "list_record"
  | "record"
  | "trait"
  | "lists"
  | "encounters"
  | "empty";
export type ListIndexInput = { query?: string; include?: string[]; exclude?: string[] };
export type EncounterIndexInput = { view?: "active" | "archived" };

export class RouteSearchError extends Error {
  constructor(parameter: string) {
    super(`This link contains an invalid or conflicting ${parameter} parameter.`);
    this.name = "RouteSearchError";
  }
}

function singleton(search: AtlasSearchParams, key: string): string | undefined {
  const value = search[key];
  if (!Array.isArray(value)) return value;
  if (value.some((entry) => entry !== value[0])) throw new RouteSearchError(key);
  return value[0];
}

function integer(
  value: string | undefined,
  min: number,
  max: number,
  fallback: number,
) {
  if (!value || !/^\d+$/.test(value)) return fallback;
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed >= min && parsed <= max
    ? parsed
    : fallback;
}

export function searchPage(search: AtlasSearchParams): number {
  const value = search.page;
  if (Array.isArray(value) && value.some((entry) => entry !== value[0])) return 1;
  return integer(Array.isArray(value) ? value[0] : value, 1, 4_294_967_295, 1);
}

function tags(search: AtlasSearchParams, key: string): string[] {
  const value = search[key];
  return normalizeTags(
    Array.isArray(value) ? value : value === undefined ? [] : [value],
  );
}

export function listIndexSearch(search: AtlasSearchParams) {
  return {
    query: singleton(search, "q")?.trim() ?? "",
    include: tags(search, "tag"),
    exclude: tags(search, "exclude_tag"),
  };
}

export function encounterIndexSearch(
  search: AtlasSearchParams,
): Required<EncounterIndexInput> {
  const value = singleton(search, "view");
  if (value !== undefined && value !== "active" && value !== "archived")
    throw new RouteSearchError("view");
  return { view: value === "archived" ? "archived" : "active" };
}

function readingSearch(search: AtlasSearchParams): AtlasSearchParams {
  validateReadingSearch(search);
  const selected = selectedNavigation(search, "");
  const landing = selectedNavigation(search, "", "focus");
  const result: AtlasSearchParams = {};
  if (selected) {
    if (selected.spell_rank !== undefined) result.rank = String(selected.spell_rank);
    const at = encodeReadingAddress("", selected, false);
    if (at !== undefined) result.at = at;
  }
  if (landing) {
    const focus = encodeReadingAddress("", landing, true);
    if (focus !== undefined) result.focus = focus;
  }
  const view = readingView(search);
  if (view.view === "explore") result.record_view = "explore";
  if (view.tab !== "incoming") result.record_tab = view.tab;
  return result;
}

function filterTransport(search: AtlasSearchParams, key: "filter" | "relationships") {
  const value = search[key];
  return Array.isArray(value) && value.every((entry) => entry === value[0])
    ? value[0]
    : value;
}

export function canonicalRouteSearch(
  kind: RouteSearchKind,
  search: AtlasSearchParams,
): AtlasSearchParams {
  if (failedFilterTransport(kind, search)) return search;
  const result: AtlasSearchParams = {};
  if (kind === "empty") return result;
  if (kind === "lists") {
    const state = listIndexSearch(search);
    if (state.query) result.q = state.query;
    if (state.include.length) result.tag = state.include;
    if (state.exclude.length) result.exclude_tag = state.exclude;
    return result;
  }
  if (kind === "encounters") {
    if (encounterIndexSearch(search).view === "archived") result.view = "archived";
    return result;
  }
  const isSearch = kind === "search" || kind === "search_record";
  const isList = kind === "list" || kind === "list_record";
  const reading =
    kind === "record" || kind === "search_record" || kind === "list_record";
  const exact = reading ? readingSearch(search) : {};
  if (isSearch || kind === "trait") {
    const profile = singleton(search, "search_profile")?.trim();
    if (profile) result.search_profile = profile;
  }
  if (isSearch) {
    const ranking = singleton(search, "ranking");
    if (ranking !== undefined && ranking !== "score") result.ranking = ranking;
  }
  if (isSearch || isList) {
    const q = singleton(search, "q")?.trim();
    if (q) result.q = q;
    if (isSearch) {
      const similar = singleton(search, "similar")?.trim();
      const traitRefs = singleton(search, "trait_refs")?.trim();
      if (similar) result.similar = similar;
      if (traitRefs) result.trait_refs = traitRefs;
    }
    for (const key of ["filter", "relationships"] as const) {
      const value = filterTransport(search, key);
      if (value !== undefined) result[key] = value;
    }
  }
  if (kind === "trait") {
    const tab = singleton(search, "tab");
    if (tab !== undefined && !["members", "references", "similar"].includes(tab))
      throw new RouteSearchError("tab");
    if (tab && tab !== "members") result.tab = tab;
  }
  if (isSearch || isList || kind === "trait") {
    const sort = search.sort;
    if (sort !== undefined) result.sort = sort;
    const seed = singleton(search, "seed");
    if (seed !== undefined) result.seed = String(parseRandomSeed(seed) ?? seed);
  }
  if (isSearch) {
    const size = integer(singleton(search, "limit"), 1, 100, 25);
    if (size !== 25) result.limit = String(size);
    singleton(search, "page");
    const page = searchPage(search);
    if (page !== 1) result.page = String(page);
  }
  return { ...result, ...exact };
}

export function failedFilterTransport(
  kind: RouteSearchKind,
  search: AtlasSearchParams,
): boolean {
  if (!["search", "search_record", "list", "list_record"].includes(kind)) return false;
  if (search.filter === undefined && search.relationships === undefined) return false;
  return (
    decodeSearchStateFromParams(new URLSearchParams(stringifySearchParams(search)))
      .filterWasReset === true
  );
}

export function routeSearch(kind: RouteSearchKind) {
  return {
    validateSearch: (search: AtlasSearchParams) => canonicalRouteSearch(kind, search),
    beforeLoad: ({ location }: { location: ParsedLocation }) => {
      if (failedFilterTransport(kind, location.search)) return;
      const search = canonicalRouteSearch(kind, location.search);
      const query = stringifySearchParams(search);
      if (query !== location.searchStr) {
        const hash = location.href.includes("#")
          ? location.href.slice(location.href.indexOf("#"))
          : "";
        throw redirect({
          href: location.pathname + query + hash,
          replace: true,
          resetScroll: false,
          state: true,
        });
      }
    },
  };
}

import type React from "react";
import type { RecordNavigationView } from "../generated/atlas";

export type AtlasRoute =
  | { kind: "invalid"; message: string }
  | { kind: "search"; selectedRecordKey: string | null }
  | { kind: "encounters" }
  | { kind: "encounter"; slug: string }
  | { kind: "encounterEdit"; slug: string }
  | { kind: "lists" }
  | { kind: "list"; slug: string; selectedRecordKey: string | null }
  | { kind: "listEdit"; slug: string }
  | { kind: "record"; recordKey: string; selection?: RecordNavigationView }
  | {
      kind: "reader";
      recordKey: string;
      previewRecordKey: string | null;
      selection?: RecordNavigationView;
      previewSelection?: RecordNavigationView;
    };

export const ATLAS_ROUTE_CHANGE_EVENT = "atlas-route-change";

export function parseAtlasRoute(pathname: string, search = ""): AtlasRoute {
  try {
    return parseRoute(pathname, search);
  } catch (error) {
    return {
      kind: "invalid",
      message: error instanceof Error ? error.message : "Invalid navigation URL",
    };
  }
}

function parseRoute(pathname: string, search = ""): AtlasRoute {
  const searchRecord = pathname.match(/^\/search\/records\/(.+)$/);
  if (searchRecord) {
    return {
      kind: "search",
      selectedRecordKey: decodeURIComponent(searchRecord[1]),
    };
  }

  const record = pathname.match(/^\/records\/(.+)$/);
  if (record) {
    return {
      kind: "record",
      recordKey: decodeURIComponent(record[1]),
      selection: selectedNavigation(search),
    };
  }

  if (pathname === "/lists") {
    return { kind: "lists" };
  }

  if (pathname === "/encounters") {
    return { kind: "encounters" };
  }

  const encounterEdit = pathname.match(/^\/encounters\/(.+)\/edit$/);
  if (encounterEdit) {
    return {
      kind: "encounterEdit",
      slug: decodeURIComponent(encounterEdit[1]),
    };
  }

  const encounter = pathname.match(/^\/encounters\/(.+)$/);
  if (encounter) {
    return { kind: "encounter", slug: decodeURIComponent(encounter[1]) };
  }

  const listEdit = pathname.match(/^\/lists\/(.+)\/edit$/);
  if (listEdit) {
    return {
      kind: "listEdit",
      slug: decodeURIComponent(listEdit[1]),
    };
  }

  const listRecord = pathname.match(/^\/lists\/(.+)\/records\/(.+)$/);
  if (listRecord) {
    return {
      kind: "list",
      slug: decodeURIComponent(listRecord[1]),
      selectedRecordKey: decodeURIComponent(listRecord[2]),
    };
  }

  const list = pathname.match(/^\/lists\/(.+)$/);
  if (list) {
    return {
      kind: "list",
      slug: decodeURIComponent(list[1]),
      selectedRecordKey: null,
    };
  }

  const reader = pathname.match(/^\/reader\/(.+)$/);
  if (reader) {
    const previewRecordKey = new URLSearchParams(search).get("preview");
    return {
      kind: "reader",
      recordKey: decodeURIComponent(reader[1]),
      previewRecordKey,
      selection: selectedNavigation(search),
      previewSelection: selectedNavigation(search, "previewSelection"),
    };
  }

  return { kind: "search", selectedRecordKey: null };
}

export function currentAtlasRoute(): AtlasRoute {
  return parseAtlasRoute(window.location.pathname, window.location.search);
}

export function atlasRoutePath(route: AtlasRoute): string {
  switch (route.kind) {
    case "invalid":
      return "/search";
    case "search":
      return searchPath(route.selectedRecordKey);
    case "encounters":
      return encountersPath();
    case "encounter":
      return encounterPath(route.slug);
    case "encounterEdit":
      return encounterEditPath(route.slug);
    case "lists":
      return listsPath();
    case "list":
      return listPath(route.slug, route.selectedRecordKey);
    case "listEdit":
      return listEditPath(route.slug);
    case "record":
      return (
        recordPath(route.recordKey) +
        (route.selection
          ? "?selection=" + encodeURIComponent(JSON.stringify(route.selection))
          : "")
      );
    case "reader": {
      const path = readerPath(route.recordKey);
      const params = new URLSearchParams();
      if (route.previewRecordKey) params.set("preview", route.previewRecordKey);
      if (route.selection) params.set("selection", JSON.stringify(route.selection));
      if (route.previewSelection)
        params.set("previewSelection", JSON.stringify(route.previewSelection));
      return path + (params.size ? "?" + params.toString() : "");
    }
  }
}

export function navigateToAtlasRoute(route: AtlasRoute) {
  history.pushState(null, "", atlasRoutePath(route));
  window.dispatchEvent(new Event(ATLAS_ROUTE_CHANGE_EVENT));
}

export function shouldHandleAtlasRouteClick(
  event: React.MouseEvent<HTMLElement>,
): boolean {
  return (
    event.button === 0 &&
    !event.defaultPrevented &&
    !event.metaKey &&
    !event.altKey &&
    !event.ctrlKey &&
    !event.shiftKey
  );
}

export function searchPath(recordKey: string | null = null): string {
  return recordKey === null
    ? "/search"
    : `/search/records/${encodeURIComponent(recordKey)}`;
}

export function listsPath(): string {
  return "/lists";
}

export function encountersPath(): string {
  return "/encounters";
}

export function encounterPath(slug: string): string {
  return `/encounters/${encodeURIComponent(slug)}`;
}

export function encounterEditPath(slug: string): string {
  return `/encounters/${encodeURIComponent(slug)}/edit`;
}

export function listPath(slug: string, recordKey: string | null = null): string {
  const path = `/lists/${encodeURIComponent(slug)}`;
  return recordKey === null ? path : `${path}/records/${encodeURIComponent(recordKey)}`;
}

export function listEditPath(slug: string): string {
  return `/lists/${encodeURIComponent(slug)}/edit`;
}

export function recordPath(recordKey: string): string {
  return `/records/${encodeURIComponent(recordKey)}`;
}

export function readerPath(recordKey: string): string {
  return `/reader/${encodeURIComponent(recordKey)}`;
}

function selectedNavigation(
  search: string,
  key = "selection",
): RecordNavigationView | undefined {
  const value = new URLSearchParams(search).get(key);
  if (!value) return undefined;
  try {
    const parsed = JSON.parse(value);
    if (typeof parsed.record_key !== "string" || !Array.isArray(parsed.owners))
      throw new Error("Invalid record navigation");
    return parsed as RecordNavigationView;
  } catch {
    throw new Error("Invalid record navigation URL");
  }
}

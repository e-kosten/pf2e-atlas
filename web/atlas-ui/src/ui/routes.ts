import type React from "react";

export type AtlasRoute =
  | { kind: "search"; selectedRecordKey: string | null }
  | { kind: "record"; recordKey: string }
  | { kind: "reader"; recordKey: string; previewRecordKey: string | null };

export const ATLAS_ROUTE_CHANGE_EVENT = "atlas-route-change";

export function parseAtlasRoute(pathname: string, search = ""): AtlasRoute {
  const searchRecord = pathname.match(/^\/search\/records\/(.+)$/);
  if (searchRecord) {
    return {
      kind: "search",
      selectedRecordKey: decodeURIComponent(searchRecord[1]),
    };
  }

  const record = pathname.match(/^\/records\/(.+)$/);
  if (record) {
    return { kind: "record", recordKey: decodeURIComponent(record[1]) };
  }

  const reader = pathname.match(/^\/reader\/(.+)$/);
  if (reader) {
    const previewRecordKey = new URLSearchParams(search).get("preview");
    return {
      kind: "reader",
      recordKey: decodeURIComponent(reader[1]),
      previewRecordKey,
    };
  }

  return { kind: "search", selectedRecordKey: null };
}

export function currentAtlasRoute(): AtlasRoute {
  return parseAtlasRoute(window.location.pathname, window.location.search);
}

export function atlasRoutePath(route: AtlasRoute): string {
  switch (route.kind) {
    case "search":
      return searchPath(route.selectedRecordKey);
    case "record":
      return recordPath(route.recordKey);
    case "reader": {
      const path = readerPath(route.recordKey);
      return route.previewRecordKey === null
        ? path
        : `${path}?preview=${encodeURIComponent(route.previewRecordKey)}`;
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

export function recordPath(recordKey: string): string {
  return `/records/${encodeURIComponent(recordKey)}`;
}

export function readerPath(recordKey: string): string {
  return `/reader/${encodeURIComponent(recordKey)}`;
}

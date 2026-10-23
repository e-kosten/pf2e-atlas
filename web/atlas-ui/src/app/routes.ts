import type React from "react";
import type { RecordNavigationView } from "../generated/atlas";

export type AtlasRoute =
  | { kind: "invalid"; message: string }
  | { kind: "trait"; traitKey: string; query?: string }
  | {
      kind: "search";
      selectedRecordKey: string | null;
      selection?: RecordNavigationView;
      query?: string;
      landing?: RecordNavigationView;
    }
  | { kind: "encounters" }
  | { kind: "encounter"; slug: string }
  | { kind: "encounterEdit"; slug: string }
  | { kind: "lists" }
  | {
      kind: "list";
      slug: string;
      query?: string;
      selectedRecordKey: string | null;
      landing?: RecordNavigationView;
    }
  | { kind: "listEdit"; slug: string }
  | {
      kind: "record";
      section?: string;
      recordKey: string;
      selection?: RecordNavigationView;
      landing?: RecordNavigationView;
    };

export const ATLAS_ROUTE_CHANGE_EVENT = "atlas-route-change";

export function parseAtlasRoute(pathname: string, search = "", hash = ""): AtlasRoute {
  try {
    const route = parseRoute(pathname, search);
    if (route.kind === "record" && hash) {
      try {
        route.section = decodeURIComponent(hash.replace(/^#/, ""));
      } catch {
        route.section = hash.replace(/^#/, "");
      }
    }
    return route;
  } catch (error) {
    return {
      kind: "invalid",
      message: error instanceof Error ? error.message : "Invalid navigation URL",
    };
  }
}

function parseRoute(pathname: string, search = ""): AtlasRoute {
  const trait = pathname.match(/^\/traits\/(.+)$/);
  if (trait)
    return {
      kind: "trait",
      traitKey: decodeURIComponent(trait[1]),
      ...(search ? { query: search } : {}),
    };

  const searchRecord = pathname.match(/^\/search\/records\/(.+)$/);
  if (searchRecord) {
    return {
      kind: "search",
      selectedRecordKey: decodeURIComponent(searchRecord[1]),
      selection: selectedNavigation(search),
      landing: selectedNavigation(search, "landing"),
    };
  }

  const record = pathname.match(/^\/records\/(.+)$/);
  if (record) {
    return {
      kind: "record",
      recordKey: decodeURIComponent(record[1]),
      selection: selectedNavigation(search),
      landing: selectedNavigation(search, "landing"),
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
      ...listQuery(search),
      landing: selectedNavigation(search, "landing"),
    };
  }

  const list = pathname.match(/^\/lists\/(.+)$/);
  if (list) {
    return {
      kind: "list",
      slug: decodeURIComponent(list[1]),
      selectedRecordKey: null,
      ...listQuery(search),
    };
  }

  return { kind: "search", selectedRecordKey: null };
}

export function currentAtlasRoute(): AtlasRoute {
  return parseAtlasRoute(
    window.location.pathname,
    window.location.search,
    window.location.hash,
  );
}

export function atlasRoutePath(route: AtlasRoute): string {
  switch (route.kind) {
    case "trait":
      return `/traits/${recordKeyPathSegment(route.traitKey)}` + (route.query ?? "");
    case "invalid":
      return "/search";
    case "search":
      return withLanding(
        withSelection(
          searchPath(route.selectedRecordKey) + (route.query ?? ""),
          route.selection,
        ),
        route.landing,
      );
    case "encounters":
      return encountersPath();
    case "encounter":
      return encounterPath(route.slug);
    case "encounterEdit":
      return encounterEditPath(route.slug);
    case "lists":
      return listsPath();
    case "list":
      return withLanding(
        listPath(route.slug, route.selectedRecordKey) + (route.query ?? ""),
        route.landing,
      );
    case "listEdit":
      return listEditPath(route.slug);
    case "record":
      return (
        withLanding(
          recordPath(route.recordKey) +
            (route.selection
              ? "?selection=" + encodeURIComponent(JSON.stringify(route.selection))
              : ""),
          route.landing,
        ) + (route.section ? "#" + encodeURIComponent(route.section) : "")
      );
  }
}
function listQuery(search: string): { query?: string } {
  const params = new URLSearchParams(search);
  params.delete("landing");
  return params.size ? { query: "?" + params.toString() } : {};
}
function withSelection(path: string, selection?: RecordNavigationView) {
  if (!selection) return path;
  const url = new URL(path, window.location.origin);
  url.searchParams.set("selection", JSON.stringify(selection));
  return url.pathname + url.search;
}
function withLanding(path: string, landing?: RecordNavigationView) {
  return landing
    ? path +
        (path.includes("?") ? "&" : "?") +
        "landing=" +
        encodeURIComponent(JSON.stringify(landing))
    : path;
}

export function navigateToAtlasRoute(route: AtlasRoute) {
  const selection = "selection" in route ? route.selection : undefined;
  const current = parseAtlasRoute(window.location.pathname, window.location.search);
  const currentKey =
    "recordKey" in current
      ? current.recordKey
      : "selectedRecordKey" in current
        ? current.selectedRecordKey
        : null;
  const inherited = history.state?.atlasOwnerOrigin;
  const origin = selection?.embedded_path.length
    ? inherited?.recordKey === selection.record_key
      ? inherited
      : currentKey === selection.record_key &&
          !("selection" in current && current.selection?.embedded_path.length)
        ? {
            recordKey: selection.record_key,
            href: window.location.pathname + window.location.search,
          }
        : undefined
    : undefined;
  history.pushState(
    origin ? { atlasOwnerOrigin: origin } : null,
    "",
    atlasRoutePath(route),
  );
  window.dispatchEvent(new Event(ATLAS_ROUTE_CHANGE_EVENT));
}

export function ownerReturnPath(parent: string, landing: RecordNavigationView): string {
  const origin = history.state?.atlasOwnerOrigin;
  const fallback = atlasRoutePath({ kind: "record", recordKey: parent });
  const current = currentAtlasRoute();
  const currentKey =
    "recordKey" in current
      ? current.recordKey
      : "selectedRecordKey" in current
        ? current.selectedRecordKey
        : null;
  const currentSelection = "selection" in current ? current.selection : undefined;
  const knownParent =
    currentKey === parent && !currentSelection?.embedded_path.length
      ? window.location.pathname + window.location.search
      : fallback;
  const url = new URL(
    origin?.recordKey === parent && typeof origin.href === "string"
      ? origin.href
      : knownParent,
    window.location.origin,
  );
  if (url.origin !== window.location.origin)
    return fallback + "?landing=" + encodeURIComponent(JSON.stringify(landing));
  url.searchParams.set("landing", JSON.stringify(landing));
  return url.pathname + url.search;
}

export function navigateToAtlasPath(path: string) {
  const url = new URL(path, window.location.origin);
  if (
    url.origin !== window.location.origin ||
    parseAtlasRoute(url.pathname, url.search, url.hash).kind === "invalid"
  )
    return;
  history.pushState(null, "", url.pathname + url.search + url.hash);
  window.dispatchEvent(new Event(ATLAS_ROUTE_CHANGE_EVENT));
}

export function shouldHandleAtlasRouteClick(
  event: Pick<
    React.MouseEvent<HTMLElement>,
    "defaultPrevented" | "metaKey" | "altKey" | "ctrlKey" | "shiftKey"
  > & { button?: number; currentTarget?: EventTarget | null },
): boolean {
  const link =
    event.currentTarget instanceof Element ? event.currentTarget.closest("a") : null;
  return (
    (event.button === undefined || event.button === 0) &&
    !event.defaultPrevented &&
    !event.metaKey &&
    !event.altKey &&
    !event.ctrlKey &&
    !event.shiftKey &&
    !link?.hasAttribute("download") &&
    (!link?.getAttribute("target") || link.getAttribute("target") === "_self")
  );
}

export function searchPath(recordKey: string | null = null): string {
  return recordKey === null
    ? "/search"
    : `/search/records/${recordKeyPathSegment(recordKey)}`;
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
  return recordKey === null
    ? path
    : `${path}/records/${recordKeyPathSegment(recordKey)}`;
}

export function listEditPath(slug: string): string {
  return `/lists/${encodeURIComponent(slug)}/edit`;
}

export function recordPath(recordKey: string): string {
  return `/records/${recordKeyPathSegment(recordKey)}`;
}

function recordKeyPathSegment(recordKey: string): string {
  return encodeURIComponent(recordKey).replace(/%3A/g, ":");
}

function selectedNavigation(
  search: string,
  key = "selection",
): RecordNavigationView | undefined {
  const value = new URLSearchParams(search).get(key);
  if (!value) return undefined;
  try {
    const parsed = JSON.parse(value);
    if (typeof parsed.record_key !== "string" || !Array.isArray(parsed.embedded_path))
      throw new Error("Invalid record navigation");
    return parsed as RecordNavigationView;
  } catch {
    throw new Error("Invalid record navigation URL");
  }
}

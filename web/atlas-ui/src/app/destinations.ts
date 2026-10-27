import { linkOptions } from "@tanstack/react-router";
import type { RecordNavigationView } from "../generated/atlas";
import {
  parseSearchParams,
  type AtlasSearchParams,
  type ReadingViewInput,
} from "./urlState";
import { encodeReadingAddress } from "./readingUrl";
import {
  canonicalRouteSearch,
  type ListIndexInput,
  type EncounterIndexInput,
} from "./routeSearch";

export type RecordPageInput = ReadingViewInput & {
  recordKey: string;
  selection?: RecordNavigationView;
  landing?: RecordNavigationView;
  section?: string;
};
export type ListPageInput = ReadingViewInput & {
  slug: string;
  query?: string;
  selectedRecordKey: string | null;
  selection?: RecordNavigationView;
  landing?: RecordNavigationView;
};
export type TraitPageInput = { traitKey: string; query?: string };
export type SearchPageInput = ReadingViewInput & {
  selectedRecordKey?: string | null;
  query?: string;
  selection?: RecordNavigationView;
  landing?: RecordNavigationView;
  page?: number;
};

function readingSearch(
  input: ReadingViewInput & {
    recordKey?: string | null;
    query?: string;
    selection?: RecordNavigationView;
    landing?: RecordNavigationView;
  },
): AtlasSearchParams {
  const search = parseSearchParams(input.query ?? "");
  for (const key of [
    "selection",
    "landing",
    "rank",
    "at",
    "focus",
    "record_view",
    "record_tab",
  ])
    delete search[key];
  if (!input.recordKey) return search;
  if (input.selection) {
    const at = encodeReadingAddress(input.recordKey, input.selection, false);
    if (input.selection.spell_rank !== undefined)
      search.rank = String(input.selection.spell_rank);
    if (at !== undefined) search.at = at;
  }
  if (input.landing) {
    const focus = encodeReadingAddress(input.recordKey, input.landing, true);
    if (focus !== undefined) search.focus = focus;
  }
  if (input.view === "explore") search.record_view = "explore";
  if (input.tab && input.tab !== "incoming") search.record_tab = input.tab;
  return search;
}
export function recordDestination(input: RecordPageInput) {
  return linkOptions({
    to: "/records/$recordKey",
    params: { recordKey: input.recordKey },
    search: canonicalRouteSearch("record", readingSearch(input)),
    hash: input.section ? encodeURIComponent(input.section) : "",
  });
}
export function searchDestination(input: SearchPageInput = {}) {
  const params = readingSearch({ ...input, recordKey: input.selectedRecordKey });
  if (input.page !== undefined) params.page = String(input.page);
  const search = canonicalRouteSearch(
    input.selectedRecordKey ? "search_record" : "search",
    params,
  );
  return input.selectedRecordKey
    ? linkOptions({
        to: "/search/records/$recordKey",
        params: { recordKey: input.selectedRecordKey },
        search,
      })
    : linkOptions({ to: "/search", search });
}
export function listDestination(input: ListPageInput) {
  const search = canonicalRouteSearch(
    input.selectedRecordKey ? "list_record" : "list",
    readingSearch({ ...input, recordKey: input.selectedRecordKey }),
  );
  return input.selectedRecordKey
    ? linkOptions({
        to: "/lists/$slug/records/$recordKey",
        params: { slug: input.slug, recordKey: input.selectedRecordKey },
        search,
      })
    : linkOptions({ to: "/lists/$slug", params: { slug: input.slug }, search });
}
export function traitDestination(input: TraitPageInput) {
  return linkOptions({
    to: "/traits/$traitKey",
    params: { traitKey: input.traitKey },
    search: canonicalRouteSearch("trait", parseSearchParams(input.query ?? "")),
  });
}
export function listsDestination(input: ListIndexInput = {}) {
  return linkOptions({
    to: "/lists",
    search: canonicalRouteSearch("lists", {
      q: input.query,
      tag: input.include,
      exclude_tag: input.exclude,
    }),
  });
}
export function encountersDestination(input: EncounterIndexInput = {}) {
  return linkOptions({
    to: "/encounters",
    search: canonicalRouteSearch("encounters", { view: input.view }),
  });
}
export function listEditDestination(input: { slug: string }) {
  return linkOptions({ to: "/lists/$slug/edit", params: { slug: input.slug } });
}
export function encounterDestination(input: { slug: string }) {
  return linkOptions({ to: "/encounters/$slug", params: { slug: input.slug } });
}
export function encounterEditDestination(input: { slug: string }) {
  return linkOptions({ to: "/encounters/$slug/edit", params: { slug: input.slug } });
}

import { linkOptions } from "@tanstack/react-router";
import type { RecordNavigationView } from "../generated/atlas";
import { parseSearchParams, type AtlasSearchParams } from "./urlState";

export type RecordPageInput = {
  recordKey: string;
  selection?: RecordNavigationView;
  landing?: RecordNavigationView;
  section?: string;
};
export type ListPageInput = {
  slug: string;
  query?: string;
  selectedRecordKey: string | null;
  landing?: RecordNavigationView;
};
export type TraitPageInput = { traitKey: string; query?: string };
export type SearchPageInput = {
  selectedRecordKey?: string | null;
  query?: string;
  selection?: RecordNavigationView;
  landing?: RecordNavigationView;
};

function readingSearch(input: {
  query?: string;
  selection?: RecordNavigationView;
  landing?: RecordNavigationView;
}): AtlasSearchParams {
  return {
    ...parseSearchParams(input.query ?? ""),
    ...(input.selection ? { selection: JSON.stringify(input.selection) } : {}),
    ...(input.landing ? { landing: JSON.stringify(input.landing) } : {}),
  };
}
export function recordDestination(input: RecordPageInput) {
  return linkOptions({
    to: "/records/$recordKey",
    params: { recordKey: input.recordKey },
    search: readingSearch(input),
    hash: input.section ? encodeURIComponent(input.section) : "",
  });
}
export function searchDestination(input: SearchPageInput = {}) {
  const search = readingSearch(input);
  return input.selectedRecordKey
    ? linkOptions({
        to: "/search/records/$recordKey",
        params: { recordKey: input.selectedRecordKey },
        search,
      })
    : linkOptions({ to: "/search", search });
}
export function listDestination(input: ListPageInput) {
  const search = readingSearch(input);
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
    search: parseSearchParams(input.query ?? ""),
  });
}
export function listsDestination() {
  return linkOptions({ to: "/lists" });
}
export function encountersDestination() {
  return linkOptions({ to: "/encounters" });
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

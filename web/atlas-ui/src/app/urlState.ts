import type { RecordNavigationView } from "../generated/atlas";
import { decodeReadingAddress, ReadingUrlError } from "./readingUrl";

/** URL values stay strings; repeated values retain their original order. */
export type AtlasSearchParams = Record<string, string | string[] | undefined>;

export function parseSearchParams(search: string): AtlasSearchParams {
  const result: AtlasSearchParams = {};
  for (const [key, value] of new URLSearchParams(search)) {
    const previous = Object.prototype.hasOwnProperty.call(result, key)
      ? result[key]
      : undefined;
    Object.defineProperty(result, key, {
      value:
        previous === undefined
          ? value
          : [...(Array.isArray(previous) ? previous : [previous]), value],
      enumerable: true,
      configurable: true,
      writable: true,
    });
  }
  return result;
}

export function stringifySearchParams(search: AtlasSearchParams): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(search)) {
    if (value === undefined) continue;
    for (const entry of Array.isArray(value) ? value : [value])
      params.append(key, entry);
  }
  const query = params.toString();
  return query ? "?" + query : "";
}

export function searchParam(
  search: AtlasSearchParams,
  key: string,
): string | undefined {
  const value = search[key];
  return Array.isArray(value) ? value[0] : value;
}

export function selectedNavigation(
  search: AtlasSearchParams,
  recordKey: string,
  key: "at" | "focus" = "at",
): RecordNavigationView | undefined {
  rejectLegacyReading(search);
  const value = readingParam(search, key);
  const rank = key === "at" ? readingRank(search) : undefined;
  if (value === undefined && rank === undefined) return undefined;
  const address =
    value === undefined
      ? {
          record_key: recordKey,
          embedded_path: [],
          field: null,
          passage: null,
          source_fingerprint: null,
        }
      : decodeReadingAddress(value, recordKey, key === "focus");
  return rank === undefined ? address : { ...address, spell_rank: rank };
}

export function validateReadingSearch(search: AtlasSearchParams): AtlasSearchParams {
  selectedNavigation(search, "", "at");
  selectedNavigation(search, "", "focus");
  readingView(search);
  return search;
}

function rejectLegacyReading(search: AtlasSearchParams) {
  if (search.selection !== undefined || search.landing !== undefined)
    throw new ReadingUrlError(
      "This record link uses an unsupported reading format. Open the record again.",
    );
}

export function readingParam(
  search: AtlasSearchParams,
  key: string,
): string | undefined {
  const value = search[key];
  if (!Array.isArray(value)) return value;
  if (value.some((entry) => entry !== value[0]))
    throw new ReadingUrlError("This record link contains conflicting parameters.");
  return value[0];
}

function readingRank(search: AtlasSearchParams): number | undefined {
  const value = readingParam(search, "rank");
  if (value === undefined) return undefined;
  if (!/^\d+$/.test(value) || Number(value) > 255)
    throw new ReadingUrlError("This record link contains an invalid spell rank.");
  return Number(value);
}

export type RecordExploreTab = "incoming" | "outgoing" | "similar" | "versions";
export type ReadingViewInput = { view?: "read" | "explore"; tab?: RecordExploreTab };

export function readingView(search: AtlasSearchParams): ReadingViewInput {
  const mode = readingParam(search, "record_view");
  const tab = readingParam(search, "record_tab");
  if (mode !== undefined && mode !== "read" && mode !== "explore")
    throw new ReadingUrlError("This record link contains an invalid reading view.");
  if (
    tab !== undefined &&
    !["incoming", "outgoing", "similar", "versions"].includes(tab)
  )
    throw new ReadingUrlError("This record link contains an invalid Explore tab.");
  return {
    view: mode === "explore" ? "explore" : "read",
    tab: (tab as RecordExploreTab | undefined) ?? "incoming",
  };
}

/** Router decodes hashes with decodeURI; authored section IDs need component decoding. */
export function documentSection(href: string): string | undefined {
  const index = href.indexOf("#");
  if (index < 0 || index === href.length - 1) return undefined;
  const section = href.slice(index + 1);
  try {
    return decodeURIComponent(section);
  } catch {
    return section;
  }
}

import type { RecordNavigationView } from "../generated/atlas";

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
  key = "selection",
): RecordNavigationView | undefined {
  const value = searchParam(search, key);
  if (!value) return undefined;
  try {
    const parsed = JSON.parse(value);
    if (typeof parsed.record_key !== "string" || !Array.isArray(parsed.embedded_path))
      throw new Error();
    return parsed as RecordNavigationView;
  } catch {
    throw new Error("Invalid record navigation URL");
  }
}

export function validateReadingSearch(search: AtlasSearchParams): AtlasSearchParams {
  selectedNavigation(search);
  selectedNavigation(search, "landing");
  return search;
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

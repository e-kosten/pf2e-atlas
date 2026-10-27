import type { RecordNavigationView } from "../src/generated/atlas";
import { encodeReadingAddress } from "../src/app/readingUrl";
import { parseSearchParams, selectedNavigation } from "../src/app/urlState";

export function recordUrl(navigation: RecordNavigationView): string {
  const query = new URLSearchParams();
  if (navigation.spell_rank !== undefined)
    query.set("rank", String(navigation.spell_rank));
  const at = encodeReadingAddress(navigation.record_key, navigation, false);
  if (at !== undefined) query.set("at", at);
  const suffix = query.toString();
  return `/records/${encodeURIComponent(navigation.record_key).replace(/%3A/g, ":")}${suffix ? `?${suffix}` : ""}`;
}

export function readUrl(url: URL, key: "at" | "focus" = "at") {
  const recordKey = decodeURIComponent(url.pathname.split("/records/")[1]);
  return (
    selectedNavigation(parseSearchParams(url.search), recordKey, key) ?? {
      record_key: recordKey,
      embedded_path: [],
      field: null,
      passage: null,
      source_fingerprint: null,
    }
  );
}

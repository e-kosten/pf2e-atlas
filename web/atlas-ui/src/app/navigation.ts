import type React from "react";
import { atlasRouter } from "./router";
import * as destinations from "./destinations";
import { recordDestination } from "./destinations";
import {
  selectedNavigation,
  parseSearchParams,
  validateReadingSearch,
} from "./urlState";
import { encodeReadingAddress } from "./readingUrl";
import type { RecordNavigationView } from "../generated/atlas";

export type Destination = ReturnType<(typeof destinations)[keyof typeof destinations]>;
export function destinationHref(destination: Destination) {
  return atlasRouter.buildLocation(destination).href;
}

/** Full-page embedded reading remembers its checked parent context. */
export function navigateDestination(destination: Destination) {
  const next = atlasRouter.buildLocation(destination);
  const nextRecordKey = atlasRouter
    .matchRoutes(next.pathname, next.search)
    .flatMap((match) =>
      "recordKey" in match.params ? [match.params.recordKey] : [],
    )[0];
  const selection = nextRecordKey
    ? selectedNavigation(next.search, nextRecordKey)
    : undefined;
  const current = atlasRouter.latestLocation;
  const currentRecordKey = atlasRouter.state.matches.flatMap((match) =>
    "recordKey" in match.params ? [match.params.recordKey] : [],
  )[0];
  const selected = currentRecordKey
    ? selectedNavigation(current.search, currentRecordKey)
    : undefined;
  const inherited = current.state.atlasOwnerOrigin;
  const origin = selection?.embedded_path.length
    ? inherited?.recordKey === selection.record_key
      ? inherited
      : currentRecordKey === selection.record_key && !selected?.embedded_path.length
        ? { recordKey: selection.record_key, href: current.href }
        : undefined
    : undefined;
  return atlasRouter.navigate({ ...destination, state: { atlasOwnerOrigin: origin } });
}

export function ownerReturnHref(parent: string, landing: RecordNavigationView): string {
  const current = atlasRouter.latestLocation;
  const fallback = destinationHref(recordDestination({ recordKey: parent, landing }));
  const origin = current.state.atlasOwnerOrigin;
  const currentRecordKey = atlasRouter.state.matches.flatMap((match) =>
    "recordKey" in match.params ? [match.params.recordKey] : [],
  )[0];
  const selected = currentRecordKey
    ? selectedNavigation(current.search, currentRecordKey)
    : undefined;
  const knownParent =
    currentRecordKey === parent && !selected?.embedded_path.length
      ? current.href
      : fallback;
  const url = new URL(
    origin?.recordKey === parent && typeof origin.href === "string"
      ? origin.href
      : knownParent,
    window.location.origin,
  );
  if (url.origin !== window.location.origin) return fallback;
  const focus = encodeReadingAddress(parent, landing, true);
  if (focus === undefined) url.searchParams.delete("focus");
  else url.searchParams.set("focus", focus);
  return url.pathname + url.search;
}

export function navigatePath(path: string) {
  const url = new URL(path, window.location.origin);
  if (url.origin !== window.location.origin) return;
  // Raw native destinations have already been projected by the shared reader.
  validateReadingSearch(parseSearchParams(url.search));
  return atlasRouter.navigate({ href: url.pathname + url.search + url.hash });
}

/** Local reading actions leave modified/native link activations to the browser. */
export function isPrimaryActivation(
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

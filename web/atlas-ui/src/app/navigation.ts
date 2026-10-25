import type React from "react";
import { atlasRouter } from "./router";
import * as destinations from "./destinations";
import { recordDestination } from "./destinations";
import { selectedNavigation, parseSearchParams } from "./urlState";
import type { RecordNavigationView } from "../generated/atlas";

export type Destination = ReturnType<(typeof destinations)[keyof typeof destinations]>;
export function destinationHref(destination: Destination) {
  return atlasRouter.buildLocation(destination).href;
}

/** Full-page embedded reading remembers its checked parent context. */
export function navigateDestination(destination: Destination) {
  const next = atlasRouter.buildLocation(destination);
  const selection = selectedNavigation(next.search);
  const current = atlasRouter.latestLocation;
  const selected = selectedNavigation(current.search);
  const currentRecordKey = atlasRouter.state.matches.flatMap((match) =>
    "recordKey" in match.params ? [match.params.recordKey] : [],
  )[0];
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
  const selected = selectedNavigation(current.search);
  const knownParent =
    currentRecordKey === parent && !selected?.embedded_path.length
      ? current.href
      : fallback;
  const url = new URL(
    origin?.recordKey === parent ? origin.href : knownParent,
    window.location.origin,
  );
  if (url.origin !== window.location.origin) return fallback;
  url.searchParams.set("landing", JSON.stringify(landing));
  return url.pathname + url.search;
}

export function navigatePath(path: string) {
  const url = new URL(path, window.location.origin);
  if (url.origin !== window.location.origin) return;
  // Raw native destinations have already been projected by the shared reader.
  selectedNavigation(parseSearchParams(url.search));
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

import type {
  EmbeddedActivityAddress,
  RecordNavigationView,
} from "../../generated/atlas";

// Activity anchors address the entire checked node; field/passage selections are
// retained by detail navigation and are distinct from parent reading landing.
export function activityAddress(
  navigation: RecordNavigationView,
): RecordNavigationView {
  return {
    record_key: navigation.record_key,
    embedded_path: navigation.embedded_path.map((owner) => ({
      collection: owner.collection,
      identity: owner.identity,
    })),
    field: null,
    passage: null,
    source_fingerprint: navigation.source_fingerprint,
  };
}
export function activityAnchor(navigation: RecordNavigationView): string {
  return `embedded-${encodeURIComponent(JSON.stringify(activityAddress(navigation)))}`;
}

export function landOnActivity(
  container: HTMLElement | null,
  address: RecordNavigationView,
  onLandingReady?: (element?: HTMLElement) => void,
) {
  const anchor = activityAnchor(address);
  const element = Array.from(
    container?.querySelectorAll<HTMLElement>("[data-atlas-activity-anchor]") ?? [],
  ).find((candidate) => candidate.dataset.atlasActivityAnchor === anchor);
  if (!element) return;
  if (!onLandingReady) element.scrollIntoView?.({ block: "start" });
  (element.querySelector<HTMLElement>("button,a[href]") ?? element).focus({
    preventScroll: true,
  });
  onLandingReady?.(element);
}

export function immediateActivityAddress(
  navigation: RecordNavigationView,
): EmbeddedActivityAddress | null {
  const [owner, ...descendants] = navigation.embedded_path;
  if (!owner || descendants.length) return null;
  return {
    record_key: navigation.record_key,
    owner,
    source_fingerprint: navigation.source_fingerprint,
  };
}

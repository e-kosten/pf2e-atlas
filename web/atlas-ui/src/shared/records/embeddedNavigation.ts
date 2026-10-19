import type { RecordNavigationView } from "../../generated/atlas";

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

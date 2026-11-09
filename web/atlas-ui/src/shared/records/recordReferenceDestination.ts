import type { RecordReferenceDestinationView } from "../../generated/atlas";

export function referenceRecordTarget(destination?: RecordReferenceDestinationView) {
  return destination?.kind === "resolved_record" ||
    destination?.kind === "resolved_embedded"
    ? destination.target
    : undefined;
}

export function referenceExternalUrl(destination?: RecordReferenceDestinationView) {
  return destination?.kind === "resolved_url" || destination?.kind === "unverified_url"
    ? destination.url
    : undefined;
}

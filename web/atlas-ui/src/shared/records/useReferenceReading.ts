import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { getRecordDetail } from "../../api/atlasApi";
import type { RecordDetailView, RecordNavigationView } from "../../generated/atlas";
import { activityAnchor } from "./ownedNavigation";

export function rootNavigation(recordKey: string): RecordNavigationView {
  return {
    record_key: recordKey,
    owners: [],
    field: null,
    passage: null,
    source_fingerprint: null,
  };
}

export function parentShowsActivity(
  parent: RecordDetailView,
  target: RecordNavigationView,
): boolean {
  if (target.field || target.passage || target.record_key !== parent.record.record_key)
    return false;
  const body = parent.presentation.body;
  const actor =
    body.kind === "creature"
      ? body.value
      : body.kind === "hazard"
        ? body.value.actor
        : null;
  return (
    actor?.activities.value?.some(
      (activity) =>
        (activity.kind === "ability" ||
          activity.kind === "strike" ||
          activity.kind === "casting_entry") &&
        activityAnchor(activity.navigation) === activityAnchor(target),
    ) ?? false
  );
}

export type ReadingRequest = {
  navigation: RecordNavigationView;
  match?: RecordNavigationView;
  landing?: RecordNavigationView;
};

// A match first checks the typed parent's visible activities. Other owned nodes
// and precise field/passage addresses retain their complete addressed reader.
export function useReferenceReading(
  initial: ReadingRequest | null = null,
  enabled = true,
  epoch = 0,
) {
  const [request, inspect] = useState(initial);
  const query = useQuery({
    queryKey: ["record-detail", request, epoch],
    enabled: enabled && request !== null,
    queryFn: async ({ signal }) => {
      const navigation = request!.navigation;
      const owned =
        navigation.owners.length ||
        navigation.field ||
        navigation.passage ||
        navigation.source_fingerprint;
      const parent = await getRecordDetail(
        navigation.record_key,
        owned
          ? { ...navigation, fields: navigation.field ? [navigation.field] : [] }
          : undefined,
        signal,
      );
      signal.throwIfAborted();
      const match = request!.match;
      if (match && !parentShowsActivity(parent, match)) {
        const detail = await getRecordDetail(
          match.record_key,
          { ...match, fields: match.field ? [match.field] : [] },
          signal,
        );
        return { detail, landing: undefined };
      }
      return { detail: parent, landing: match ?? request!.landing };
    },
  });
  return {
    request,
    inspect,
    detail: query.data?.detail,
    landing: query.data?.landing,
    loading: query.isFetching,
    error: query.error,
    retry: () => {
      void query.refetch();
    },
  };
}
export type ReferenceReading = ReturnType<typeof useReferenceReading>;

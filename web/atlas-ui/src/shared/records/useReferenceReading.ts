import { recordDetailRequest } from "./recordDetailQuery";
import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
import { queryOptions, useQuery } from "@tanstack/react-query";
import { useCallback, useReducer } from "react";
import { AtlasApiError, getRecordDetail } from "../../api/atlasApi";
import type { RecordDetailView, RecordNavigationView } from "../../generated/atlas";
import { activityAnchor } from "./embeddedNavigation";

export function rootNavigation(recordKey: string): RecordNavigationView {
  return {
    record_key: recordKey,
    embedded_path: [],
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

/** Shared exact query for visible reading and preparing a picker selection. */
export function referenceReadingQueryOptions(
  request: ReadingRequest | null,
  epoch = 0,
  generation?: string | null,
) {
  function checked(detail: RecordDetailView) {
    if (generation && detail.artifact_generation !== generation)
      throw new AtlasApiError(409, "The reference collection changed.", {
        code: "artifact_generation_changed",
        message: "The reference collection changed.",
        recoverable_action: "refresh_results",
      });
    return detail;
  }
  return queryOptions({
    queryKey: [
      "record-detail",
      request,
      epoch,
      ...(generation === undefined ? [] : [generation]),
    ],
    enabled: request !== null,
    queryFn: async ({ signal }) => {
      const navigation = request!.navigation;
      const embedded =
        navigation.embedded_path.length ||
        navigation.field ||
        navigation.passage ||
        navigation.spell_rank ||
        navigation.source_fingerprint;
      const parent = await getRecordDetail(
        navigation.record_key,
        embedded ? recordDetailRequest(navigation) : undefined,
        signal,
      );
      signal.throwIfAborted();
      checked(parent);
      const match = request!.match;
      if (match && !parentShowsActivity(parent, match)) {
        const detail = await getRecordDetail(
          match.record_key,
          recordDetailRequest(match),
          signal,
        );
        return { detail: checked(detail), parent, landing: undefined };
      }
      return { detail: parent, parent, landing: match ?? request!.landing };
    },
  });
}

// A match first checks the typed parent's visible activities. Other embedded nodes
// and precise field/passage addresses retain their complete addressed reader.
export function useReferenceReading(
  initial: ReadingRequest | null = null,
  enabled = true,
  epoch = 0,
  generation?: string | null,
) {
  const [session, dispatch] = useReducer(
    (
      state: {
        request: ReadingRequest | null;
        history: ReadingRequest[];
        restoringHistory: boolean;
      },
      action:
        | { kind: "select" | "inspect" | "rank"; request: ReadingRequest | null }
        | { kind: "back" },
    ) => {
      if (action.kind === "back")
        return state.history.length
          ? {
              request: state.history[state.history.length - 1]!,
              history: state.history.slice(0, -1),
              restoringHistory: true,
            }
          : state;
      const same = JSON.stringify(state.request) === JSON.stringify(action.request);
      return {
        request: action.request,
        restoringHistory: false,
        history:
          action.kind === "select" || !action.request
            ? []
            : same || action.kind === "rank"
              ? state.history
              : [...state.history, ...(state.request ? [state.request] : [])].slice(
                  -24,
                ),
      };
    },
    { request: initial, history: [], restoringHistory: false },
  );
  const request = session.request;
  const inspect = useCallback(
    (request: ReadingRequest | null) => dispatch({ kind: "inspect", request }),
    [],
  );
  const select = useCallback(
    (request: ReadingRequest | null) => dispatch({ kind: "select", request }),
    [],
  );
  const query = useQuery({
    ...referenceReadingQueryOptions(request, epoch, generation),
    enabled: enabled && request !== null,
  });
  const retained = useRetainedRecord(
    JSON.stringify([
      recordReadingIdentity(
        (request?.match ?? request?.navigation)?.record_key ?? null,
        request?.match ?? request?.navigation,
      ),
      request?.landing,
      epoch,
      generation,
    ]),
    query.data,
    query.error,
  );
  return {
    changeRank: (navigation: RecordNavigationView) =>
      dispatch({ kind: "rank", request: { navigation, landing: request?.landing } }),
    request,
    restoringHistory: session.restoringHistory,
    inspect,
    select,
    canGoBack: session.history.length > 0,
    back: () => dispatch({ kind: "back" }),
    detail: retained?.detail,
    parentDetail: retained?.parent,
    landing: retained?.landing,
    loading: query.isFetching,
    error: query.error,
    retry: () => {
      void query.refetch();
    },
  };
}
export type ReferenceReading = ReturnType<typeof useReferenceReading>;

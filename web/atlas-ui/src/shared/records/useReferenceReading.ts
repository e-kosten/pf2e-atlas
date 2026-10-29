import { useReferenceQueryRetirement } from "../queries/useReferenceQueryRetirement";
import { recordDetailQueryOptions } from "./recordDetailQuery";
import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
import { useQuery } from "@tanstack/react-query";
import { useCallback, useMemo, useReducer, useState } from "react";
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

export function referenceReadingFallback(
  request: ReadingRequest | null,
  parent: RecordDetailView | undefined,
): RecordNavigationView | undefined {
  return parent && request?.match && !parentShowsActivity(parent, request.match)
    ? request.match
    : undefined;
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
  const parentOptions = recordDetailQueryOptions(
    request?.navigation.record_key ?? null,
    request?.navigation,
    generation,
  );
  const parentQuery = useQuery({
    ...parentOptions,
    enabled: enabled && request !== null,
  });
  const fallback = referenceReadingFallback(request, parentQuery.data);
  const exactOptions = recordDetailQueryOptions(
    fallback?.record_key ?? null,
    fallback,
    generation ?? parentQuery.data?.artifact_generation,
  );
  const exactQuery = useQuery({
    ...exactOptions,
    enabled: enabled && fallback !== undefined && !parentQuery.error,
  });
  const parentRetirement = useReferenceQueryRetirement(parentOptions.queryKey);
  const exactRetirement = useReferenceQueryRetirement(exactOptions.queryKey);
  const error = parentQuery.error ?? (fallback ? exactQuery.error : null);
  const detail = fallback ? exactQuery.data : parentQuery.data;
  const data = useMemo(
    () =>
      detail && parentQuery.data ? { detail, parent: parentQuery.data } : undefined,
    [detail, parentQuery.data],
  );
  const identity = recordReadingIdentity(
    (request?.match ?? request?.navigation)?.record_key ?? null,
    request?.match ?? request?.navigation,
  );
  const [basis, setBasis] = useState({ identity, generation });
  const readingGeneration =
    generation ??
    parentQuery.data?.artifact_generation ??
    (basis.identity === identity ? basis.generation : undefined);
  if (basis.identity !== identity || basis.generation !== readingGeneration)
    setBasis({ identity, generation: readingGeneration });
  const retained = useRetainedRecord(
    JSON.stringify([
      identity,
      request?.landing,
      epoch,
      readingGeneration,
      parentRetirement,
      exactRetirement,
    ]),
    data,
    error,
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
    landing: fallback ? undefined : (request?.match ?? request?.landing),
    loading:
      parentQuery.isFetching || (fallback !== undefined && exactQuery.isFetching),
    error,
    retry: () => {
      if (fallback && !parentQuery.error) void exactQuery.refetch();
      else void parentQuery.refetch();
    },
  };
}
export type ReferenceReading = ReturnType<typeof useReferenceReading>;

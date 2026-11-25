import { isArtifactGenerationChanged } from "../queries/referenceQueries";
import { useReferenceQueryRetirement } from "../queries/useReferenceQueryRetirement";
import { recordDetailQueryOptions } from "./recordDetailQuery";
import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
import { useQuery, useQueryClient } from "@tanstack/react-query";
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
  fields?: string[];
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
  ownerGeneration?: string | null,
  sectionRenderingRecord?: string,
) {
  const [session, dispatch] = useReducer(
    (
      state: {
        request: ReadingRequest | null;
        history: ReadingRequest[];
        basis?: { generation: string; epoch: number };
        restoringHistory: boolean;
      },
      action:
        | {
            kind: "select" | "inspect" | "rank";
            request: ReadingRequest | null;
            basis?: { generation: string; epoch: number };
          }
        | { kind: "back" }
        | { kind: "reload" },
    ) => {
      if (action.kind === "reload") return { ...state, basis: undefined };
      if (action.kind === "back")
        return state.history.length
          ? {
              request: state.history[state.history.length - 1]!,
              basis: state.basis,
              history: state.history.slice(0, -1),
              restoringHistory: true,
            }
          : state;
      const same = JSON.stringify(state.request) === JSON.stringify(action.request);
      return {
        request: action.request,
        basis: action.kind === "select" ? action.basis : (action.basis ?? state.basis),
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
  const generation =
    ownerGeneration ??
    (session.basis?.epoch === epoch ? session.basis.generation : undefined);
  const inspect = useCallback(
    (request: ReadingRequest | null, generation?: string | null) =>
      dispatch({
        kind: "inspect",
        request,
        basis: generation ? { generation, epoch } : undefined,
      }),
    [epoch],
  );
  const select = useCallback(
    (request: ReadingRequest | null, generation?: string | null) =>
      dispatch({
        kind: "select",
        request,
        basis: generation ? { generation, epoch } : undefined,
      }),
    [epoch],
  );
  const renderSections =
    sectionRenderingRecord !== undefined &&
    sectionRenderingRecord === request?.navigation.record_key;
  const parentOptions = recordDetailQueryOptions(
    request?.navigation.record_key ?? null,
    request?.navigation,
    generation,
    request?.fields,
    renderSections,
  );
  const client = useQueryClient();
  // A cold link may already have opened this exact document. Preserve its
  // original freshness when a result from the same collection selects it.
  const currentOptions =
    generation && request
      ? recordDetailQueryOptions(
          request.navigation.record_key,
          request.navigation,
          undefined,
          request.fields,
          renderSections,
        )
      : undefined;
  const latest = currentOptions
    ? client.getQueryState<RecordDetailView>(currentOptions.queryKey)
    : undefined;
  const matching =
    latest?.status === "success" &&
    !latest.isInvalidated &&
    latest.data?.artifact_generation === generation
      ? latest
      : undefined;
  const parentQuery = useQuery({
    ...parentOptions,
    initialData: matching?.data,
    initialDataUpdatedAt: matching?.dataUpdatedAt,
    enabled: enabled && request !== null,
  });
  const fallback = referenceReadingFallback(request, parentQuery.data);
  const exactOptions = recordDetailQueryOptions(
    fallback?.record_key ?? null,
    fallback,
    generation ?? parentQuery.data?.artifact_generation,
    undefined,
    sectionRenderingRecord !== undefined &&
      sectionRenderingRecord === fallback?.record_key,
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
      request?.fields,
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
      dispatch({
        kind: "rank",
        basis: detail ? { generation: detail.artifact_generation, epoch } : undefined,
        request: {
          navigation,
          landing: request?.landing,
          fields: detail?.selected_fields ?? request?.fields,
        },
      }),
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
      if (!ownerGeneration && request && isArtifactGenerationChanged(error)) {
        // The document itself supplied the generation. Reopen its current
        // parent before resolving any exact field, passage, or rank again.
        client.removeQueries({ queryKey: parentOptions.queryKey, exact: true });
        if (currentOptions)
          client.removeQueries({ queryKey: currentOptions.queryKey, exact: true });
        if (fallback)
          client.removeQueries({ queryKey: exactOptions.queryKey, exact: true });
        dispatch({ kind: "reload" });
      } else if (fallback && !parentQuery.error) void exactQuery.refetch();
      else void parentQuery.refetch();
    },
  };
}
export type ReferenceReading = ReturnType<typeof useReferenceReading>;

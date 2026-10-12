import { useCallback, useRef, useState } from "react";
import type { RecordPreviewAnchor } from "./recordPreviewTypes";
import type {
  RecordNavigationView,
  RecordPresentationView,
} from "../../generated/atlas";
import { useRecordDetail } from "./useRecordDetail";
import { useQuery } from "@tanstack/react-query";
import { getEncounterReference } from "../../api/atlasApi";

type EncounterPreviewContext = {
  encounterRef: string;
  identity: string;
  generation: string | null;
  parentRecordKey: string | undefined;
  presentation: RecordPresentationView | undefined;
};

export function useRecordPreview(encounterContext?: EncounterPreviewContext) {
  const [recordKey, setRecordKey] = useState<string | null>(null);
  const [anchor, setAnchor] = useState<RecordPreviewAnchor | null>(null);
  const [selection, setSelection] = useState<RecordNavigationView>();
  const opener = useRef<HTMLElement | null>(null);
  const [requestedContext, setRequestedContext] = useState<{
    identity: string;
    generation: string | null;
  }>();
  const contextualError = requestedContext
    ? requestedContext.identity !== encounterContext?.identity
      ? "The selected participant has changed. Reopen the activity for this participant."
      : !requestedContext.generation
        ? "Reference content is unavailable for this participant."
        : requestedContext.generation !== encounterContext?.generation
          ? "The reference collection has changed. Reload the encounter and reopen the activity."
          : !encounterContext?.presentation
            ? "This participant's reference is no longer available."
            : undefined
    : undefined;
  // Runtime views never seed immutable reference caches. A changed participant
  // presentation starts a new request; composition stays in Rust app-service.
  const contextual = useQuery({
    queryKey: [
      "encounter-record-detail",
      encounterContext?.encounterRef,
      requestedContext?.identity,
      requestedContext?.generation,
      recordKey,
      JSON.stringify(encounterContext?.presentation, (_key, value) =>
        typeof value === "bigint" ? value.toString() : value,
      ),
      selection,
    ],
    enabled: Boolean(requestedContext && !contextualError && selection),
    queryFn: ({ signal }) =>
      getEncounterReference(
        encounterContext!.encounterRef,
        requestedContext!.identity,
        {
          artifact_generation: requestedContext!.generation!,
          selection: {
            record_key: recordKey!,
            owners: selection!.owners,
            fields: selection!.field ? [selection!.field] : [],
            passage: selection!.passage,
            source_fingerprint: selection!.source_fingerprint,
          },
        },
        signal,
      ),
    staleTime: 0,
    gcTime: 0,
    retry: false,
  });
  const detail = useRecordDetail(requestedContext ? null : recordKey, selection);

  const close = useCallback(() => {
    if (opener.current?.isConnected) opener.current.focus();
    opener.current = null;
    setRecordKey(null);
    setAnchor(null);
    setSelection(undefined);
    setRequestedContext(undefined);
  }, []);

  const open = useCallback(
    (
      nextRecordKey: string,
      anchor?: HTMLElement,
      navigation?: RecordNavigationView,
    ) => {
      opener.current ??=
        anchor ??
        (document.activeElement instanceof HTMLElement ? document.activeElement : null);
      setRecordKey(nextRecordKey);
      setAnchor((current) => current ?? anchor ?? null);
      setSelection(navigation);
      const contextual =
        encounterContext?.parentRecordKey === nextRecordKey && navigation;
      setRequestedContext(
        contextual
          ? {
              identity: encounterContext.identity,
              generation: encounterContext.generation,
            }
          : undefined,
      );
    },
    [encounterContext],
  );

  return {
    anchor,
    close,
    detail: requestedContext
      ? contextualError
        ? undefined
        : contextual.data
      : detail.data,
    error: requestedContext
      ? contextualError
        ? new Error(contextualError)
        : contextual.error
      : detail.error,
    retry: contextualError
      ? undefined
      : () => {
          if (requestedContext) void contextual.refetch();
          else void detail.refetch();
        },
    loading: requestedContext
      ? !contextualError && (contextual.isLoading || contextual.isFetching)
      : detail.isLoading || detail.isFetching,
    open,
    recordKey,
    selection,
  };
}

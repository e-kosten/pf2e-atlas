import { useCallback, useRef, useState } from "react";
import type { RecordPreviewAnchor } from "./recordPreviewTypes";
import type {
  RecordNavigationView,
  RecordPresentationView,
} from "../../generated/atlas";
import { useRecordDetail } from "./useRecordDetail";
import { useQuery } from "@tanstack/react-query";
import { getEncounterReference } from "../../api/atlasApi";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";

type EncounterPreviewContext = {
  encounterRef: string;
  identity: string;
  generation: string | null;
  parentRecordKey: string | undefined;
  presentation: RecordPresentationView | undefined;
  reloadEncounter?: () => Promise<unknown>;
};

export function useRecordPreview(encounterContext?: EncounterPreviewContext) {
  const [recordKey, setRecordKey] = useState<string | null>(null);
  const [anchor, setAnchor] = useState<RecordPreviewAnchor | null>(null);
  const [selection, setSelection] = useState<RecordNavigationView>();
  const opener = useRef<HTMLElement | null>(null);
  const session = useRef(0);
  const [reloadState, setReloadState] = useState<
    { kind: "loading" } | { kind: "finished" } | { kind: "failed"; error: Error } | null
  >(null);
  const [requestedContext, setRequestedContext] = useState<{
    identity: string;
    generation: string | null;
  }>();
  const contextualError = requestedContext
    ? reloadState?.kind === "finished"
      ? "The encounter has refreshed. Close this preview and reopen the activity to use its current context."
      : requestedContext.identity !== encounterContext?.identity
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
    enabled: Boolean(requestedContext && !contextualError && !reloadState && selection),
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

  const reloadOwner = async () => {
    if (!requestedContext) {
      await detail.refetch();
      return;
    }
    if (!encounterContext?.reloadEncounter || reloadState?.kind === "loading") return;
    const currentSession = session.current;
    setReloadState({ kind: "loading" });
    try {
      await encounterContext.reloadEncounter();
      if (session.current === currentSession) setReloadState({ kind: "finished" });
    } catch (error) {
      if (session.current === currentSession)
        setReloadState({
          kind: "failed",
          error: error instanceof Error ? error : new Error("Encounter refresh failed"),
        });
    }
  };

  const close = useCallback(() => {
    if (opener.current?.isConnected) opener.current.focus();
    opener.current = null;
    session.current += 1;
    setReloadState(null);
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
      session.current += 1;
      setReloadState(null);
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
      ? contextualError || reloadState
        ? undefined
        : contextual.data
      : detail.data,
    error: requestedContext
      ? reloadState?.kind === "loading"
        ? null
        : reloadState?.kind === "failed"
          ? reloadState.error
          : contextualError
            ? new Error(contextualError)
            : contextual.error
      : detail.error,
    retry:
      reloadState?.kind === "failed" && encounterContext?.reloadEncounter
        ? () => {
            void reloadOwner();
          }
        : contextualError || reloadState
          ? undefined
          : requestedContext && isArtifactGenerationChanged(contextual.error)
            ? encounterContext?.reloadEncounter
              ? () => {
                  void reloadOwner();
                }
              : undefined
            : () => {
                if (requestedContext) void contextual.refetch();
                else void detail.refetch();
              },
    loading: requestedContext
      ? reloadState?.kind === "loading" ||
        (!reloadState &&
          !contextualError &&
          (contextual.isLoading || contextual.isFetching))
      : detail.isLoading || detail.isFetching,
    reload:
      requestedContext && !encounterContext?.reloadEncounter
        ? undefined
        : () => {
            void reloadOwner();
          },
    reloadLabel: requestedContext ? "Reload encounter" : "Reload record",
    retryLabel:
      requestedContext &&
      (reloadState?.kind === "failed" || isArtifactGenerationChanged(contextual.error))
        ? "Reload encounter"
        : "Retry",
    open,
    recordKey,
    selection,
  };
}

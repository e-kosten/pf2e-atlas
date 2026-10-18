import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
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
  const [landing, setLanding] = useState<RecordNavigationView>();
  type PriorDocument = {
    recordKey: string;
    selection?: RecordNavigationView;
    landing?: RecordNavigationView;
    context?: { identity: string; generation: string | null };
    scroll: number;
    focusHref?: string;
  };
  const [history, setHistory] = useState<PriorDocument[]>([]);
  const [restore, setRestore] = useState({
    scroll: 0,
    focusHref: undefined as string | undefined,
  });
  const [restoringHistory, setRestoringHistory] = useState(false);
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
            spell_rank: selection!.spell_rank,
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
  const retainedContextual = useRetainedRecord(
    JSON.stringify(
      [
        encounterContext?.encounterRef,
        requestedContext,
        encounterContext?.presentation,
        recordReadingIdentity(recordKey, selection),
      ],
      (_key, value) => (typeof value === "bigint" ? value.toString() : value),
    ),
    contextual.data,
    contextual.error,
  );
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
    setLanding(undefined);
    setHistory([]);
    setRestoringHistory(false);
  }, []);

  const open = useCallback(
    (
      nextRecordKey: string,
      anchor?: HTMLElement,
      navigation?: RecordNavigationView,
      nextLanding?: RecordNavigationView,
    ) => {
      if (
        recordKey &&
        (recordKey !== nextRecordKey ||
          JSON.stringify(selection) !== JSON.stringify(navigation) ||
          JSON.stringify(landing) !== JSON.stringify(nextLanding))
      ) {
        const scrollHost = anchor?.closest<HTMLElement>(
          ".record-preview-popover__body,.record-preview-drawer__body",
        );
        setHistory((prior) =>
          [
            ...prior,
            {
              recordKey,
              selection,
              landing,
              context: requestedContext,
              scroll: scrollHost?.scrollTop ?? 0,
              focusHref: anchor?.closest("a")?.getAttribute("href") ?? undefined,
            },
          ].slice(-24),
        );
      }
      opener.current ??=
        anchor ??
        (document.activeElement instanceof HTMLElement ? document.activeElement : null);
      session.current += 1;
      setReloadState(null);
      setRecordKey(nextRecordKey);
      setAnchor((current) => current ?? anchor ?? null);
      setSelection(navigation);
      setLanding(nextLanding);
      setRestore({ scroll: 0, focusHref: undefined });
      setRestoringHistory(false);
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
    [encounterContext, recordKey, selection, landing, requestedContext],
  );

  return {
    anchor,
    close,
    detail: requestedContext
      ? contextualError || reloadState
        ? undefined
        : retainedContextual
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
    changeRank: (spell_rank: number | undefined) => {
      const current =
        selection ??
        (requestedContext ? contextual.data?.selected : detail.data?.selected);
      if (current) setSelection({ ...current, spell_rank });
    },
    recordKey,
    selection,
    landing,
    readingHistory: {
      address: JSON.stringify([recordReadingIdentity(recordKey, selection), landing]),
      restoringHistory,
      restore,
      canGoBack: history.length > 0,
      onBack: () => {
        const prior = history[history.length - 1];
        if (!prior) return;
        session.current += 1;
        setReloadState(null);
        setHistory(history.slice(0, -1));
        setRecordKey(prior.recordKey);
        setSelection(prior.selection);
        setLanding(prior.landing);
        setRequestedContext(prior.context);
        setRestore({ scroll: prior.scroll, focusHref: prior.focusHref });
        setRestoringHistory(true);
      },
    },
  };
}

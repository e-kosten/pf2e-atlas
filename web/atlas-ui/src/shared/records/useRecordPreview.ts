import { RecordContextError } from "./recordContextError";
import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
import { useCallback, useLayoutEffect, useRef, useState } from "react";
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
  revision?: bigint;
  conditions?: { condition_id: bigint; condition_key?: string | null }[];
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
    context?: { identity: string; generation: string | null; conditionId?: bigint };
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
  // Openers include prepared HTML links outside React's prop ownership. Keep
  // their expanded state with the preview owner, including nested navigation.
  useLayoutEffect(() => {
    const element = opener.current;
    if (!recordKey || !element) return;
    const previous = element.getAttribute("aria-expanded");
    element.setAttribute("aria-expanded", "true");
    return () => {
      if (previous === null) element.removeAttribute("aria-expanded");
      else element.setAttribute("aria-expanded", previous);
    };
  }, [recordKey]);
  const session = useRef(0);
  const [reloadState, setReloadState] = useState<
    { kind: "loading" } | { kind: "finished" } | { kind: "failed"; error: Error } | null
  >(null);
  const [requestedContext, setRequestedContext] = useState<{
    identity: string;
    generation: string | null;
    conditionId?: bigint;
  }>();
  const contextualError = requestedContext
    ? reloadState?.kind === "finished"
      ? new RecordContextError("refreshed")
      : requestedContext.identity !== encounterContext?.identity
        ? new RecordContextError("participantChanged")
        : requestedContext.conditionId !== undefined &&
            encounterContext?.conditions &&
            !encounterContext.conditions.some(
              (c) => c.condition_id === requestedContext.conditionId,
            )
          ? new RecordContextError("conditionRemoved")
          : !requestedContext.generation
            ? new RecordContextError("referenceUnavailable")
            : requestedContext.generation !== encounterContext?.generation
              ? new RecordContextError("generationChanged")
              : !encounterContext?.presentation &&
                  requestedContext.conditionId === undefined
                ? new RecordContextError("participantReferenceRemoved")
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
      requestedContext?.conditionId?.toString(),
      encounterContext?.revision?.toString(),
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
          condition_id: requestedContext!.conditionId,
          selection: {
            record_key: recordKey!,
            spell_rank: selection!.spell_rank,
            embedded_path: selection!.embedded_path,
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
  // Runtime refresh is a new request, not a new source document. Retain the
  // addressed content while Rust prepares fresh context; identity/generation and
  // exact navigation still prevent borrowing content across sources.
  const retainedContextual = useRetainedRecord(
    JSON.stringify(
      [
        encounterContext?.encounterRef,
        requestedContext,
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
    if (opener.current?.isConnected) opener.current.focus({ preventScroll: true });
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
        const scrollHost = anchor?.closest<HTMLElement>(".record-preview-drawer__body");
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
      const condition = encounterContext?.conditions?.find(
        (condition) =>
          condition.condition_key === nextRecordKey &&
          condition.condition_id.toString() === anchor?.dataset.atlasConditionId,
      );
      const addressed =
        navigation ??
        (condition
          ? {
              record_key: nextRecordKey,
              embedded_path: [],
              field: null,
              passage: null,
              source_fingerprint: null,
            }
          : undefined);
      setSelection(addressed);
      setLanding(nextLanding);
      setRestore({ scroll: 0, focusHref: undefined });
      setRestoringHistory(false);
      const contextual =
        (encounterContext?.parentRecordKey === nextRecordKey && navigation) ||
        condition;
      setRequestedContext(
        contextual
          ? {
              identity: encounterContext!.identity,
              generation: encounterContext!.generation,
              conditionId: condition?.condition_id,
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
            ? contextualError
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

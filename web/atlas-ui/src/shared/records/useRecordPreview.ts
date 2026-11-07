import { useDismissOnReadingNavigation } from "../ui/overlays/useDismissOnReadingNavigation";
import { referenceQueryMeta } from "../queries/referenceQueries";
import { RecordContextError } from "./recordContextError";
import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
import { useCallback, useLayoutEffect, useRef, useState } from "react";
import type { RecordPreviewAnchor } from "./recordPreviewTypes";
import type {
  RecordNavigationView,
  ReferenceDestinationView,
  ReferenceHelpView,
  RecordPresentationView,
} from "../../generated/atlas";
import { useRecordDetail } from "./useRecordDetail";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getEncounterReference } from "../../api/atlasApi";
import {
  useDefinitionDetail,
  definitionDetailQueryOptions,
} from "./useDefinitionDetail";
import type {
  ReferenceHelpHandler,
  ReferenceHelpReturn,
  ReferenceHelpCloseReason,
} from "./ReferenceHelp";
import {
  isArtifactGenerationChanged,
  retireReferenceQueries,
} from "../queries/referenceQueries";
import { recordDetailQueryOptions } from "./recordDetailQuery";

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

function previewBookmark(anchor?: HTMLElement) {
  const drawer = anchor?.closest<HTMLElement>(".record-preview-drawer");
  const pane = drawer?.querySelector<HTMLElement>(".record-preview-drawer__body");
  const focusRegion = anchor?.closest(".record-preview-drawer__header")
    ? ("header" as const)
    : pane?.contains(anchor ?? null)
      ? ("body" as const)
      : undefined;
  return {
    scroll: pane?.scrollTop,
    focusHref: anchor?.closest("a")?.getAttribute("href") ?? undefined,
    focusRegion,
  };
}

type ReadingFrame = {
  destination: ReferenceDestinationView;
  help?: ReferenceHelpView;
  generation?: string;
  selection?: RecordNavigationView;
  landing?: RecordNavigationView;
  context?: { identity: string; generation: string | null; conditionId?: bigint };
};

type ReadingBookmark = ReturnType<typeof previewBookmark> & { scroll: number };

function sameDocument(current: ReadingFrame, next: ReadingFrame) {
  if (
    current.destination.kind === "definition" &&
    next.destination.kind === "definition"
  ) {
    return (
      current.destination.key === next.destination.key &&
      current.help?.definition_key === next.help?.definition_key
    );
  }
  return (
    current.destination.kind === "record" &&
    next.destination.kind === "record" &&
    current.destination.navigation.record_key ===
      next.destination.navigation.record_key &&
    JSON.stringify(current.help) === JSON.stringify(next.help) &&
    JSON.stringify(current.selection) === JSON.stringify(next.selection) &&
    JSON.stringify(current.landing) === JSON.stringify(next.landing) &&
    current.context?.conditionId === next.context?.conditionId &&
    current.context?.identity === next.context?.identity
  );
}

export function useRecordPreview(encounterContext?: EncounterPreviewContext) {
  const queryClient = useQueryClient();
  const [frame, setFrame] = useState<ReadingFrame | null>(null);
  const {
    help,
    generation,
    selection,
    landing,
    context: requestedContext,
  } = frame ?? {};
  const destination = frame?.destination ?? null;
  const recordKey =
    destination?.kind === "record" ? destination.navigation.record_key : null;
  const [anchor, setAnchor] = useState<RecordPreviewAnchor | null>(null);
  const [history, setHistory] = useState<
    { frame: ReadingFrame; bookmark: ReadingBookmark }[]
  >([]);
  const [restore, setRestore] = useState<ReadingBookmark>({
    scroll: 0,
    focusHref: undefined,
    focusRegion: undefined,
  });
  const [restoringHistory, setRestoringHistory] = useState(false);
  const opener = useRef<HTMLElement | null>(null);
  const returnToOwner = useRef<ReferenceHelpReturn | undefined>(undefined);
  // Openers include prepared HTML links outside React's prop ownership. Keep
  // their expanded state with the preview owner, including nested navigation.
  useLayoutEffect(() => {
    const element = opener.current;
    if (!destination || !element) return;
    const previous = element.getAttribute("aria-expanded");
    element.setAttribute("aria-expanded", "true");
    return () => {
      if (previous === null) element.removeAttribute("aria-expanded");
      else element.setAttribute("aria-expanded", previous);
    };
  }, [destination]);
  const session = useRef(0);
  const [reloadState, setReloadState] = useState<
    { kind: "loading" } | { kind: "finished" } | { kind: "failed"; error: Error } | null
  >(null);
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
    meta: referenceQueryMeta,
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
  const detail = useRecordDetail(
    requestedContext || reloadState?.kind === "loading" ? null : recordKey,
    selection,
    generation,
  );
  const definition = useDefinitionDetail(
    reloadState?.kind === "loading"
      ? null
      : destination?.kind === "definition"
        ? destination.key
        : null,
    generation,
  );

  const refreshReference = async () => {
    if (!destination || reloadState?.kind === "loading") return;
    const currentSession = session.current;
    const focused = document.activeElement;
    const bookmark = previewBookmark(
      focused instanceof HTMLElement ? focused : undefined,
    );
    setRestore({
      ...bookmark,
      scroll: bookmark.scroll ?? restore.scroll,
    });
    setRestoringHistory(true);
    setReloadState({ kind: "loading" });
    try {
      await retireReferenceQueries(queryClient);
      if (session.current !== currentSession) return;
      const key =
        destination.kind === "definition" ? destination.key : help?.definition_key;
      if (key) {
        const current = await queryClient.fetchQuery(definitionDetailQueryOptions(key));
        if (session.current !== currentSession) return;
        const updated = current.help?.destination ?? {
          kind: "definition" as const,
          key,
        };
        setFrame(
          (prior) =>
            prior && {
              ...prior,
              destination: updated,
              help: current.help ?? undefined,
              generation: current.artifact_generation,
              landing: undefined,
              selection: updated.kind === "record" ? updated.navigation : undefined,
            },
        );
        if (!current.help) {
          setReloadState({
            kind: "failed",
            error: new RecordContextError("definitionUnavailable"),
          });
          return;
        }
      } else if (recordKey) {
        const current = await queryClient.fetchQuery(
          recordDetailQueryOptions(recordKey, selection),
        );
        if (session.current !== currentSession) return;
        setFrame(
          (prior) => prior && { ...prior, generation: current.artifact_generation },
        );
      }
      setReloadState(null);
    } catch (error) {
      if (session.current === currentSession)
        setReloadState({
          kind: "failed",
          error:
            error instanceof Error
              ? error
              : new Error("Unable to refresh this reference."),
        });
    }
  };

  const reloadOwner = async () => {
    if (!requestedContext) {
      await refreshReference();
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

  const finishSession = useCallback((reason: ReferenceHelpCloseReason) => {
    const returnOwner = returnToOwner.current;
    returnToOwner.current = undefined;
    if (!returnOwner && reason === "return" && opener.current?.isConnected)
      opener.current.focus({ preventScroll: true });
    opener.current = null;
    session.current += 1;
    setReloadState(null);
    setFrame(null);
    setAnchor(null);
    setHistory([]);
    setRestoringHistory(false);
    returnOwner?.(reason);
  }, []);
  const close = useCallback(() => finishSession("return"), [finishSession]);
  const closeForNavigation = useCallback(
    () => finishSession("navigate"),
    [finishSession],
  );

  useDismissOnReadingNavigation(destination !== null, closeForNavigation);

  const openFrame = useCallback(
    (next: ReadingFrame, anchor?: HTMLElement) => {
      if (frame && !sameDocument(frame, next)) {
        const bookmark = previewBookmark(anchor);
        setHistory((prior) =>
          [
            ...prior,
            { frame, bookmark: { ...bookmark, scroll: bookmark.scroll ?? 0 } },
          ].slice(-24),
        );
      }
      opener.current ??=
        anchor ??
        (document.activeElement instanceof HTMLElement ? document.activeElement : null);
      session.current += 1;
      setReloadState(null);
      setFrame(next);
      setAnchor((current) => current ?? anchor ?? null);
      setRestore({ scroll: 0, focusHref: undefined, focusRegion: undefined });
      setRestoringHistory(false);
    },
    [frame],
  );

  const open = useCallback(
    (
      nextRecordKey: string,
      anchor?: HTMLElement,
      navigation?: RecordNavigationView,
      landing?: RecordNavigationView,
    ) => {
      const condition = encounterContext?.conditions?.find(
        (condition) =>
          condition.condition_key === nextRecordKey &&
          condition.condition_id.toString() === anchor?.dataset.atlasConditionId,
      );
      const destinationNavigation = navigation ?? {
        record_key: nextRecordKey,
        embedded_path: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      };
      const contextual =
        (encounterContext?.parentRecordKey === nextRecordKey && navigation) ||
        condition;
      openFrame(
        {
          destination: { kind: "record", navigation: destinationNavigation },
          selection: condition ? destinationNavigation : navigation,
          landing,
          context:
            contextual && encounterContext
              ? {
                  identity: encounterContext.identity,
                  generation: encounterContext.generation,
                  conditionId: condition?.condition_id,
                }
              : undefined,
        },
        anchor,
      );
    },
    [encounterContext, openFrame],
  );

  const openHelp: ReferenceHelpHandler = useCallback(
    (help, anchor, generation, finishOwner) => {
      const destination = help.destination;
      returnToOwner.current ??= finishOwner;
      openFrame(
        {
          destination,
          help,
          generation,
          selection: destination.kind === "record" ? destination.navigation : undefined,
        },
        anchor,
      );
    },
    [openFrame],
  );

  const standalone = destination?.kind === "definition";
  const referenceError =
    reloadState?.kind === "failed"
      ? reloadState.error
      : standalone
        ? definition.error
        : detail.error;
  const refreshRequired =
    isArtifactGenerationChanged(referenceError) || reloadState?.kind === "failed";

  return {
    anchor,
    close,
    closeForNavigation,
    destination,
    help: standalone
      ? definition.data
        ? (definition.data.help ?? undefined)
        : help
      : help,
    definition: standalone ? definition.data : undefined,
    openHelp,
    detail: requestedContext
      ? contextualError || reloadState
        ? undefined
        : retainedContextual
      : reloadState
        ? undefined
        : detail.data,
    error: standalone
      ? referenceError
      : requestedContext
        ? reloadState?.kind === "loading"
          ? null
          : reloadState?.kind === "failed"
            ? reloadState.error
            : contextualError
              ? contextualError
              : contextual.error
        : referenceError,
    retry: standalone
      ? () => {
          if (refreshRequired) void refreshReference();
          else void definition.refetch();
        }
      : !requestedContext && refreshRequired
        ? () => void refreshReference()
        : reloadState?.kind === "failed" && encounterContext?.reloadEncounter
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
    loading: standalone
      ? reloadState?.kind === "loading" || definition.isFetching
      : requestedContext
        ? reloadState?.kind === "loading" ||
          (!reloadState &&
            !contextualError &&
            (contextual.isLoading || contextual.isFetching))
        : reloadState?.kind === "loading" || detail.isLoading || detail.isFetching,
    reload:
      requestedContext && !encounterContext?.reloadEncounter
        ? undefined
        : () => {
            void reloadOwner();
          },
    reloadLabel: requestedContext
      ? "Reload encounter"
      : help?.definition_key || standalone
        ? "Refresh definition"
        : "Refresh record",
    retryLabel:
      requestedContext &&
      (reloadState?.kind === "failed" || isArtifactGenerationChanged(contextual.error))
        ? "Reload encounter"
        : refreshRequired
          ? help?.definition_key || standalone
            ? "Refresh definition"
            : "Refresh record"
          : "Retry",
    open,
    changeRank: (spell_rank: number | undefined) => {
      const current =
        selection ??
        (requestedContext ? contextual.data?.selected : detail.data?.selected);
      if (current)
        setFrame(
          (prior) => prior && { ...prior, selection: { ...current, spell_rank } },
        );
    },
    selection,
    landing,
    readingHistory: {
      address: JSON.stringify([
        destination?.kind,
        destination?.kind === "definition"
          ? destination.key
          : recordReadingIdentity(recordKey, selection),
        help?.definition_key,
        generation,
        landing,
      ]),
      restoringHistory,
      restore,
      canGoBack: history.length > 0,
      onBack: () => {
        const prior = history[history.length - 1];
        if (!prior) return;
        session.current += 1;
        setReloadState(null);
        setHistory(history.slice(0, -1));
        setFrame(prior.frame);
        setRestore(prior.bookmark);
        setRestoringHistory(true);
      },
    },
  };
}

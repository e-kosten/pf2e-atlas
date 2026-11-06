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
  definitionHelpMatchesRecord,
  definitionDetailQueryOptions,
} from "./useDefinitionDetail";
import type { ReferenceHelpHandler } from "./ReferenceHelp";
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

export function useRecordPreview(encounterContext?: EncounterPreviewContext) {
  const queryClient = useQueryClient();
  const [destination, setDestination] = useState<ReferenceDestinationView | null>(null);
  const [help, setHelp] = useState<ReferenceHelpView>();
  const [generation, setGeneration] = useState<string>();
  const recordKey =
    destination?.kind === "record" ? destination.navigation.record_key : null;
  const [anchor, setAnchor] = useState<RecordPreviewAnchor | null>(null);
  const [selection, setSelection] = useState<RecordNavigationView>();
  const [landing, setLanding] = useState<RecordNavigationView>();
  type PriorDocument = {
    destination: ReferenceDestinationView;
    help?: ReferenceHelpView;
    generation?: string;
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
  const readingContext =
    destination?.kind === "record" ? destination.reading_context : undefined;
  const detail = useRecordDetail(
    requestedContext || reloadState?.kind === "loading" ? null : recordKey,
    selection,
    readingContext,
    generation,
  );
  const definition = useDefinitionDetail(
    reloadState?.kind === "loading"
      ? null
      : destination?.kind === "definition"
        ? destination.key
        : (help?.definition_key ?? null),
    generation,
  );

  const refreshReference = async () => {
    if (!destination || reloadState?.kind === "loading") return;
    const currentSession = session.current;
    const focused = document.activeElement;
    const scrollHost =
      focused instanceof HTMLElement
        ? focused.closest<HTMLElement>(".record-preview-drawer__body")
        : null;
    setRestore({
      scroll: scrollHost?.scrollTop ?? restore.scroll,
      focusHref:
        focused instanceof Element
          ? (focused.closest("a")?.getAttribute("href") ?? undefined)
          : undefined,
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
        setGeneration(current.artifact_generation);
        setHelp(current.help);
        setLanding(undefined);
        if (!current.help.destination) {
          setDestination({ kind: "definition", key });
          setSelection(undefined);
          setReloadState({
            kind: "failed",
            error: new RecordContextError("definitionUnavailable"),
          });
          return;
        }
        setDestination(current.help.destination);
        setSelection(
          current.help.destination.kind === "record"
            ? current.help.destination.navigation
            : undefined,
        );
      } else if (recordKey) {
        const current = await queryClient.fetchQuery(
          recordDetailQueryOptions(recordKey, selection, undefined, readingContext),
        );
        if (session.current !== currentSession) return;
        setGeneration(current.artifact_generation);
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

  const close = useCallback(() => {
    if (opener.current?.isConnected) opener.current.focus({ preventScroll: true });
    opener.current = null;
    session.current += 1;
    setReloadState(null);
    setDestination(null);
    setHelp(undefined);
    setGeneration(undefined);
    setAnchor(null);
    setSelection(undefined);
    setRequestedContext(undefined);
    setLanding(undefined);
    setHistory([]);
    setRestoringHistory(false);
  }, []);

  useDismissOnReadingNavigation(destination !== null, close);

  const open = useCallback(
    (
      nextRecordKey: string,
      anchor?: HTMLElement,
      navigation?: RecordNavigationView,
      nextLanding?: RecordNavigationView,
      nextHelp?: ReferenceHelpView,
      nextGeneration?: string,
    ) => {
      if (
        destination &&
        (recordKey !== nextRecordKey ||
          JSON.stringify(help) !== JSON.stringify(nextHelp) ||
          JSON.stringify(selection) !== JSON.stringify(navigation) ||
          JSON.stringify(landing) !== JSON.stringify(nextLanding))
      ) {
        const scrollHost = anchor?.closest<HTMLElement>(".record-preview-drawer__body");
        setHistory((prior) =>
          [
            ...prior,
            {
              destination,
              help,
              generation,
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
      setDestination({
        kind: "record",
        navigation: navigation ?? {
          record_key: nextRecordKey,
          embedded_path: [],
          field: null,
          passage: null,
          source_fingerprint: null,
        },
        reading_context:
          nextHelp?.destination?.kind === "record"
            ? nextHelp.destination.reading_context
            : undefined,
      });
      setHelp(nextHelp);
      setGeneration(nextGeneration);
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
        !nextHelp &&
        ((encounterContext?.parentRecordKey === nextRecordKey && navigation) ||
          condition);
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
    [
      encounterContext,
      destination,
      recordKey,
      selection,
      landing,
      requestedContext,
      help,
      generation,
    ],
  );

  const openHelp: ReferenceHelpHandler = useCallback(
    (nextHelp, nextAnchor, nextGeneration) => {
      const next = nextHelp.destination;
      if (!next) return;
      if (next.kind === "record") {
        open(
          next.navigation.record_key,
          nextAnchor,
          next.navigation,
          undefined,
          nextHelp,
          nextGeneration,
        );
        return;
      }
      if (
        destination &&
        (JSON.stringify(destination) !== JSON.stringify(next) ||
          help?.definition_key !== nextHelp.definition_key)
      ) {
        const scrollHost = nextAnchor.closest<HTMLElement>(
          ".record-preview-drawer__body",
        );
        setHistory((prior) =>
          [
            ...prior,
            {
              destination,
              help,
              generation,
              selection,
              landing,
              context: requestedContext,
              scroll: scrollHost?.scrollTop ?? 0,
              focusHref: nextAnchor.closest("a")?.getAttribute("href") ?? undefined,
            },
          ].slice(-24),
        );
      }
      opener.current ??= nextAnchor;
      session.current += 1;
      setReloadState(null);
      setDestination(next);
      setHelp(nextHelp);
      setGeneration(nextGeneration);
      setAnchor((current) => current ?? nextAnchor);
      setSelection(undefined);
      setLanding(undefined);
      setRequestedContext(undefined);
      setRestore({ scroll: 0, focusHref: undefined });
      setRestoringHistory(false);
    },
    [open, destination, help, generation, selection, landing, requestedContext],
  );

  const standalone = destination?.kind === "definition";
  const referenceError =
    reloadState?.kind === "failed"
      ? reloadState.error
      : standalone
        ? definition.error
        : (detail.error ?? definition.error);
  const refreshRequired =
    isArtifactGenerationChanged(referenceError) || reloadState?.kind === "failed";

  return {
    anchor,
    close,
    destination,
    help: standalone
      ? (definition.data?.help ?? help)
      : definition.data &&
          destination?.kind === "record" &&
          definitionHelpMatchesRecord(
            definition.data.help,
            selection ?? destination.navigation,
            readingContext,
          )
        ? definition.data.help
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
      if (current) setSelection({ ...current, spell_rank });
    },
    recordKey,
    selection,
    landing,
    readingHistory: {
      address: JSON.stringify([
        destination?.kind,
        destination?.kind === "definition"
          ? destination.key
          : recordReadingIdentity(recordKey, selection),
        readingContext,
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
        setDestination(prior.destination);
        setHelp(prior.help);
        setGeneration(prior.generation);
        setSelection(prior.selection);
        setLanding(prior.landing);
        setRequestedContext(prior.context);
        setRestore({ scroll: prior.scroll, focusHref: prior.focusHref });
        setRestoringHistory(true);
      },
    },
  };
}

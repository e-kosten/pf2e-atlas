import { useQuery, useQueryClient } from "@tanstack/react-query";
import { persistentReviewQuery } from "./usePersistentReview";
import { conditionLifecycleQuery } from "./conditionLifecycleQuery";
import type {
  ConditionTurnEvent,
  EncounterTurnBoundaryView,
  RollOutcome,
} from "../../generated/atlas";
import { useEffect, useRef, useState } from "react";
import { getEncounterTurnReview } from "../../api/atlasApi";
import type {
  EncounterMutationCommand,
  useEncounterMutations,
} from "./useEncounterMutations";

/** One notification for the completed turn boundary; closing has no gameplay effect. */
export function useEncounterTurnReview({
  encounterRef,
  mutations,
  onAdvance,
  expectedRevision,
  onFinish,
  onPresent,
}: {
  encounterRef: string;
  expectedRevision: bigint;
  mutations: ReturnType<typeof useEncounterMutations>;
  onAdvance: (expectedRevision: bigint) => Promise<boolean>;
  onFinish?: () => void;
  onPresent?: (participantKey: string, event: ConditionTurnEvent) => void;
}) {
  const client = useQueryClient();
  const key = ["encounter-turn-review", encounterRef, expectedRevision.toString()];
  const [preparing, setPreparing] = useState(false);
  const [open, setOpen] = useState(false);
  const [event, setEvent] = useState<ConditionTurnEvent>("end");
  const nextStart = useRef<string | null>(null);
  const [boundaryKey, setBoundaryKey] = useState<string>();
  const [contextKey, setContextKey] = useState(0);
  const [readError, setReadError] = useState<Error | null>(null);
  const [receipt, setReceipt] = useState("");
  const [outcomes, setOutcomes] = useState<RollOutcome[]>([]);
  const active = useRef(true);
  const advancing = useRef(false);
  const unseen = useRef<{
    encounterRef: string;
    sourceRevision: bigint;
    previousBoundary?: string;
  } | null>(null);
  useEffect(() => {
    active.current = true;
    return () => {
      active.current = false;
    };
  }, []);
  const review = useQuery({
    queryKey: key,
    queryFn: () => getEncounterTurnReview(encounterRef),
    retry: false,
    placeholderData: (previous, query) =>
      query?.queryKey[1] === encounterRef ? previous : undefined,
  });
  const presentUnseen = async (
    data: import("../../generated/atlas").EncounterTurnReviewView,
  ) => {
    const pending = unseen.current;
    if (!pending) return;
    const belongsToAdvance = (boundary: EncounterTurnBoundaryView | null) =>
      boundary?.source_revision === pending.sourceRevision &&
      boundary.boundary_key !== pending.previousBoundary;
    const outgoing = belongsToAdvance(data.outgoing) ? data.outgoing! : undefined;
    const incoming =
      belongsToAdvance(data.incoming) &&
      (!outgoing || data.incoming!.boundary_key === outgoing.boundary_key)
        ? data.incoming!
        : undefined;
    const meaningful = (boundary?: EncounterTurnBoundaryView) =>
      !!boundary &&
      (boundary.reminders.length > 0 || boundary.persistent_damage_groups.length > 0);
    const first = meaningful(outgoing)
      ? outgoing
      : meaningful(incoming)
        ? incoming
        : undefined;
    if (pending.encounterRef !== encounterRef || !first) {
      unseen.current = null;
      onFinish?.();
      return;
    }
    const reads: Promise<unknown>[] = [];
    if (
      outgoing &&
      !outgoing.participant_removed &&
      outgoing.persistent_damage_groups.length
    )
      reads.push(
        client.fetchQuery(
          persistentReviewQuery(
            encounterRef,
            outgoing.participant_key,
            data.expected_revision,
            outgoing.boundary_key,
          ),
        ),
      );
    for (const participantKey of new Set(
      (outgoing?.reminders ?? [])
        .filter((r) => r.kind === "condition_recovery")
        .map((r) => r.participant_key),
    )) {
      reads.push(
        client.fetchQuery(
          conditionLifecycleQuery(
            encounterRef,
            participantKey,
            data.expected_revision,
            outgoing!.boundary_key,
          ),
        ),
      );
    }
    await Promise.allSettled(reads);
    if (!active.current || unseen.current !== pending) return;
    unseen.current = null;
    nextStart.current =
      first.event === "end" && meaningful(incoming) ? incoming!.boundary_key : null;
    onPresent?.(first.participant_key, first.event);
    setContextKey((value) => value + 1);
    setBoundaryKey(first.boundary_key);
    setEvent(first.event);
    setOpen(true);
  };
  const refresh = async () => {
    try {
      const data = await client.fetchQuery({
        queryKey: key,
        queryFn: () => getEncounterTurnReview(encounterRef),
        staleTime: 0,
      });
      if (active.current) {
        setReadError(null);
        await presentUnseen(data);
      }
      return data;
    } catch (cause) {
      if (active.current)
        setReadError(
          cause instanceof Error ? cause : new Error("Could not load turn reminders."),
        );
      return undefined;
    }
  };
  const begin = async () => {
    if (mutations.pending || advancing.current) return;
    advancing.current = true;
    unseen.current = null;
    nextStart.current = null;
    setPreparing(true);
    setOpen(false);
    setBoundaryKey(undefined);
    setReceipt("");
    setOutcomes([]);
    try {
      const before = review.data ?? (await refresh());
      if (!active.current || !(await onAdvance(expectedRevision)) || !active.current)
        return;
      unseen.current = {
        encounterRef,
        sourceRevision: expectedRevision,
        previousBoundary:
          before?.outgoing?.boundary_key ?? before?.incoming?.boundary_key,
      };
      const after = await refresh();
      if (!after && active.current) onFinish?.();
    } finally {
      advancing.current = false;
      if (active.current) setPreparing(false);
    }
  };
  const execute = async (
    command: EncounterMutationCommand,
    savedMessage = "",
  ): Promise<Awaited<ReturnType<typeof mutations.run>>["receipt"] | undefined> => {
    try {
      const result = await mutations.run(command);
      if (!result.active || !active.current) return;
      setReceipt(savedMessage);
      const rolls =
        "roll" in result.receipt
          ? result.receipt.roll
            ? [result.receipt.roll]
            : []
          : "groups" in result.receipt
            ? result.receipt.groups.flatMap((group) => (group.roll ? [group.roll] : []))
            : [];
      // Persistent outcomes belong to their damage section/recovery popover.
      if (
        rolls.length &&
        command.kind !== "persistent-damage" &&
        command.kind !== "persistent-recovery"
      ) {
        setOutcomes((previous) => [...previous, ...rolls]);
      }
      if (result.refreshed) await refresh();
      return result.receipt;
    } catch (error) {
      const current = await mutations.retryRefresh();
      if (current && active.current) await refresh();
      throw error;
    }
  };
  return {
    open,
    event,
    hasNext: event === "end" && nextStart.current !== null,
    preparing,
    contextKey,
    boundaryKey,
    begin,
    execute,
    refresh,
    receipt,
    outcomes,
    data: review.data,
    error: readError ?? review.error,
    loading: review.isFetching,
    close: () => {
      if (!open) return;
      unseen.current = null;
      const incoming = review.data?.incoming;
      if (
        event === "end" &&
        nextStart.current &&
        incoming?.boundary_key === nextStart.current
      ) {
        nextStart.current = null;
        onPresent?.(incoming.participant_key, incoming.event);
        setEvent("start");
        setBoundaryKey(incoming.boundary_key);
        setContextKey((value) => value + 1);
        setReceipt("");
        setOutcomes([]);
        return;
      }
      nextStart.current = null;
      onFinish?.();
      setOpen(false);
      setBoundaryKey(undefined);
    },
  };
}

import { useQuery, useQueryClient } from "@tanstack/react-query";
import type { RollOutcome } from "../../generated/atlas";
import { useEffect, useRef, useState } from "react";
import { getEncounterTurnReview } from "../../api/atlasApi";
import type {
  EncounterMutationCommand,
  useEncounterMutations,
} from "./useEncounterMutations";

/** One notification for the turn just completed; closing has no gameplay effect. */
export function useEncounterTurnReview({
  encounterRef,
  mutations,
  onAdvance,
  expectedRevision,
  onFinish,
}: {
  encounterRef: string;
  expectedRevision: bigint;
  mutations: ReturnType<typeof useEncounterMutations>;
  onAdvance: (expectedRevision: bigint) => Promise<boolean>;
  onFinish?: () => void;
}) {
  const client = useQueryClient();
  const key = ["encounter-turn-review", encounterRef, expectedRevision.toString()];
  const [open, setOpen] = useState(false);
  const [boundaryKey, setBoundaryKey] = useState<string>();
  const [contextKey, setContextKey] = useState(0);
  const [readError, setReadError] = useState<Error | null>(null);
  const [receipt, setReceipt] = useState("");
  const [outcomes, setOutcomes] = useState<RollOutcome[]>([]);
  const active = useRef(true);
  const advancing = useRef(false);
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
  const refresh = async () => {
    try {
      const data = await client.fetchQuery({
        queryKey: key,
        queryFn: () => getEncounterTurnReview(encounterRef),
        staleTime: 0,
      });
      if (active.current) setReadError(null);
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
    setOpen(false);
    setBoundaryKey(undefined);
    setReceipt("");
    setOutcomes([]);
    try {
      const before = review.data ?? (await refresh());
      if (!active.current || !(await onAdvance(expectedRevision)) || !active.current)
        return;
      const after = await refresh();
      if (!active.current) return;
      const outgoing = after?.outgoing;
      setContextKey((value) => value + 1);
      if (
        before &&
        outgoing &&
        outgoing.boundary_key !== before.outgoing?.boundary_key &&
        (outgoing.reminders.length > 0 || outgoing.persistent_damage_groups.length > 0)
      ) {
        setBoundaryKey(outgoing.boundary_key);
        setOpen(true);
      } else {
        onFinish?.();
      }
    } finally {
      advancing.current = false;
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
      onFinish?.();
      setOpen(false);
      setBoundaryKey(undefined);
    },
  };
}

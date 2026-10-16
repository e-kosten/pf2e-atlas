import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { getEncounterTurnReview } from "../../api/atlasApi";
import type {
  EncounterMutationCommand,
  useEncounterMutations,
} from "./useEncounterMutations";

export function useEncounterTurnReview({
  encounterRef,
  mutations,
  onAdvance,
}: {
  encounterRef: string;
  mutations: ReturnType<typeof useEncounterMutations>;
  onAdvance: (expectedRevision: bigint) => Promise<boolean>;
}) {
  const [open, setOpen] = useState(false);
  const [receipt, setReceipt] = useState("");
  const active = useRef(true);
  useEffect(() => {
    active.current = true;
    return () => {
      active.current = false;
    };
  }, []);
  const review = useQuery({
    queryKey: ["encounter-turn-review", encounterRef],
    queryFn: () => getEncounterTurnReview(encounterRef),
    enabled: false,
    retry: false,
  });
  const refresh = async () => {
    const result = await review.refetch();
    return active.current && !result.isError ? result.data : undefined;
  };
  const begin = async () => {
    if (mutations.pending || review.isFetching) return;
    setReceipt("");
    const result = await refresh();
    if (!result) return;
    if (result.reminders.length) setOpen(true);
    else await onAdvance(result.expected_revision);
  };
  const advance = async () => {
    if (!review.data || review.isFetching || review.error || mutations.pending) return;
    if (await onAdvance(review.data.expected_revision)) setOpen(false);
  };
  const execute = async (command: EncounterMutationCommand, savedMessage: string) => {
    try {
      const result = await mutations.run(command);
      if (!result.active || !active.current) return false;
      setReceipt(savedMessage);
      if (result.refreshed) await refresh();
      // The receipt confirms this action is saved even when a following read fails.
      // Clear its amount draft; recovery re-reads state and never repeats the write.
      return true;
    } catch (error) {
      // Reconcile current data and the review token after a rejected action;
      // this read never retries the write or advances the turn.
      const current = await mutations.retryRefresh();
      if (current && active.current) await refresh();
      throw error;
    }
  };
  return {
    open,
    close: () => setOpen(false),
    begin,
    advance,
    execute,
    refresh,
    receipt,
    data: review.data,
    error: review.error,
    loading: review.isFetching,
  };
}

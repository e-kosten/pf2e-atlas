import {
  CancelledError,
  isCancelledError,
  useQueryClient,
  type Query,
} from "@tanstack/react-query";
import { useLayoutEffect, useRef, useState } from "react";
import {
  referenceReadingFallback,
  type ReadingRequest,
  type ReferenceReading,
} from "./useReferenceReading";

import { recordDetailQueryOptions } from "./recordDetailQuery";
import type { RecordNavigationView } from "../../generated/atlas";

type Selection = {
  request: ReadingRequest;
  name: string;
  generation: string | null;
  commit: () => void;
};
type Attempt = Selection & {
  ticket: number;
  epoch: number | string;
  origin: ReadingRequest | null;
  loading: boolean;
  slow: boolean;
  error: Error | null;
};

/** Prepare a different identity without changing the current document or Add target. */
export function usePreparedReferenceSelection(
  reading: ReferenceReading,
  enabled: boolean,
  epoch: number | string,
  generation: string | null,
) {
  const client = useQueryClient();
  const sequence = useRef(0);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const [attempt, setAttempt] = useState<Attempt | null>(null);
  // Closing, retiring the collection, or deliberately navigating the current
  // reader cancels this UI intent. Do not cancel a shared cache query globally.
  useLayoutEffect(
    () => () => {
      sequence.current += 1;
      clearTimeout(timer.current);
    },
    [enabled, epoch, generation, reading.request],
  );
  const current =
    enabled &&
    attempt?.epoch === epoch &&
    attempt.generation === generation &&
    attempt.origin === reading.request
      ? attempt
      : null;
  if (attempt && !current) setAttempt(null);

  async function select(selection: Selection) {
    if (!enabled) return;
    clearTimeout(timer.current);
    const ticket = ++sequence.current;
    const next: Attempt = {
      ...selection,
      ticket,
      epoch,
      origin: reading.request,
      loading: true,
      slow: false,
      error: null,
    };
    setAttempt(next);
    timer.current = setTimeout(() => {
      if (sequence.current === ticket)
        setAttempt((value) =>
          value?.ticket === ticket ? { ...value, slow: true } : value,
        );
    }, 500);
    const prepared = new Set<Query>();
    const isCurrent = () =>
      sequence.current === ticket &&
      [...prepared].every(
        (query) => client.getQueryCache().get(query.queryHash) === query,
      );
    async function prepareDocument(
      navigation: RecordNavigationView,
      documentGeneration = selection.generation,
    ) {
      const options = recordDetailQueryOptions(
        navigation.record_key,
        navigation,
        documentGeneration,
      );
      const promise = client.fetchQuery({ ...options, retry: false });
      const sourceQuery = client
        .getQueryCache()
        .find({ queryKey: options.queryKey, exact: true });
      if (sourceQuery) prepared.add(sourceQuery);
      const detail = await promise;
      if (
        sequence.current !== ticket ||
        client.getQueryCache().find({ queryKey: options.queryKey, exact: true }) !==
          sourceQuery
      )
        throw new CancelledError();
      return detail;
    }
    try {
      const parent = await prepareDocument(selection.request.navigation);
      const fallback = referenceReadingFallback(selection.request, parent);
      if (fallback) await prepareDocument(fallback, parent.artifact_generation);
      if (!isCurrent()) throw new CancelledError();
      // React batches these updates: the candidate, reading address and already
      // cached exact document become visible together, with no loading shell.
      reading.select(selection.request);
      selection.commit();
      setAttempt(null);
    } catch (error) {
      if (sequence.current === ticket)
        setAttempt(
          isCancelledError(error)
            ? null
            : {
                ...next,
                loading: false,
                error:
                  error instanceof Error ? error : new Error("Could not load record"),
              },
        );
    } finally {
      if (sequence.current === ticket) clearTimeout(timer.current);
    }
  }
  function cancel() {
    sequence.current += 1;
    clearTimeout(timer.current);
    setAttempt(null);
  }
  return {
    current,
    select,
    cancel,
    retry: () => {
      if (current) void select(current);
    },
  };
}

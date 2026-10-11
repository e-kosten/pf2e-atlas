import { useCallback, useRef, useState } from "react";
import type { RecordPreviewAnchor } from "./recordPreviewTypes";
import type { ActorActivityView, RecordNavigationView } from "../../generated/atlas";
import { useRecordDetail } from "./useRecordDetail";
import { useQuery } from "@tanstack/react-query";
import { getOwnedReading } from "../../api/atlasApi";
import { activityAddress, activityAnchor } from "./ownedNavigation";

type ActivityPreviewContext = {
  identity: string;
  generation: string | null;
  parentRecordKey: string | undefined;
  activities: ActorActivityView[];
};

export function useRecordPreview(activityContext?: ActivityPreviewContext) {
  const [recordKey, setRecordKey] = useState<string | null>(null);
  const [anchor, setAnchor] = useState<RecordPreviewAnchor | null>(null);
  const [selection, setSelection] = useState<RecordNavigationView>();
  const opener = useRef<HTMLElement | null>(null);
  const [requestedContext, setRequestedContext] = useState<{
    identity: string;
    generation: string | null;
  }>();
  const selectedAddress = selection ? activityAddress(selection) : undefined;
  const currentActivity =
    requestedContext && selectedAddress
      ? activityContext?.activities.find(
          (activity) =>
            activityAnchor(activity.navigation) === activityAnchor(selectedAddress),
        )
      : undefined;
  const contextualError = requestedContext
    ? requestedContext.identity !== activityContext?.identity
      ? "The selected participant has changed. Reopen the activity for this participant."
      : !requestedContext.generation
        ? "Authored activity content is unavailable for this participant."
        : requestedContext.generation !== activityContext?.generation
          ? "The reference collection has changed. Reload the encounter and reopen the activity."
          : !currentActivity
            ? "This activity is no longer available on the current participant."
            : undefined
    : undefined;
  // Contextual content uses a separate cache. Adjusted activity facts are always
  // resolved from the current supplied presentation, never from cached detail.
  const authored = useQuery({
    queryKey: [
      "owned-reading",
      requestedContext?.generation,
      recordKey,
      "activity",
      selectedAddress,
    ],
    enabled: Boolean(requestedContext && !contextualError && selectedAddress),
    queryFn: ({ signal }) =>
      getOwnedReading(
        {
          artifact_generation: requestedContext!.generation!,
          parent: {
            record_key: recordKey!,
            owners: [],
            field: null,
            passage: null,
            source_fingerprint: null,
          },
          selection: { kind: "activity", navigation: selectedAddress! },
        },
        signal,
      ),
    staleTime: Infinity,
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
      const address = navigation ? activityAddress(navigation) : undefined;
      const contextual =
        activityContext?.parentRecordKey === nextRecordKey &&
        address &&
        activityContext.activities.some(
          (activity) => activityAnchor(activity.navigation) === activityAnchor(address),
        );
      setRequestedContext(
        contextual
          ? {
              identity: activityContext.identity,
              generation: activityContext.generation,
            }
          : undefined,
      );
    },
    [activityContext],
  );

  return {
    anchor,
    close,
    detail: detail.data,
    error: requestedContext
      ? contextualError
        ? new Error(contextualError)
        : authored.error
      : detail.error,
    retry: () => {
      if (requestedContext) void authored.refetch();
      else void detail.refetch();
    },
    loading: requestedContext
      ? !contextualError && (authored.isLoading || authored.isFetching)
      : detail.isLoading || detail.isFetching,
    contextual: requestedContext
      ? {
          recoverable: !contextualError,
          activity: contextualError ? undefined : currentActivity,
          entry: contextualError
            ? undefined
            : authored.data?.entries.find(
                (entry) =>
                  activityAnchor(entry.navigation) === activityAnchor(selectedAddress!),
              ),
        }
      : undefined,
    open,
    recordKey,
    selection,
  };
}

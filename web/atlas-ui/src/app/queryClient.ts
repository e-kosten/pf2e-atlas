import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query";
import {
  AtlasConnectionError,
  getAvailabilityFailure,
  isAvailabilityFailure,
  isWorkspaceFailure,
  clearAvailabilityFailure,
  reportAvailabilityFailure,
} from "../api/availability";
import { AtlasApiError } from "../api/atlasApi";
import { AtlasCompatibilityError } from "../api/compatibility";

export function createAtlasQueryClient() {
  const client = new QueryClient({
    queryCache: new QueryCache({
      onError: reportAvailabilityFailure,
      onSuccess: (data, query) => {
        const failure = getAvailabilityFailure();
        const readiness =
          query.queryKey[0] === "readiness" &&
          typeof data === "object" &&
          data &&
          "status" in data
            ? data
            : null;
        const ready = readiness?.status === "ready";
        const workspaceReady =
          ready ||
          !!(readiness && "reference_error" in readiness && readiness.reference_error);
        if (
          !failure ||
          (!(failure instanceof AtlasConnectionError) &&
            !ready &&
            !(workspaceReady && isWorkspaceFailure(failure)))
        )
          return;
        clearAvailabilityFailure();
        // A successful read proves reconnection. Refresh active failed reads;
        // inactive reads become stale for their next mount. Never replay writes.
        void client.invalidateQueries({
          predicate: (entry) =>
            entry.state.error instanceof AtlasConnectionError ||
            (ready && isAvailabilityFailure(entry.state.error)) ||
            (workspaceReady && isWorkspaceFailure(entry.state.error)),
        });
      },
    }),
    mutationCache: new MutationCache({
      onError: (error) => {
        reportAvailabilityFailure(error);
        if (
          error instanceof AtlasApiError &&
          error.appError?.code === "setup_in_progress"
        )
          void client.invalidateQueries({ queryKey: ["readiness"] });
      },
    }),
    defaultOptions: {
      queries: {
        retry: (failureCount, error) => {
          if (error instanceof AtlasCompatibilityError) return false;
          if (error instanceof AtlasApiError && error.status < 500) return false;
          return failureCount < 1;
        },
        staleTime: 20_000,
      },
    },
  });
  // Other Atlas windows and the CLI can change saved data between visits.
  for (const key of [
    "saved-lists",
    "saved-list",
    "saved-list-item-note",
    "parties",
    "encounters",
    "encounter",
    "encounter-route",
  ]) {
    client.setQueryDefaults([key], { refetchOnWindowFocus: "always" });
  }
  return client;
}

import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query";
import {
  AtlasConnectionError,
  getAvailabilityFailure,
  isReferenceFailure,
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
        const ready =
          query.queryKey[0] === "readiness" &&
          typeof data === "object" &&
          data &&
          "status" in data &&
          data.status === "ready";
        if (!failure || (!(failure instanceof AtlasConnectionError) && !ready)) return;
        clearAvailabilityFailure();
        // A successful read proves reconnection. Refresh active failed reads;
        // inactive reads become stale for their next mount. Never replay writes.
        void client.invalidateQueries({
          predicate: (entry) =>
            entry.state.error instanceof AtlasConnectionError ||
            isReferenceFailure(entry.state.error),
        });
      },
    }),
    mutationCache: new MutationCache({ onError: reportAvailabilityFailure }),
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
  return client;
}

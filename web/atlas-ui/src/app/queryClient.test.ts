import { AtlasCompatibilityError } from "../api/compatibility";
import { createAtlasQueryClient } from "./queryClient";
import { AtlasApiError, rollFlatCheck } from "../api/atlasApi";
import { clearCompatibilityFailure } from "../api/compatibility";
import { QueryObserver } from "@tanstack/react-query";
import { waitFor } from "@testing-library/react";
import {
  AtlasConnectionError,
  clearAvailabilityFailure,
  getAvailabilityFailure,
} from "../api/availability";

afterEach(clearAvailabilityFailure);

it("recovers a transient read before reporting a connection failure", async () => {
  const client = createAtlasQueryClient();
  const read = vi
    .fn()
    .mockRejectedValueOnce(new AtlasConnectionError(false, "offline"))
    .mockResolvedValueOnce("ready");
  await expect(
    client.fetchQuery({
      queryKey: ["record"],
      retryDelay: 0,
      queryFn: () => {
        expect(getAvailabilityFailure()).toBeNull();
        return read();
      },
    }),
  ).resolves.toBe("ready");
  expect(read).toHaveBeenCalledTimes(2);
  expect(getAvailabilityFailure()).toBeNull();
  client.clear();
});

it("reports exhausted reads once and clears connection status when readiness recovers", async () => {
  const client = createAtlasQueryClient();
  const error = new AtlasConnectionError(false, "offline");
  const read = vi.fn().mockRejectedValue(error);
  await expect(
    client.fetchQuery({ queryKey: ["record"], retryDelay: 0, queryFn: read }),
  ).rejects.toBe(error);
  expect(read).toHaveBeenCalledTimes(2);
  expect(getAvailabilityFailure()).toBe(error);
  await client.fetchQuery({
    queryKey: ["readiness"],
    queryFn: async () => ({ status: "ready" }),
  });
  expect(getAvailabilityFailure()).toBeNull();
  client.clear();
});

it("never replays an unconfirmed mutation when readiness recovers", async () => {
  const client = createAtlasQueryClient();
  const error = new AtlasConnectionError(true, "connection lost");
  const write = vi.fn().mockRejectedValue(error);
  const mutation = client.getMutationCache().build(client, { mutationFn: write });
  await expect(mutation.execute(undefined)).rejects.toBe(error);
  expect(getAvailabilityFailure()).toBe(error);
  await client.fetchQuery({
    queryKey: ["readiness"],
    queryFn: async () => ({ status: "ready" }),
  });
  await client.invalidateQueries();
  expect(write).toHaveBeenCalledTimes(1);
  client.clear();
});

it("keeps reference unavailability shared until the collection is ready", async () => {
  const client = createAtlasQueryClient();
  const error = new AtlasApiError(422, "projection policy internals", {
    code: "artifact_incompatible",
    message: "projection policy internals",
  });
  await expect(
    client.fetchQuery({
      queryKey: ["record"],
      queryFn: async () => {
        throw error;
      },
    }),
  ).rejects.toBe(error);
  expect(getAvailabilityFailure()).toBe(error);
  await client.fetchQuery({
    queryKey: ["readiness"],
    queryFn: async () => ({ status: "blocked" }),
  });
  expect(getAvailabilityFailure()).toBe(error);
  await client.fetchQuery({
    queryKey: ["readiness"],
    staleTime: 0,
    queryFn: async () => ({ status: "ready" }),
  });
  expect(getAvailabilityFailure()).toBeNull();
  client.clear();
});

it("recovers visible failed reads when another read proves reconnection", async () => {
  const client = createAtlasQueryClient();
  const error = new AtlasConnectionError(false, "offline");
  const read = vi.fn().mockRejectedValueOnce(error).mockResolvedValue("updated record");
  const observer = new QueryObserver(client, {
    queryKey: ["record"],
    queryFn: read,
    retry: false,
  });
  const unsubscribe = observer.subscribe(() => {});
  await waitFor(() => expect(observer.getCurrentResult().error).toBe(error));
  await client.fetchQuery({
    queryKey: ["another-record"],
    queryFn: async () => "ready",
  });
  await waitFor(() => expect(observer.getCurrentResult().data).toBe("updated record"));
  expect(getAvailabilityFailure()).toBeNull();
  expect(read).toHaveBeenCalledTimes(2);
  unsubscribe();
  client.clear();
});

it.each([
  new AtlasApiError(400, "Invalid query"),
  new AtlasCompatibilityError("other-contract"),
])("does not retry a permanent read failure: %s", async (error) => {
  const client = createAtlasQueryClient();
  const read = vi.fn().mockRejectedValue(error);
  await expect(
    client.fetchQuery({ queryKey: ["permanent"], retryDelay: 0, queryFn: read }),
  ).rejects.toBe(error);
  expect(read).toHaveBeenCalledOnce();
  expect(getAvailabilityFailure()).toBeNull();
  client.clear();
});

it("never replays a transient random command during read recovery", async () => {
  clearCompatibilityFailure();
  const fetch = vi.fn().mockRejectedValue(new TypeError("roll response lost"));
  vi.stubGlobal("fetch", fetch);
  const client = createAtlasQueryClient();
  try {
    const roll = client.getMutationCache().build(client, {
      mutationFn: () => rollFlatCheck({ dc: 15n }),
    });
    await expect(roll.execute(undefined)).rejects.toMatchObject({ write: true });
    await client.fetchQuery({
      queryKey: ["readiness"],
      queryFn: async () => ({ status: "ready" }),
    });
    await client.invalidateQueries();
    expect(fetch).toHaveBeenCalledOnce();
    expect(fetch.mock.calls[0][0]).toBe("/api/rolls/flat-check");
    expect(fetch.mock.calls[0][1]).not.toHaveProperty("intent");
  } finally {
    client.clear();
    vi.unstubAllGlobals();
  }
});

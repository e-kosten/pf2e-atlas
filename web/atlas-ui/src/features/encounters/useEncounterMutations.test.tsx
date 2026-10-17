import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { AtlasApiError } from "../../api/atlasApi";
import type { UpdateEncounterParticipantRequest } from "../../generated/atlas";
import { useEncounterMutations } from "./useEncounterMutations";

const api = vi.hoisted(() => ({
  getEncounter: vi.fn(),
  updateEncounterParticipant: vi.fn(),
  updateEncounter: vi.fn(),
  deleteEncounter: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../api/atlasApi")>()),
  ...api,
}));
const request: UpdateEncounterParticipantRequest = {
  participant_key: "pc",
  expected_revision: 3n,
  display_name: "PC",
  side: "pc",
  participant_variant: "normal",
  temporary_hp: 0n,
  defeated: false,
  hidden: false,
  hp_edit: false,
  max_hp_edit: false,
  variant_edit: false,
  use_derived_max: false,
};
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
function mount() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return {
    client,
    ...renderHook(() => useEncounterMutations("old-slug", "encounter-key"), {
      wrapper,
    }),
  };
}
beforeEach(() => {
  vi.resetAllMocks();
  api.updateEncounterParticipant.mockResolvedValue({ encounter_key: "encounter-key" });
  api.getEncounter.mockResolvedValue({
    encounter: { encounter_key: "encounter-key", slug: "renamed-slug" },
  });
});
it("guards same-tick duplicates and remains pending until canonical receipt refresh completes", async () => {
  const write = deferred<{ encounter_key: string }>();
  const read = deferred<unknown>();
  api.updateEncounterParticipant.mockReturnValue(write.promise);
  api.getEncounter.mockReturnValue(read.promise);
  const { result, client } = mount();
  let saving!: ReturnType<typeof result.current.run>;
  await act(async () => {
    saving = result.current.run({ kind: "participant", request });
    await expect(result.current.run({ kind: "participant", request })).rejects.toThrow(
      "Wait",
    );
  });
  expect(api.updateEncounterParticipant).toHaveBeenCalledExactlyOnceWith(
    "encounter-key",
    request,
  );
  await waitFor(() => expect(result.current.pending).toBe(true));
  await act(async () => write.resolve({ encounter_key: "encounter-key" }));
  await waitFor(() =>
    expect(api.getEncounter).toHaveBeenCalledExactlyOnceWith("encounter-key"),
  );
  await waitFor(() => expect(result.current.pending).toBe(true));
  const view = { encounter: { encounter_key: "encounter-key", slug: "renamed-slug" } };
  await act(async () => {
    read.resolve(view);
    await saving;
  });
  await waitFor(() => expect(result.current.pending).toBe(false));
  expect(client.getQueryData(["encounter", "old-slug"])).toEqual(view);
  expect(client.getQueryData(["encounter", "encounter-key"])).toEqual(view);
});
it("reports committed refresh failure and retries only its read", async () => {
  api.getEncounter.mockRejectedValueOnce(new Error("Artifact unavailable"));
  const { result } = mount();
  await act(async () => {
    await expect(
      result.current.run({ kind: "participant", request }),
    ).resolves.toMatchObject({ refreshed: false });
  });
  expect(result.current.error).toBeNull();
  expect(result.current.refreshMessage).toMatch(/^Saved/);
  expect(result.current.refreshError?.message).toBe("Artifact unavailable");
  await expect(result.current.run({ kind: "participant", request })).rejects.toThrow(
    "Wait",
  );
  await act(async () => {
    await result.current.retryRefresh();
  });
  expect(result.current.refreshError).toBeNull();
  expect(api.updateEncounterParticipant).toHaveBeenCalledOnce();
  expect(api.getEncounter).toHaveBeenCalledTimes(2);
});
it.each([false, true])(
  "refreshes a typed participant conflict once without resubmitting (read failure %s)",
  async (readFailure) => {
    const conflict = new AtlasApiError(409, "Stale participant", {
      code: "encounter_participant_conflict",
      message: "Stale participant",
      retryable: false,
    });
    api.updateEncounterParticipant.mockRejectedValue(conflict);
    if (readFailure) api.getEncounter.mockRejectedValue(new Error("Read unavailable"));
    const { result } = mount();
    await act(async () => {
      await expect(result.current.run({ kind: "participant", request })).rejects.toBe(
        conflict,
      );
    });
    expect(result.current.errorMessage).toMatch(/Encounter changed/);
    expect(api.getEncounter).toHaveBeenCalledExactlyOnceWith("encounter-key");
    expect(api.updateEncounterParticipant).toHaveBeenCalledOnce();
    if (readFailure) {
      expect(result.current.refreshMessage).toMatch(/^Participant changed/);
      expect(result.current.refreshMessage).not.toMatch(/Saved/);
      expect(result.current.refreshError?.message).toBe("Read unavailable");
    }
  },
);
it("does not treat an unrelated metadata 409 as a participant conflict", async () => {
  const error = new AtlasApiError(409, "Slug already exists", {
    code: "encounter_already_exists",
    message: "Slug already exists",
  });
  api.updateEncounter.mockRejectedValue(error);
  const { result } = mount();
  await act(async () => {
    await expect(
      result.current.run({
        kind: "metadata",
        request: {
          encounter_key: "encounter-key",
          slug: "old-slug",
          name: "PC",
          status: "draft",
        },
      }),
    ).rejects.toBe(error);
  });
  await waitFor(() => expect(result.current.errorMessage).toBe("Slug already exists"));
  expect(api.getEncounter).not.toHaveBeenCalled();
});

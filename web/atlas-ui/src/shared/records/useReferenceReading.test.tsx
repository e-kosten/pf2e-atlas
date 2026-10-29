import { recordDetailQueryOptions, recordDetailRequest } from "./recordDetailQuery";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { Activity, type ReactNode } from "react";
import { getRecordDetail } from "../../api/atlasApi";
import {
  detailFixture,
  activityFixture,
  actorFixture,
  knownFact,
} from "../../test/fixtures";
import { retireReferenceQueries } from "../queries/referenceQueries";
import { usePreparedReferenceSelection } from "./usePreparedReferenceSelection";
import { useRecordDetail } from "./useRecordDetail";
import { rootNavigation, useReferenceReading } from "./useReferenceReading";
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: vi.fn(),
}));
const api = vi.mocked(getRecordDetail);
function wrapper({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      {children}
    </QueryClientProvider>
  );
}
beforeEach(() => {
  api.mockReset();
});
it("sends an explicit zero rank to the detail service", async () => {
  const selected = { ...rootNavigation("spells:cantrip"), spell_rank: 0 };
  api.mockResolvedValue({ ...detailFixture("spells:cantrip"), selected });
  const { result } = renderHook(() => useReferenceReading({ navigation: selected }), {
    wrapper,
  });
  await waitFor(() => expect(result.current.detail?.selected.spell_rank).toBe(0));
  expect(api).toHaveBeenCalledWith(
    "spells:cantrip",
    {
      record_key: "spells:cantrip",
      embedded_path: [],
      fields: [],
      passage: null,
      source_fingerprint: null,
      spell_rank: 0,
    },
    expect.any(AbortSignal),
  );
});

it("lands a typed inline activity in its parent without reading a separate document", async () => {
  const activity = activityFixture("Jaws", "actors:ghoul");
  const parent = detailFixture("actors:ghoul");
  parent.presentation.body = {
    kind: "creature",
    value: { ...actorFixture(), activities: knownFact([activity]) },
  };
  api.mockResolvedValue(parent);
  const { result } = renderHook(
    () =>
      useReferenceReading({
        navigation: rootNavigation("actors:ghoul"),
        match: activity.navigation,
      }),
    { wrapper },
  );
  await waitFor(() => expect(result.current.detail).toEqual(parent));
  expect(result.current.landing).toEqual(activity.navigation);
  expect(api).toHaveBeenCalledTimes(1);
});
it("reads an exact field address even when its owner is a parent activity", async () => {
  const match = {
    ...activityFixture("Jaws", "actors:ghoul").navigation,
    field: "/system/description/value",
    source_fingerprint: "snapshot",
  };
  api
    .mockResolvedValueOnce(detailFixture("actors:ghoul"))
    .mockResolvedValueOnce({ ...detailFixture("actors:ghoul"), selected: match });
  const { result } = renderHook(
    () => useReferenceReading({ navigation: rootNavigation("actors:ghoul"), match }),
    { wrapper },
  );
  await waitFor(() => expect(result.current.detail?.selected).toEqual(match));
  expect(api).toHaveBeenLastCalledWith(
    "actors:ghoul",
    recordDetailRequest(match),
    expect.any(AbortSignal),
  );
  expect(result.current.landing).toBeUndefined();
});
it("does not start an addressed child read after an abandoned parent finishes late", async () => {
  let finish!: (detail: ReturnType<typeof detailFixture>) => void;
  api
    .mockReturnValueOnce(
      new Promise((resolve) => {
        finish = resolve;
      }),
    )
    .mockResolvedValue(detailFixture("spells:heal"));
  const { result } = renderHook(
    () =>
      useReferenceReading({
        navigation: rootNavigation("actors:ghoul"),
        match: activityFixture("Jaws", "actors:ghoul").navigation,
      }),
    { wrapper },
  );
  await waitFor(() => expect(api).toHaveBeenCalledTimes(1));
  act(() => result.current.inspect({ navigation: rootNavigation("spells:heal") }));
  await waitFor(() =>
    expect(result.current.detail?.record.record_key).toBe("spells:heal"),
  );
  expect(api.mock.calls[0][2]?.aborted).toBe(true);
  await act(async () => finish(detailFixture("actors:ghoul")));
  expect(api).toHaveBeenCalledTimes(2);
  expect(result.current.detail?.record.record_key).toBe("spells:heal");
});

it("bounds local history and restores complete owner, field, passage, fingerprint and landing without changing selection", () => {
  const exact = {
    ...activityFixture("Jaws", "actors:ghoul").navigation,
    field: "system.description.value",
    source_fingerprint: "exact-snapshot",
  };
  const first = { navigation: exact, landing: exact };
  const { result } = renderHook(() => useReferenceReading(first, false), { wrapper });
  expect(result.current.restoringHistory).toBe(false);
  act(() => result.current.inspect({ navigation: rootNavigation("spells:heal") }));
  act(() => result.current.back());
  expect(result.current.request).toEqual(first);
  expect(result.current.restoringHistory).toBe(true);
  expect(result.current.canGoBack).toBe(false);
  for (let i = 0; i < 30; i++)
    act(() => result.current.inspect({ navigation: rootNavigation(`rules:${i}`) }));
  for (let i = 0; i < 24; i++) act(() => result.current.back());
  expect(result.current.request?.navigation.record_key).toBe("rules:5");
  expect(result.current.canGoBack).toBe(false);
  act(() => result.current.select(first));
  expect(result.current.request).toEqual(first);
  expect(result.current.restoringHistory).toBe(false);
  expect(result.current.canGoBack).toBe(false);
});

it("changes a matched spell's rank without re-reading its old match or adding history", async () => {
  const match = activityFixture("Spell", "actors:caster").navigation;
  const selected = { ...detailFixture("actors:caster"), selected: match };
  api
    .mockResolvedValueOnce(detailFixture("actors:caster"))
    .mockResolvedValueOnce(selected);
  const { result } = renderHook(
    () => useReferenceReading({ navigation: rootNavigation("actors:caster"), match }),
    { wrapper },
  );
  await waitFor(() => expect(result.current.detail).toEqual(selected));
  let finish!: (detail: typeof selected) => void;
  api.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  const ranked = { ...match, spell_rank: 5 };
  act(() => result.current.changeRank(ranked));
  await waitFor(() => expect(api).toHaveBeenCalledTimes(3));
  expect(result.current.detail).toEqual(selected);
  expect(result.current.canGoBack).toBe(false);
  expect(api).toHaveBeenLastCalledWith(
    "actors:caster",
    recordDetailRequest(ranked),
    expect.any(AbortSignal),
  );
  await act(async () => finish({ ...selected, selected: ranked }));
  await waitFor(() => expect(result.current.detail?.selected.spell_rank).toBe(5));
  expect(api).toHaveBeenCalledTimes(3);
});

it("shares raw documents between previews and readers while landing stays local", async () => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 20_000 } },
  });
  const parent = detailFixture("actors:ghoul");
  api.mockResolvedValue(parent);
  const { result } = renderHook(
    () => ({
      preview: useRecordDetail("actors:ghoul"),
      reading: useReferenceReading({ navigation: rootNavigation("actors:ghoul") }),
    }),
    {
      wrapper: ({ children }) => (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      ),
    },
  );
  await waitFor(() => expect(result.current.reading.detail).toEqual(parent));
  expect(result.current.preview.data).toEqual(parent);
  expect(api).toHaveBeenCalledTimes(1);
  const landing = activityFixture("Jaws", "actors:ghoul").navigation;
  act(() =>
    result.current.reading.inspect({
      navigation: rootNavigation("actors:ghoul"),
      landing,
    }),
  );
  expect(result.current.reading.landing).toEqual(landing);
  expect(result.current.reading.detail).toEqual(parent);
  expect(api).toHaveBeenCalledTimes(1);
  const cached = client.getQueryData(recordDetailQueryOptions("actors:ghoul").queryKey);
  expect(cached).toEqual(parent);
  expect(cached).not.toHaveProperty("landing");
  expect(cached).not.toHaveProperty("detail");
});

it("reuses an exact fallback already prepared by another reader", async () => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 20_000 } },
  });
  const match = {
    ...activityFixture("Jaws", "actors:ghoul").navigation,
    field: "/system/description/value",
    source_fingerprint: "snapshot",
  };
  const exact = { ...detailFixture("actors:ghoul"), selected: match };
  api.mockResolvedValueOnce(exact).mockResolvedValueOnce(detailFixture("actors:ghoul"));
  await client.fetchQuery(
    recordDetailQueryOptions(match.record_key, match, exact.artifact_generation),
  );
  const { result } = renderHook(
    () =>
      useReferenceReading(
        { navigation: rootNavigation("actors:ghoul"), match },
        true,
        3,
        exact.artifact_generation,
      ),
    {
      wrapper: ({ children }) => (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      ),
    },
  );
  await waitFor(() => expect(result.current.detail).toEqual(exact));
  expect(result.current.landing).toBeUndefined();
  expect(api).toHaveBeenCalledTimes(2);
});

it("drops retained reader content when another surface retires raw references", async () => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 20_000 } },
  });
  const parent = detailFixture("actors:ghoul");
  api.mockResolvedValueOnce(parent);
  const { result } = renderHook(
    () => useReferenceReading({ navigation: rootNavigation("actors:ghoul") }),
    {
      wrapper: ({ children }) => (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      ),
    },
  );
  await waitFor(() => expect(result.current.detail).toEqual(parent));
  api.mockRejectedValue(new Error("Reference offline"));
  await act(async () => {
    await retireReferenceQueries(client);
  });
  await waitFor(() => expect(result.current.error?.message).toBe("Reference offline"));
  expect(result.current.detail).toBeUndefined();
});

it("cannot commit a prepared fallback after its already-loaded parent is removed", async () => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 20_000 } },
  });
  const parent = detailFixture("actors:ghoul");
  const match = {
    ...activityFixture("Jaws", "actors:ghoul").navigation,
    field: "/system/description/value",
  };
  let finish!: (detail: ReturnType<typeof detailFixture>) => void;
  api.mockResolvedValueOnce(parent).mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  const commit = vi.fn();
  const { result } = renderHook(
    () => {
      const reading = useReferenceReading(null, true, 0, parent.artifact_generation);
      return {
        reading,
        selection: usePreparedReferenceSelection(
          reading,
          true,
          0,
          parent.artifact_generation,
        ),
      };
    },
    {
      wrapper: ({ children }) => (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      ),
    },
  );
  let preparing!: Promise<void>;
  act(() => {
    preparing = result.current.selection.select({
      request: { navigation: rootNavigation("actors:ghoul"), match },
      name: "Jaws",
      generation: parent.artifact_generation,
      commit,
    });
  });
  await waitFor(() => expect(api).toHaveBeenCalledTimes(2));
  act(() =>
    client.removeQueries({
      queryKey: recordDetailQueryOptions(
        "actors:ghoul",
        undefined,
        parent.artifact_generation,
      ).queryKey,
      exact: true,
    }),
  );
  await act(async () => {
    finish({ ...parent, selected: match });
    await preparing;
  });
  expect(commit).not.toHaveBeenCalled();
  expect(result.current.reading.request).toBeNull();
  expect(result.current.selection.current).toBeNull();
});

it("does not combine a new parent with an independently cached fallback from another generation", async () => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 20_000 } },
  });
  const match = {
    ...activityFixture("Jaws", "actors:ghoul").navigation,
    field: "/system/description/value",
  };
  const old = {
    ...detailFixture("actors:ghoul"),
    selected: match,
    artifact_generation: "generation-a",
  };
  const parent = {
    ...detailFixture("actors:ghoul"),
    artifact_generation: "generation-b",
  };
  api
    .mockResolvedValueOnce(old)
    .mockResolvedValueOnce(parent)
    .mockResolvedValueOnce({ ...parent, selected: match });
  await client.fetchQuery(recordDetailQueryOptions(match.record_key, match));
  const { result } = renderHook(
    () => useReferenceReading({ navigation: rootNavigation("actors:ghoul"), match }),
    {
      wrapper: ({ children }) => (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      ),
    },
  );
  await waitFor(() =>
    expect(result.current.detail?.artifact_generation).toBe("generation-b"),
  );
  expect(result.current.parentDetail?.artifact_generation).toBe("generation-b");
  expect(api).toHaveBeenCalledTimes(3);
  expect(
    client.getQueryData(recordDetailQueryOptions(match.record_key, match).queryKey),
  ).toEqual(old);
});

it("does not lend a retired document after Activity suspension misses the removal event", async () => {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 20_000 } },
  });
  let visible = true;
  const parent = detailFixture("actors:ghoul");
  api.mockResolvedValueOnce(parent);
  const { result, rerender } = renderHook(
    () => useReferenceReading({ navigation: rootNavigation("actors:ghoul") }),
    {
      wrapper: ({ children }) => (
        <QueryClientProvider client={client}>
          <Activity mode={visible ? "visible" : "hidden"}>{children}</Activity>
        </QueryClientProvider>
      ),
    },
  );
  await waitFor(() => expect(result.current.detail).toEqual(parent));
  visible = false;
  rerender();
  await act(async () => retireReferenceQueries(client));
  api.mockRejectedValue(new Error("New reference offline"));
  visible = true;
  rerender();
  await waitFor(() =>
    expect(result.current.error?.message).toBe("New reference offline"),
  );
  expect(result.current.detail).toBeUndefined();
});

import { recordDetailRequest } from "./recordDetailQuery";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { getRecordDetail } from "../../api/atlasApi";
import {
  detailFixture,
  activityFixture,
  actorFixture,
  knownFact,
} from "../../test/fixtures";
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
beforeEach(() => api.mockReset());
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

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
    { ...match, fields: [match.field] },
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
  act(() => result.current.inspect({ navigation: rootNavigation("spells:heal") }));
  act(() => result.current.back());
  expect(result.current.request).toEqual(first);
  expect(result.current.canGoBack).toBe(false);
  for (let i = 0; i < 30; i++)
    act(() => result.current.inspect({ navigation: rootNavigation(`rules:${i}`) }));
  for (let i = 0; i < 24; i++) act(() => result.current.back());
  expect(result.current.request?.navigation.record_key).toBe("rules:5");
  expect(result.current.canGoBack).toBe(false);
  act(() => result.current.select(first));
  expect(result.current.request).toEqual(first);
  expect(result.current.canGoBack).toBe(false);
});

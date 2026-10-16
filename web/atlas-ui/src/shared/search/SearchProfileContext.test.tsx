import { act, renderHook } from "@testing-library/react";
import { readinessFixture } from "../../test/fixtures";
import type { AppReadinessView } from "../../generated/atlas";
import { ATLAS_ROUTE_CHANGE_EVENT } from "../../app/routes";
import {
  SEARCH_PROFILE_PREFERENCE_KEY,
  useSearchProfileSelection,
} from "./SearchProfileContext";

beforeEach(() => {
  localStorage.clear();
  history.replaceState(null, "", "/search");
});

it.each([
  ["?search_profile=embeddinggemma", "bge-base", "embeddinggemma"],
  ["", "embeddinggemma", "embeddinggemma"],
  ["", null, "bge-base"],
])(
  "resolves URL %s and preference %s only after server metadata",
  (query, preference, expected) => {
    if (preference) localStorage.setItem(SEARCH_PROFILE_PREFERENCE_KEY, preference);
    history.replaceState(null, "", "/search" + query);
    const { result, rerender } = renderHook(
      ({ readiness }: { readiness: AppReadinessView | undefined }) =>
        useSearchProfileSelection(readiness),
      { initialProps: { readiness: undefined as AppReadinessView | undefined } },
    );
    expect(result.current.id).toBeNull();
    expect(result.current.pendingUrlProfile).toBe(query ? "embeddinggemma" : null);
    expect(result.current.loading).toBe(true);
    rerender({ readiness: readinessFixture() });
    expect(result.current.id).toBe(expected);
    expect(result.current.pendingUrlProfile).toBeNull();
    expect(result.current.error).toBeNull();
    rerender({
      readiness: { ...readinessFixture(), default_search_profile: "embeddinggemma" },
    });
    expect(result.current.id).toBe(expected);
  },
);

it("uses the server launch default and keeps unavailable explicit choices visible", () => {
  const readiness = readinessFixture();
  readiness.default_search_profile = "embeddinggemma";
  const launch = renderHook(() => useSearchProfileSelection(readiness));
  expect(launch.result.current.id).toBe("embeddinggemma");
  act(() => {
    history.replaceState(null, "", "/search?search_profile=unknown");
    window.dispatchEvent(new PopStateEvent("popstate"));
  });
  expect(launch.result.current.id).toBe("unknown");
  expect(launch.result.current.error).toContain("not stored");
  expect(launch.result.current.storedVectors).toBe(false);
  act(() => launch.result.current.select("embeddinggemma"));
  readiness.search_profiles[1] = {
    ...readiness.search_profiles[1],
    assets_ready: false,
    usable: false,
  };
  launch.rerender();
  expect(launch.result.current.id).toBe("embeddinggemma");
  expect(launch.result.current.usable).toBe(false);
  expect(launch.result.current.storedVectors).toBe(true);
  expect(launch.result.current.error).toContain("unavailable for text search");
});

it("keeps live tabs independent, retains route changes, and honors explicit Back URLs", () => {
  const first = renderHook(() => useSearchProfileSelection(readinessFixture()));
  const otherTab = renderHook(() => useSearchProfileSelection(readinessFixture()));
  act(() => first.result.current.select("embeddinggemma"));
  expect(otherTab.result.current.id).toBe("bge-base");
  act(() =>
    window.dispatchEvent(
      new StorageEvent("storage", {
        key: SEARCH_PROFILE_PREFERENCE_KEY,
        newValue: "embeddinggemma",
      }),
    ),
  );
  expect(otherTab.result.current.id).toBe("bge-base");
  act(() => {
    history.replaceState(null, "", "/encounters");
    window.dispatchEvent(new Event(ATLAS_ROUTE_CHANGE_EVENT));
  });
  expect(first.result.current.id).toBe("embeddinggemma");
  act(() => {
    history.replaceState(null, "", "/search?search_profile=bge-base");
    window.dispatchEvent(new PopStateEvent("popstate"));
  });
  expect(first.result.current.id).toBe("bge-base");
});

it("keeps deliberate selection working when browser storage is unavailable", () => {
  const get = vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
    throw new Error("blocked");
  });
  const set = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
    throw new Error("blocked");
  });
  try {
    const { result } = renderHook(() => useSearchProfileSelection(readinessFixture()));
    act(() => result.current.select("embeddinggemma"));
    expect(result.current.id).toBe("embeddinggemma");
  } finally {
    get.mockRestore();
    set.mockRestore();
  }
});

it.each(["missing-profile", "embeddinggemma"])(
  "treats unavailable preference %s as an optional launch default",
  (preference) => {
    localStorage.setItem(SEARCH_PROFILE_PREFERENCE_KEY, preference);
    const readiness = readinessFixture();
    readiness.search_profiles[1].usable = false;
    const { result } = renderHook(() => useSearchProfileSelection(readiness));
    expect(result.current.id).toBe("bge-base");
    expect(result.current.error).toBeNull();
  },
);

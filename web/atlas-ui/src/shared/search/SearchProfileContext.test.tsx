import { act, renderHook as renderHookDom } from "@testing-library/react";
import { readinessFixture } from "../../test/fixtures";
import type { AppReadinessView } from "../../generated/atlas";
import { RouterTestContext } from "../../test/router";
import { createAtlasRouter } from "../../app/router";
import { createBrowserHistory, createMemoryHistory } from "@tanstack/react-router";
import type { ReactNode } from "react";
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

it("defers model warnings and retains explicit link intent while the reference is blocked", () => {
  history.replaceState(null, "", "/search?search_profile=unknown");
  const ready = readinessFixture();
  const { result, rerender } = renderHook(
    ({ readiness }: { readiness: AppReadinessView }) =>
      useSearchProfileSelection(readiness),
    {
      initialProps: { readiness: { ...ready, status: "blocked" } as AppReadinessView },
    },
  );
  expect(result.current.id).toBe("unknown");
  expect(result.current.usable).toBe(false);
  expect(result.current.storedVectors).toBe(false);
  expect(result.current.error).toBeNull();
  expect(result.current.canResetLink).toBe(false);
  rerender({ readiness: ready });
  expect(result.current.id).toBe("unknown");
  expect(result.current.error).toContain("not stored");
});

it("uses the server launch default and keeps unavailable explicit choices visible", async () => {
  const readiness = readinessFixture();
  readiness.default_search_profile = "embeddinggemma";
  const launch = renderHook(() => useSearchProfileSelection(readiness));
  expect(launch.result.current.id).toBe("embeddinggemma");
  await act(async () => {
    history.replaceState(null, "", "/search?search_profile=unknown");
    window.dispatchEvent(new PopStateEvent("popstate"));
  });
  expect(launch.result.current.id).toBe("unknown");
  expect(launch.result.current.error).toContain("not stored");
  expect(launch.result.current.storedVectors).toBe(false);
  await act(async () => launch.result.current.select("embeddinggemma"));
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

it("keeps live tabs independent, retains route changes, and honors explicit Back URLs", async () => {
  const firstRouter = createAtlasRouter(
    createMemoryHistory({ initialEntries: ["/search"] }),
  );
  const otherRouter = createAtlasRouter(
    createMemoryHistory({ initialEntries: ["/search"] }),
  );
  const first = renderHookDom(() => useSearchProfileSelection(readinessFixture()), {
    wrapper: ({ children }: { children: ReactNode }) => (
      <RouterTestContext router={firstRouter}>{children}</RouterTestContext>
    ),
  });
  const otherTab = renderHookDom(() => useSearchProfileSelection(readinessFixture()), {
    wrapper: ({ children }: { children: ReactNode }) => (
      <RouterTestContext router={otherRouter}>{children}</RouterTestContext>
    ),
  });
  await act(async () => first.result.current.select("embeddinggemma"));
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
  await act(async () => {
    await firstRouter.navigate({ to: "/encounters" });
  });
  expect(first.result.current.id).toBe("embeddinggemma");
  await act(async () => {
    await firstRouter.navigate({
      to: "/search",
      search: { search_profile: "bge-base" },
    });
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

const renderHook: typeof renderHookDom = (callback, options) => {
  const router = createAtlasRouter(createBrowserHistory());
  return renderHookDom(callback, {
    ...options,
    wrapper: ({ children }) => (
      <RouterTestContext router={router}>{children}</RouterTestContext>
    ),
  });
};

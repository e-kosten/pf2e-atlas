import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { useReferenceGeneration } from "./useReferenceGeneration";
import { retireReferenceQueries } from "../queries/referenceQueries";

function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

function setup() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  const hook = renderHook(
    ({ token, active }) => useReferenceGeneration(token, active),
    { initialProps: { token: "criterion A", active: true }, wrapper },
  );
  return { ...hook, client };
}

describe("reference generation sessions", () => {
  it("adopts once, seeds the final query immediately and retires that seed before any observer mounts", async () => {
    const { result, client } = setup();
    const ticket = result.current.capture();
    const data = { artifact_generation: "A", rows: ["one"] };
    act(() => {
      result.current.adopt(ticket, "A", { queryKey: ["new-reference", "A"], data });
      result.current.adopt(ticket, "B", {
        queryKey: ["new-reference", "B"],
        data: { artifact_generation: "B" },
      });
    });
    expect(result.current.generation).toBe("A");
    expect(client.getQueryData(["new-reference", "A"])).toBe(data);
    expect(client.getQueryData(["new-reference", "B"])).toBeUndefined();
    await retireReferenceQueries(client);
    expect(client.getQueryData(["new-reference", "A"])).toBeUndefined();
  });

  it("rejects aborted and obsolete criteria tickets even when the original criterion becomes current again", () => {
    const { result, rerender, client } = setup();
    const ticket = result.current.capture();
    rerender({ token: "criterion B", active: true });
    rerender({ token: "criterion A", active: true });
    const controller = new AbortController();
    const aborted = result.current.capture(controller.signal);
    controller.abort();
    act(() => {
      result.current.adopt(ticket, "late", { queryKey: ["late"], data: 1 });
      result.current.adopt(aborted, "aborted", { queryKey: ["aborted"], data: 2 });
    });
    expect(result.current.generation).toBeNull();
    expect(client.getQueryData(["late"])).toBeUndefined();
    expect(client.getQueryData(["aborted"])).toBeUndefined();
  });

  it("fences suspended reads while completing recovery for the retained Search session", async () => {
    const { result, rerender } = setup();
    const ticket = result.current.capture();
    const prepare = deferred();
    let recovery!: Promise<void>;
    act(() => {
      recovery = result.current.refresh(() => prepare.promise);
    });
    rerender({ token: "criterion A", active: false });
    await act(async () => {
      prepare.resolve();
      await recovery;
    });
    expect(result.current.recovering).toBe(false);
    expect(result.current.epoch).toBe(1);
    rerender({ token: "criterion A", active: true });
    act(() => result.current.adopt(ticket, "late"));
    expect(result.current.generation).toBeNull();
    act(() => result.current.adopt(result.current.capture(), "B"));
    expect(result.current.generation).toBe("B");
  });

  it("does not complete an obsolete settings replacement into a closed and reopened picker", async () => {
    const { result, rerender, client } = setup();
    const cancellation = deferred();
    const started = deferred();
    const cancel = client.cancelQueries.bind(client);
    vi.spyOn(client, "cancelQueries").mockImplementation(async (filters, options) => {
      await cancel(filters, options);
      started.resolve();
      await cancellation.promise;
    });
    let recovery!: Promise<void>;
    await act(async () => {
      recovery = result.current.applyGeneration("old replacement", () => {});
      await started.promise;
    });
    rerender({ token: "criterion A", active: false });
    act(() => result.current.reset());
    rerender({ token: "criterion A", active: true });
    act(() => {
      result.current.adopt(result.current.capture(), "new session", {
        queryKey: ["new-session", "new session"],
        data: "new content",
      });
    });
    await act(async () => {
      cancellation.resolve();
      await recovery;
    });
    expect(result.current.generation).toBe("new session");
    expect(result.current.epoch).toBe(1);
    expect(result.current.recovering).toBe(false);
    expect(client.getQueryData(["new-session", "new session"])).toBe("new content");
  });

  it("does not begin obsolete retirement when a picker resets during preparation", async () => {
    const { result, client } = setup();
    const prepare = deferred();
    const cancel = vi.spyOn(client, "cancelQueries");
    let recovery!: Promise<void>;
    act(() => {
      recovery = result.current.refresh(() => prepare.promise);
    });
    act(() => result.current.reset());
    await act(async () => {
      prepare.resolve();
      await recovery;
    });
    expect(cancel).not.toHaveBeenCalled();
    expect(result.current.epoch).toBe(1);
    expect(result.current.recovering).toBe(false);
  });

  it("keeps concurrent surface generations independent while a profile change only fences retrieval", async () => {
    const client = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );
    const { result, rerender } = renderHook(
      ({ profile }) => ({
        search: useReferenceGeneration(profile, true),
        picker: useReferenceGeneration("picker", true),
      }),
      { initialProps: { profile: "profile A" }, wrapper },
    );
    const old = result.current.search.capture();
    act(() => result.current.picker.adopt(result.current.picker.capture(), "B"));
    const cancel = vi.spyOn(client, "cancelQueries");
    client.setQueryData(["saved-list"], { draft: "keep" });
    rerender({ profile: "profile B" });
    act(() => result.current.search.adopt(old, "old profile"));
    expect(result.current.search.generation).toBeNull();
    act(() => result.current.search.adopt(result.current.search.capture(), "A"));
    expect(result.current.search.generation).toBe("A");
    expect(result.current.picker.generation).toBe("B");
    expect(client.getQueryData(["saved-list"])).toEqual({ draft: "keep" });
    expect(cancel).not.toHaveBeenCalled();
  });
});

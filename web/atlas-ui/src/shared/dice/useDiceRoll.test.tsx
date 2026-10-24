import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AtlasApiError } from "../../api/atlasApi";
import { useDiceRoll } from "./useDiceRoll";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((complete) => {
    resolve = complete;
  });
  return { promise, resolve };
}
function reducedMotion() {
  vi.spyOn(window, "matchMedia").mockReturnValue({ matches: true } as MediaQueryList);
}
afterEach(() => {
  vi.restoreAllMocks();
  vi.useRealTimers();
});

describe("useDiceRoll", () => {
  it("suppresses duplicate clicks through the request and visual reveal", async () => {
    vi.useFakeTimers();
    const response = deferred<number>();
    const action = vi.fn(() => response.promise);
    const { result } = renderHook(() => useDiceRoll<number>());
    let completion!: Promise<void>;
    act(() => {
      completion = result.current.run(action);
      void result.current.run(action);
    });
    expect(action).toHaveBeenCalledTimes(1);
    expect(result.current.busy).toBe(true);
    await act(async () => {
      response.resolve(17);
      await response.promise;
    });
    expect(result.current.animating).toBe(true);
    act(() => {
      void result.current.run(action);
    });
    expect(action).toHaveBeenCalledTimes(1);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(650);
      await completion;
    });
    expect(result.current.result).toBe(17);
    expect(result.current.busy).toBe(false);
    expect(result.current.animating).toBe(false);
  });

  it("settles immediately with reduced motion", async () => {
    reducedMotion();
    const { result } = renderHook(() => useDiceRoll<number>());
    await act(async () => {
      await result.current.run(async () => 12);
    });
    expect(result.current.result).toBe(12);
    expect(result.current.busy).toBe(false);
    expect(result.current.animating).toBe(false);
  });

  it("removes the previous outcome after failure and preserves actionable API errors", async () => {
    reducedMotion();
    const { result } = renderHook(() => useDiceRoll<number>());
    await act(async () => {
      await result.current.run(async () => 12);
    });
    await act(async () => {
      await result.current.run(async () => {
        throw new AtlasApiError(400, "Reopen this table.");
      });
    });
    expect(result.current.result).toBeUndefined();
    expect(result.current.error).toBe("Reopen this table.");
    expect(result.current.busy).toBe(false);
    await act(async () => {
      await result.current.run(async () => 9);
    });
    expect(result.current.result).toBe(9);
    expect(result.current.error).toBeUndefined();
  });

  it("can retain a previous confirmed outcome without treating a failed roll as success", async () => {
    reducedMotion();
    const { result } = renderHook(() =>
      useDiceRoll<number>({ retainResultOnError: true }),
    );
    await act(async () => {
      await result.current.run(async () => 12);
    });
    const failed = vi.fn(async () => {
      throw new Error("connection interrupted");
    });
    await act(async () => {
      await result.current.run(failed);
    });
    expect(failed).toHaveBeenCalledTimes(1);
    expect(result.current.result).toBe(12);
    expect(result.current.error).toBe("Couldn’t confirm the roll.");
    expect(result.current.busy).toBe(false);
    await act(async () => {
      await result.current.run(async () => 8);
    });
    expect(result.current.result).toBe(8);
    expect(result.current.error).toBeUndefined();
  });

  it("does not retry an uncertain response or surface server internals", async () => {
    reducedMotion();
    const action = vi.fn(async () => {
      throw new AtlasApiError(500, "private server details");
    });
    const { result } = renderHook(() => useDiceRoll<number>());
    await act(async () => {
      await result.current.run(action);
    });
    expect(action).toHaveBeenCalledTimes(1);
    expect(result.current.error).toBe("Couldn’t confirm the roll.");
  });

  it("allows an in-flight action to complete after unmount without starting motion", async () => {
    vi.useFakeTimers();
    const response = deferred<number>();
    const action = vi.fn(() => response.promise);
    const { result, unmount } = renderHook(() => useDiceRoll<number>());
    let completion!: Promise<void>;
    act(() => {
      completion = result.current.run(action);
    });
    unmount();
    response.resolve(8);
    await completion;
    expect(action).toHaveBeenCalledTimes(1);
    expect(vi.getTimerCount()).toBe(0);
  });
});

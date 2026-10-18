import { AtlasApiError } from "../../api/atlasApi";
import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
import { rootNavigation } from "./useReferenceReading";

describe("same-document updates", () => {
  it("invalidates retained content even when a failed query still has cached data", () => {
    const data = { rank: 3 };
    const { result, rerender } = renderHook(
      ({ error }) => useRetainedRecord("spell", data, error),
      { initialProps: { error: null as Error | null } },
    );
    rerender({
      error: new AtlasApiError(409, "Changed", {
        code: "artifact_generation_changed",
        message: "Changed",
      }),
    });
    expect(result.current).toBeUndefined();
  });
  it("retains content while loading and after failure, replacing it only on success", () => {
    const old = { rank: 3 };
    const next = { rank: 5 };
    const { result, rerender } = renderHook(
      ({ data, error }) => useRetainedRecord("spell", data, error),
      {
        initialProps: {
          data: old as typeof old | undefined,
          error: null as Error | null,
        },
      },
    );
    rerender({ data: undefined, error: null });
    expect(result.current).toBe(old);
    rerender({ data: undefined, error: new Error("offline") });
    expect(result.current).toBe(old);
    rerender({ data: next, error: null });
    expect(result.current).toBe(next);
  });

  it("does not show the previous document in a new source context", () => {
    const { result, rerender } = renderHook(
      ({ key, data }) => useRetainedRecord(key, data, null),
      {
        initialProps: { key: "participant1", data: "spell" as string | undefined },
      },
    );
    rerender({ key: "participant2", data: undefined });
    expect(result.current).toBeUndefined();
    rerender({ key: "participant1", data: undefined });
    expect(result.current).toBeUndefined();
  });

  it("excludes only rank from source reading identity", () => {
    const root = rootNavigation("spells:one");
    expect(recordReadingIdentity(root.record_key, { ...root, spell_rank: 5 })).toBe(
      recordReadingIdentity(root.record_key),
    );
    for (const change of [
      { field: "/description" },
      { source_fingerprint: "changed" },
      { record_key: "spells:two" },
    ]) {
      const selected = { ...root, ...change };
      expect(recordReadingIdentity(selected.record_key, selected)).not.toBe(
        recordReadingIdentity(root.record_key),
      );
    }
  });
});

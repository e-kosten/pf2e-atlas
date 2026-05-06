import { describe, expect, it } from "vitest";

import {
  createNoopTerminalDebugTraceService,
  createTerminalDebugTraceService,
  isTerminalDebugTraceEnabled,
} from "../../src/tui/debug-trace.js";

describe("terminal debug trace", () => {
  it("is opt-in through PF2E_TUI_DEBUG", () => {
    expect(isTerminalDebugTraceEnabled({})).toBe(false);
    expect(isTerminalDebugTraceEnabled({ PF2E_TUI_DEBUG: "0" })).toBe(false);
    expect(isTerminalDebugTraceEnabled({ PF2E_TUI_DEBUG: "1" })).toBe(true);
    expect(isTerminalDebugTraceEnabled({ PF2E_TUI_DEBUG: "true" })).toBe(true);
    expect(isTerminalDebugTraceEnabled({ PF2E_TUI_DEBUG: "on" })).toBe(true);
  });

  it("does not retain spans when disabled", () => {
    const trace = createNoopTerminalDebugTraceService();
    trace.startSpan("filterExplorer.loadModel").end();

    expect(trace.snapshot()).toEqual({
      enabled: false,
      running: [],
      recent: [],
      slowRecent: [],
      slowThresholdMs: 50,
      slowRetentionMs: 120000,
    });
  });

  it("tracks running and completed spans when enabled", () => {
    let now = 100;
    const trace = createTerminalDebugTraceService({
      enabled: true,
      now: () => now,
      slowThresholdMs: 25,
    });

    const span = trace.startSpan("filterExplorer.loadModel", { mode: "matching" });
    now = 140;
    expect(trace.snapshot().running).toMatchObject([
      {
        name: "filterExplorer.loadModel",
        elapsedMs: 40,
        metadata: { mode: "matching" },
      },
    ]);

    span.end({ rootNodes: 2 });

    expect(trace.snapshot().running).toEqual([]);
    expect(trace.snapshot().recent).toMatchObject([
      {
        name: "filterExplorer.loadModel",
        endedAt: 140,
        elapsedMs: 40,
        metadata: { mode: "matching", rootNodes: "2" },
      },
    ]);
    expect(trace.snapshot().slowRecent).toMatchObject([
      {
        name: "filterExplorer.loadModel",
        endedAt: 140,
        elapsedMs: 40,
        metadata: { mode: "matching", rootNodes: "2" },
      },
    ]);
  });

  it("keeps slow spans separate from high-volume recent spans until retention expires", () => {
    let now = 0;
    const trace = createTerminalDebugTraceService({
      enabled: true,
      now: () => now,
      slowThresholdMs: 25,
      slowRetentionMs: 1000,
    });

    const slowSpan = trace.startSpan("backend.resolveDiscoveryRecordKeys");
    now = 100;
    slowSpan.end();
    for (let index = 0; index < 60; index += 1) {
      const fastSpan = trace.startSpan("filterExplorer.buildScreenModel");
      now += 1;
      fastSpan.end();
    }

    expect(trace.snapshot().recent).not.toContainEqual(
      expect.objectContaining({ name: "backend.resolveDiscoveryRecordKeys" }),
    );
    expect(trace.snapshot().slowRecent).toContainEqual(
      expect.objectContaining({ name: "backend.resolveDiscoveryRecordKeys", elapsedMs: 100 }),
    );

    now = 1200;
    expect(trace.snapshot().slowRecent).not.toContainEqual(
      expect.objectContaining({ name: "backend.resolveDiscoveryRecordKeys" }),
    );
  });
});

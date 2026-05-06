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
      slowThresholdMs: 50,
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
        elapsedMs: 40,
        metadata: { mode: "matching", rootNodes: "2" },
      },
    ]);
  });
});

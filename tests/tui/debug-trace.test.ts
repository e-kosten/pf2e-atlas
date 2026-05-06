import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import {
  createDefaultTerminalDebugTraceFilePath,
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

  it("writes span start and end events to an opt-in trace file", () => {
    const tempDirectory = mkdtempSync(join(tmpdir(), "pf2e-tui-debug-"));
    try {
      const traceFilePath = join(tempDirectory, "nested", "trace.jsonl");
      let now = 1000;
      const trace = createTerminalDebugTraceService({
        enabled: true,
        now: () => now,
        traceFilePath,
      });

      trace.runSpan(
        "filterExplorer.loadModel",
        { mode: "matching" },
        () => {
          now = 1010;
          return trace.runSpan(
            "filterExplorer.loadDomain",
            { targetFields: "traits" },
            () => {
              now = 1030;
              return { rootNodes: 204 };
            },
            (result) => result,
          );
        },
        () => {
          now = 1050;
          return {};
        },
      );

      expect(trace.snapshot().traceFilePath).toBe(traceFilePath);
      const events = readFileSync(traceFilePath, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line) as Record<string, unknown>);

      expect(events).toEqual([
        {
          event: "span_start",
          spanId: 1,
          name: "filterExplorer.loadModel",
          at: 1000,
          metadata: { mode: "matching" },
        },
        {
          event: "span_start",
          spanId: 2,
          parentSpanId: 1,
          name: "filterExplorer.loadDomain",
          at: 1010,
          metadata: { targetFields: "traits" },
        },
        {
          event: "span_end",
          spanId: 2,
          parentSpanId: 1,
          name: "filterExplorer.loadDomain",
          startedAt: 1010,
          endedAt: 1030,
          elapsedMs: 20,
          metadata: { targetFields: "traits", rootNodes: "204" },
        },
        {
          event: "span_end",
          spanId: 1,
          name: "filterExplorer.loadModel",
          startedAt: 1000,
          endedAt: 1050,
          elapsedMs: 50,
          metadata: { mode: "matching" },
        },
      ]);
    } finally {
      rmSync(tempDirectory, { recursive: true, force: true });
    }
  });

  it("tracks parent spans by async context instead of a process-wide stack", async () => {
    const tempDirectory = mkdtempSync(join(tmpdir(), "pf2e-tui-debug-"));
    try {
      const traceFilePath = join(tempDirectory, "trace.jsonl");
      let now = 2000;
      const trace = createTerminalDebugTraceService({
        enabled: true,
        now: () => now,
        traceFilePath,
      });

      await trace.runSpan(
        "filterExplorer.loadModel",
        {},
        async () => {
          const nestedTask = Promise.resolve().then(() => {
            now = 2010;
            return trace.runSpan("filterExplorer.loadDomain", {}, () => {
              now = 2020;
            });
          });
          now = 2005;
          await nestedTask;
        },
        () => ({}),
      );

      now = 2030;
      const sibling = trace.startSpan("filterExplorer.reconcileModel");
      now = 2040;
      sibling.end();

      const events = readFileSync(traceFilePath, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line) as Record<string, unknown>);
      const startsByName = new Map(events.filter((event) => event.event === "span_start").map((event) => [event.name, event]));

      expect(startsByName.get("filterExplorer.loadDomain")).toMatchObject({
        parentSpanId: 1,
      });
      expect(startsByName.get("filterExplorer.reconcileModel")).not.toHaveProperty("parentSpanId");
    } finally {
      rmSync(tempDirectory, { recursive: true, force: true });
    }
  });

  it("builds the default trace file path under the TUI debug cache", () => {
    expect(createDefaultTerminalDebugTraceFilePath("/repo")).toMatch(
      /^\/repo\/\.cache\/tui-debug\/trace-.+\.jsonl$/,
    );
  });
});

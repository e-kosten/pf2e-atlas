import { AsyncLocalStorage } from "node:async_hooks";
import { mkdirSync, appendFileSync } from "node:fs";
import { dirname, join } from "node:path";

export type TerminalDebugTraceMetadataValue = string | number | boolean | null | undefined;

export type TerminalDebugTraceMetadata = Readonly<Record<string, TerminalDebugTraceMetadataValue>>;

export type TerminalDebugTraceSpanSnapshot = {
  readonly id: number;
  readonly name: string;
  readonly startedAt: number;
  readonly endedAt?: number;
  readonly elapsedMs: number;
  readonly status: "running" | "completed";
  readonly metadata: Readonly<Record<string, string>>;
  readonly parentSpanId?: number;
};

export type TerminalDebugTraceSnapshot = {
  readonly enabled: boolean;
  readonly traceFilePath?: string;
  readonly running: readonly TerminalDebugTraceSpanSnapshot[];
  readonly recent: readonly TerminalDebugTraceSpanSnapshot[];
  readonly slowRecent: readonly TerminalDebugTraceSpanSnapshot[];
  readonly slowThresholdMs: number;
  readonly slowRetentionMs: number;
};

export type TerminalDebugTraceSpan = {
  end: (metadata?: TerminalDebugTraceMetadata) => void;
};

export type Pf2eTerminalDebugTraceService = {
  readonly enabled: boolean;
  startSpan: (name: string, metadata?: TerminalDebugTraceMetadata) => TerminalDebugTraceSpan;
  snapshot: () => TerminalDebugTraceSnapshot;
  clear: () => void;
};

const DEBUG_TRACE_BUFFER_SIZE = 50;
const DEBUG_TRACE_SLOW_BUFFER_SIZE = 20;
const DEFAULT_SLOW_THRESHOLD_MS = 50;
const DEFAULT_SLOW_RETENTION_MS = 120_000;

type Clock = () => number;

type RunningSpan = {
  readonly id: number;
  readonly parentSpanId?: number;
  readonly name: string;
  readonly startedAt: number;
  metadata: Record<string, string>;
};

const noopSpan: TerminalDebugTraceSpan = {
  end: () => undefined,
};

const noopSnapshot: TerminalDebugTraceSnapshot = {
  enabled: false,
  running: [],
  recent: [],
  slowRecent: [],
  slowThresholdMs: DEFAULT_SLOW_THRESHOLD_MS,
  slowRetentionMs: DEFAULT_SLOW_RETENTION_MS,
};

type TerminalDebugTraceJsonEvent =
  | {
      readonly event: "span_start";
      readonly spanId: number;
      readonly parentSpanId?: number;
      readonly name: string;
      readonly at: number;
      readonly metadata: Readonly<Record<string, string>>;
    }
  | {
      readonly event: "span_end";
      readonly spanId: number;
      readonly parentSpanId?: number;
      readonly name: string;
      readonly startedAt: number;
      readonly endedAt: number;
      readonly elapsedMs: number;
      readonly metadata: Readonly<Record<string, string>>;
    };

function normalizeMetadata(metadata: TerminalDebugTraceMetadata | undefined): Record<string, string> {
  const normalized: Record<string, string> = {};
  for (const [key, value] of Object.entries(metadata ?? {})) {
    if (value === undefined || value === null) {
      continue;
    }
    normalized[key] = String(value);
  }
  return normalized;
}

function mergeMetadata(
  current: Record<string, string>,
  metadata: TerminalDebugTraceMetadata | undefined,
): Record<string, string> {
  return {
    ...current,
    ...normalizeMetadata(metadata),
  };
}

export function isTerminalDebugTraceEnabled(env: NodeJS.ProcessEnv = process.env): boolean {
  const value = env.PF2E_TUI_DEBUG?.trim().toLowerCase();
  return value === "1" || value === "true" || value === "yes" || value === "on";
}

function formatTraceTimestamp(date = new Date()): string {
  return date.toISOString().replaceAll(":", "-");
}

export function createDefaultTerminalDebugTraceFilePath(cwd = process.cwd()): string {
  return join(cwd, ".cache", "tui-debug", `trace-${formatTraceTimestamp()}.jsonl`);
}

function writeTraceEvent(traceFilePath: string | undefined, event: TerminalDebugTraceJsonEvent): void {
  if (!traceFilePath) {
    return;
  }
  mkdirSync(dirname(traceFilePath), { recursive: true });
  appendFileSync(traceFilePath, `${JSON.stringify(event)}\n`, "utf8");
}

export function createNoopTerminalDebugTraceService(): Pf2eTerminalDebugTraceService {
  return {
    enabled: false,
    startSpan: () => noopSpan,
    snapshot: () => noopSnapshot,
    clear: () => undefined,
  };
}

export function createTerminalDebugTraceService(options: {
  enabled: boolean;
  now?: Clock;
  slowThresholdMs?: number;
  slowRetentionMs?: number;
  traceFilePath?: string;
}): Pf2eTerminalDebugTraceService {
  if (!options.enabled) {
    return createNoopTerminalDebugTraceService();
  }

  const now = options.now ?? (() => Date.now());
  const slowThresholdMs = options.slowThresholdMs ?? DEFAULT_SLOW_THRESHOLD_MS;
  const slowRetentionMs = options.slowRetentionMs ?? DEFAULT_SLOW_RETENTION_MS;
  const traceFilePath = options.traceFilePath;
  const running = new Map<number, RunningSpan>();
  const recent: TerminalDebugTraceSpanSnapshot[] = [];
  const slowRecent: TerminalDebugTraceSpanSnapshot[] = [];
  const activeSpanStack = new AsyncLocalStorage<readonly number[]>();
  let nextId = 1;

  return {
    enabled: true,
    startSpan: (name, metadata) => {
      const id = nextId;
      nextId += 1;
      const currentStack = activeSpanStack.getStore() ?? [];
      const parentSpanId = currentStack.at(-1);
      const span: RunningSpan = {
        id,
        ...(parentSpanId !== undefined ? { parentSpanId } : {}),
        name,
        startedAt: now(),
        metadata: normalizeMetadata(metadata),
      };
      running.set(id, span);
      activeSpanStack.enterWith([...currentStack, id]);
      writeTraceEvent(traceFilePath, {
        event: "span_start",
        spanId: id,
        ...(parentSpanId !== undefined ? { parentSpanId } : {}),
        name,
        at: span.startedAt,
        metadata: span.metadata,
      });
      let ended = false;

      return {
        end: (endMetadata) => {
          if (ended) {
            return;
          }
          ended = true;
          const finishedAt = now();
          const current = running.get(id) ?? span;
          running.delete(id);
          const stack = activeSpanStack.getStore() ?? [];
          const stackIndex = stack.lastIndexOf(id);
          if (stackIndex !== -1) {
            activeSpanStack.enterWith(stack.filter((spanId) => spanId !== id));
          }
          const completedSpan: TerminalDebugTraceSpanSnapshot = {
            id,
            name: current.name,
            ...(current.parentSpanId !== undefined ? { parentSpanId: current.parentSpanId } : {}),
            startedAt: current.startedAt,
            endedAt: finishedAt,
            elapsedMs: Math.max(0, finishedAt - current.startedAt),
            status: "completed",
            metadata: mergeMetadata(current.metadata, endMetadata),
          };
          writeTraceEvent(traceFilePath, {
            event: "span_end",
            spanId: completedSpan.id,
            ...(completedSpan.parentSpanId !== undefined ? { parentSpanId: completedSpan.parentSpanId } : {}),
            name: completedSpan.name,
            startedAt: completedSpan.startedAt,
            endedAt: finishedAt,
            elapsedMs: completedSpan.elapsedMs,
            metadata: completedSpan.metadata,
          });
          recent.unshift(completedSpan);
          if (recent.length > DEBUG_TRACE_BUFFER_SIZE) {
            recent.length = DEBUG_TRACE_BUFFER_SIZE;
          }
          if (completedSpan.elapsedMs >= slowThresholdMs) {
            slowRecent.unshift(completedSpan);
            if (slowRecent.length > DEBUG_TRACE_SLOW_BUFFER_SIZE) {
              slowRecent.length = DEBUG_TRACE_SLOW_BUFFER_SIZE;
            }
          }
        },
      };
    },
    snapshot: () => {
      const snapshotAt = now();
      return {
        enabled: true,
        ...(traceFilePath ? { traceFilePath } : {}),
        running: [...running.values()].map((span) => ({
          id: span.id,
          name: span.name,
          ...(span.parentSpanId !== undefined ? { parentSpanId: span.parentSpanId } : {}),
          startedAt: span.startedAt,
          elapsedMs: Math.max(0, snapshotAt - span.startedAt),
          status: "running",
          metadata: span.metadata,
        })),
        recent,
        slowRecent: slowRecent.filter(
          (span) => span.endedAt === undefined || snapshotAt - span.endedAt <= slowRetentionMs,
        ),
        slowThresholdMs,
        slowRetentionMs,
      };
    },
    clear: () => {
      running.clear();
      activeSpanStack.enterWith([]);
      recent.length = 0;
      slowRecent.length = 0;
    },
  };
}

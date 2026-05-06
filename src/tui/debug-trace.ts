export type TerminalDebugTraceMetadataValue = string | number | boolean | null | undefined;

export type TerminalDebugTraceMetadata = Readonly<Record<string, TerminalDebugTraceMetadataValue>>;

export type TerminalDebugTraceSpanSnapshot = {
  readonly id: number;
  readonly name: string;
  readonly startedAt: number;
  readonly elapsedMs: number;
  readonly status: "running" | "completed";
  readonly metadata: Readonly<Record<string, string>>;
};

export type TerminalDebugTraceSnapshot = {
  readonly enabled: boolean;
  readonly running: readonly TerminalDebugTraceSpanSnapshot[];
  readonly recent: readonly TerminalDebugTraceSpanSnapshot[];
  readonly slowThresholdMs: number;
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
const DEFAULT_SLOW_THRESHOLD_MS = 50;

type Clock = () => number;

type RunningSpan = {
  readonly id: number;
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
  slowThresholdMs: DEFAULT_SLOW_THRESHOLD_MS,
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
}): Pf2eTerminalDebugTraceService {
  if (!options.enabled) {
    return createNoopTerminalDebugTraceService();
  }

  const now = options.now ?? (() => Date.now());
  const slowThresholdMs = options.slowThresholdMs ?? DEFAULT_SLOW_THRESHOLD_MS;
  const running = new Map<number, RunningSpan>();
  const recent: TerminalDebugTraceSpanSnapshot[] = [];
  let nextId = 1;

  return {
    enabled: true,
    startSpan: (name, metadata) => {
      const id = nextId;
      nextId += 1;
      const span: RunningSpan = {
        id,
        name,
        startedAt: now(),
        metadata: normalizeMetadata(metadata),
      };
      running.set(id, span);
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
          recent.unshift({
            id,
            name: current.name,
            startedAt: current.startedAt,
            elapsedMs: Math.max(0, finishedAt - current.startedAt),
            status: "completed",
            metadata: mergeMetadata(current.metadata, endMetadata),
          });
          if (recent.length > DEBUG_TRACE_BUFFER_SIZE) {
            recent.length = DEBUG_TRACE_BUFFER_SIZE;
          }
        },
      };
    },
    snapshot: () => {
      const snapshotAt = now();
      return {
        enabled: true,
        running: [...running.values()].map((span) => ({
          id: span.id,
          name: span.name,
          startedAt: span.startedAt,
          elapsedMs: Math.max(0, snapshotAt - span.startedAt),
          status: "running",
          metadata: span.metadata,
        })),
        recent,
        slowThresholdMs,
      };
    },
    clear: () => {
      running.clear();
      recent.length = 0;
    },
  };
}

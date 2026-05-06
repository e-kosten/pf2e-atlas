export type SearchTraceMetadataValue = string | number | boolean | null | undefined;

export type SearchTraceMetadata = Readonly<Record<string, SearchTraceMetadataValue>>;

export type SearchTraceSpan = {
  end: (metadata?: SearchTraceMetadata) => void;
};

export type SearchTraceSink = {
  startSpan: (name: string, metadata?: SearchTraceMetadata) => SearchTraceSpan;
};

export function traceSync<T>(
  trace: SearchTraceSink | undefined,
  name: string,
  metadata: SearchTraceMetadata,
  task: () => T,
  endMetadata?: (result: T) => SearchTraceMetadata,
): T {
  const span = trace?.startSpan(name, metadata);
  try {
    const result = task();
    span?.end(endMetadata?.(result));
    return result;
  } catch (error) {
    span?.end({ error: (error as Error).message });
    throw error;
  }
}

export async function traceAsync<T>(
  trace: SearchTraceSink | undefined,
  name: string,
  metadata: SearchTraceMetadata,
  task: () => Promise<T>,
  endMetadata?: (result: T) => SearchTraceMetadata,
): Promise<T> {
  const span = trace?.startSpan(name, metadata);
  try {
    const result = await task();
    span?.end(endMetadata?.(result));
    return result;
  } catch (error) {
    span?.end({ error: (error as Error).message });
    throw error;
  }
}

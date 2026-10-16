import { useMemo, useRef } from "react";

// Scroll continuity is independent of query data and encounter form state.
export function useBrowsePositions() {
  const positions = useRef({
    list: 0,
    readers: new Map<string, number>(),
    focus: new Map<string, string>(),
  });
  return useMemo(
    () => ({
      readList: () => positions.current.list,
      saveList: (position: number) => {
        positions.current.list = position;
      },
      readReader: (address: string) => positions.current.readers.get(address) ?? 0,
      saveReader: (address: string, position: number) => {
        positions.current.readers.set(address, position);
        if (positions.current.readers.size > 48) {
          const oldest = positions.current.readers.keys().next().value!;
          positions.current.readers.delete(oldest);
          positions.current.focus.delete(oldest);
        }
      },
      readReaderFocus: (address: string) => positions.current.focus.get(address),
      saveReaderFocus: (address: string, href: string | null | undefined) => {
        if (href) positions.current.focus.set(address, href);
      },
    }),
    [],
  );
}
export type BrowsePositions = ReturnType<typeof useBrowsePositions>;

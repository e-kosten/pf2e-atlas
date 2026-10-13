import { useMemo, useRef } from "react";

// Scroll continuity is independent of query data and encounter form state.
export function useBrowsePositions() {
  const positions = useRef({ list: 0, readers: new Map<string, number>() });
  return useMemo(
    () => ({
      readList: () => positions.current.list,
      saveList: (position: number) => {
        positions.current.list = position;
      },
      readReader: (address: string) => positions.current.readers.get(address) ?? 0,
      saveReader: (address: string, position: number) => {
        positions.current.readers.set(address, position);
      },
    }),
    [],
  );
}
export type BrowsePositions = ReturnType<typeof useBrowsePositions>;

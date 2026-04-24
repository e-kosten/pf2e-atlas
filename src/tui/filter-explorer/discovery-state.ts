import React from "react";

import type { FilterExplorerDiscoveryMode, FilterExplorerDiscoveryState } from "./types.js";

export function useFilterExplorerDiscoveryState(options: {
  initialMode?: FilterExplorerDiscoveryMode;
  availableModes?: readonly FilterExplorerDiscoveryMode[];
  enabled?: boolean;
  resetKey?: string;
} = {}): FilterExplorerDiscoveryState | undefined {
  const {
    availableModes,
    enabled = true,
    initialMode = "matching",
    resetKey,
  } = options;
  const [mode, setMode] = React.useState<FilterExplorerDiscoveryMode>(initialMode);

  React.useEffect(() => {
    setMode(initialMode);
  }, [initialMode, resetKey]);

  return React.useMemo(
    () =>
      enabled
        ? {
            mode,
            ...(availableModes ? { availableModes } : {}),
            onModeChange: (nextMode: FilterExplorerDiscoveryMode) => {
              setMode(nextMode);
            },
          }
        : undefined,
    [availableModes, enabled, mode],
  );
}

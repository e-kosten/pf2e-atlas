import React from "react";

import type { FilterExplorerDiscoveryState, FilterExplorerModeSwitchOption } from "./types.js";

export function useFilterExplorerDiscoveryState<TMode extends string>(options: {
  initialMode?: TMode;
  modes: readonly FilterExplorerModeSwitchOption<TMode>[];
  enabled?: boolean;
  resetKey?: string;
}): FilterExplorerDiscoveryState<TMode> | undefined {
  const {
    modes,
    enabled = true,
    initialMode,
    resetKey,
  } = options;
  const resolvedInitialMode = initialMode ?? modes[0]?.value;
  const [mode, setMode] = React.useState<TMode | undefined>(resolvedInitialMode);

  React.useEffect(() => {
    setMode(resolvedInitialMode);
  }, [resolvedInitialMode, resetKey]);

  return React.useMemo(
    () =>
      enabled && mode
        ? {
            mode,
            modes,
            onModeChange: (nextMode: TMode) => {
              setMode(nextMode);
            },
          }
        : undefined,
    [enabled, mode, modes],
  );
}

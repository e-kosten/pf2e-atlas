import React from "react";

import { useDerivedTagTerminalSize } from "./context.js";
import { clampTerminalTwoPaneLeftWidth } from "./screen-layout.js";
import type { DerivedTagTerminalTwoPaneScreenProps } from "./types.js";

export type TerminalTwoPaneResizeState = {
  leftWidth: number;
  resize: NonNullable<DerivedTagTerminalTwoPaneScreenProps["resize"]>;
};

export function useTerminalTwoPaneResize(options: {
  defaultLeftWidth: number;
  minLeftWidth?: number;
  minRightWidth?: number;
}): TerminalTwoPaneResizeState {
  const size = useDerivedTagTerminalSize();
  const [leftWidth, setLeftWidth] = React.useState(() =>
    clampTerminalTwoPaneLeftWidth(size.width, options.defaultLeftWidth, {
      minLeftWidth: options.minLeftWidth,
      minRightWidth: options.minRightWidth,
    }),
  );

  React.useEffect(() => {
    setLeftWidth((current) =>
      clampTerminalTwoPaneLeftWidth(size.width, current, {
        minLeftWidth: options.minLeftWidth,
        minRightWidth: options.minRightWidth,
      }),
    );
  }, [options.minLeftWidth, options.minRightWidth, size.width]);

  return React.useMemo(
    () => ({
      leftWidth,
      resize: {
        minLeftWidth: options.minLeftWidth,
        minRightWidth: options.minRightWidth,
        onLeftWidthChange: (nextLeftWidth: number) => {
          setLeftWidth(
            clampTerminalTwoPaneLeftWidth(size.width, nextLeftWidth, {
              minLeftWidth: options.minLeftWidth,
              minRightWidth: options.minRightWidth,
            }),
          );
        },
      },
    }),
    [leftWidth, options.minLeftWidth, options.minRightWidth, size.width],
  );
}

import type React from "react";
import type { Box, Text } from "ink";

import type { DerivedTagTerminalTone } from "./types.js";

export type DerivedTagTerminalSurface = "app" | "panel";

const TERMINAL_THEME = {
  surfaces: {
    app: {
      backgroundColor: "black",
    },
    panel: {
      backgroundColor: "black",
    },
  },
  tones: {
    default: {
      color: "white",
    },
    heading: {
      color: "cyan",
      bold: true,
    },
    section: {
      color: "white",
      bold: true,
    },
    dim: {
      color: "gray",
    },
    accent: {
      color: "cyan",
    },
    success: {
      color: "green",
    },
    warning: {
      color: "yellow",
    },
    danger: {
      color: "red",
    },
    selected: {
      color: "black",
      backgroundColor: "white",
      bold: true,
    },
  } satisfies Record<DerivedTagTerminalTone, React.ComponentProps<typeof Text>>,
} satisfies {
  surfaces: Record<DerivedTagTerminalSurface, React.ComponentProps<typeof Box>>;
  tones: Record<DerivedTagTerminalTone, React.ComponentProps<typeof Text>>;
};

export function terminalSurfaceProps(surface: DerivedTagTerminalSurface): React.ComponentProps<typeof Box> {
  return TERMINAL_THEME.surfaces[surface];
}

export function terminalToneProps(tone: DerivedTagTerminalTone): React.ComponentProps<typeof Text> {
  return TERMINAL_THEME.tones[tone];
}

export function terminalBackdropTextProps(
  props: React.ComponentProps<typeof Text>,
  backdropActive: boolean,
): React.ComponentProps<typeof Text> {
  if (!backdropActive) {
    return props;
  }

  return {
    ...props,
    color: "gray",
    backgroundColor: TERMINAL_THEME.surfaces.app.backgroundColor,
  };
}

export function terminalHyperlinkTextProps(
  props: React.ComponentProps<typeof Text>,
): React.ComponentProps<typeof Text> {
  return {
    ...props,
    color: props.color ?? TERMINAL_THEME.tones.accent.color,
    underline: true,
  };
}

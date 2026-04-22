import type { DerivedTagTerminalTwoPaneFocus, DerivedTagTerminalTwoPaneLayoutMode } from "./types.js";

export function getTerminalPaneBodyHeight(
  sessionOrHeight: number | { height: number },
  options: { hasSubtitle?: boolean; footerLineCount?: number },
): number {
  const height = typeof sessionOrHeight === "number" ? sessionOrHeight : sessionOrHeight.height;
  const headerHeight = options.hasSubtitle ? 3 : 2;
  const footerHeight = options.footerLineCount ?? 0;
  const contentHeight = Math.max(0, height - headerHeight - footerHeight);
  return Math.max(0, contentHeight - 2);
}

export function getTerminalTwoPaneDimensions(
  sessionOrWidth: number | { width: number },
  preferredLeftWidth?: number,
): { leftWidth: number; rightWidth: number; separatorWidth: number } {
  const totalWidth = typeof sessionOrWidth === "number" ? sessionOrWidth : sessionOrWidth.width;
  const separatorWidth = 1;
  const leftWidth = Math.max(
    24,
    Math.min(preferredLeftWidth ?? Math.floor(totalWidth * 0.38), totalWidth - separatorWidth - 20),
  );
  const rightWidth = Math.max(20, totalWidth - leftWidth - separatorWidth);

  return { leftWidth, rightWidth, separatorWidth };
}

export function getTerminalTwoPaneDetailWidth(
  sessionOrWidth: number | { width: number },
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  preferredLeftWidth?: number,
): number {
  if (layoutMode === "detail-only") {
    return typeof sessionOrWidth === "number" ? sessionOrWidth : sessionOrWidth.width;
  }

  return getTerminalTwoPaneDimensions(sessionOrWidth, preferredLeftWidth).rightWidth;
}

export function getTerminalThreePaneDimensions(
  sessionOrWidth: number | { width: number },
  preferredLeftWidth?: number,
  preferredCenterWidth?: number,
): { leftWidth: number; centerWidth: number; rightWidth: number; separatorWidth: number } {
  const totalWidth = typeof sessionOrWidth === "number" ? sessionOrWidth : sessionOrWidth.width;
  const separatorWidth = 1;
  const separatorCount = 2;
  const availableWidth = Math.max(3, totalWidth - separatorWidth * separatorCount);
  const minimumPaneWidth = Math.max(12, Math.floor(availableWidth / 3));
  const clampWidth = (value: number, min: number, max: number): number => Math.max(min, Math.min(value, max));

  const maxLeftWidth = Math.max(minimumPaneWidth, availableWidth - minimumPaneWidth * 2);
  const leftWidth = clampWidth(preferredLeftWidth ?? Math.floor(totalWidth * 0.28), minimumPaneWidth, maxLeftWidth);

  const maxCenterWidth = Math.max(minimumPaneWidth, availableWidth - leftWidth - minimumPaneWidth);
  const centerWidth = clampWidth(
    preferredCenterWidth ?? Math.floor(totalWidth * 0.32),
    minimumPaneWidth,
    maxCenterWidth,
  );
  const rightWidth = Math.max(minimumPaneWidth, availableWidth - leftWidth - centerWidth);

  return {
    leftWidth,
    centerWidth,
    rightWidth,
    separatorWidth,
  };
}

export function toggleTerminalTwoPaneFocus(activePane: DerivedTagTerminalTwoPaneFocus): DerivedTagTerminalTwoPaneFocus {
  return activePane === "list" ? "detail" : "list";
}

export function normalizeTerminalTwoPaneLayoutMode(
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  activePane: DerivedTagTerminalTwoPaneFocus,
): DerivedTagTerminalTwoPaneLayoutMode {
  return activePane === "detail" ? layoutMode : "split";
}

export function toggleTerminalTwoPaneLayoutMode(
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  activePane: DerivedTagTerminalTwoPaneFocus,
): DerivedTagTerminalTwoPaneLayoutMode {
  if (activePane !== "detail") {
    return "split";
  }
  return layoutMode === "split" ? "detail-only" : "split";
}

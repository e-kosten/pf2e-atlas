import React from "react";
import { Box, Text } from "ink";

import { useDerivedTagTerminalCapabilities, useDerivedTagTerminalSize } from "./context.js";
import {
  TerminalRows,
  fitToWidth,
  getRenderedTerminalLineCount,
  renderRows,
  terminalToneProps,
} from "./line-rendering.js";
import { getTerminalThreePaneDimensions, getTerminalTwoPaneDimensions } from "./screen-layout.js";
import type {
  DerivedTagTerminalInlinePromptPanelProps,
  DerivedTagTerminalLine,
  DerivedTagTerminalPane,
  DerivedTagTerminalPaneScreenProps,
  DerivedTagTerminalTextScreenProps,
  DerivedTagTerminalThreePaneScreenProps,
  DerivedTagTerminalTwoPaneScreenProps,
} from "./types.js";

export function TerminalHeader({
  title,
  subtitle,
  width,
}: {
  title: string;
  subtitle?: string;
  width: number;
}): React.JSX.Element {
  return (
    <Box flexDirection="column" width={width}>
      <Text wrap="truncate-end" {...terminalToneProps("heading")}>
        {fitToWidth(title, width)}
      </Text>
      {subtitle ? (
        <Text wrap="truncate-end" {...terminalToneProps("accent")}>
          {fitToWidth(subtitle, width)}
        </Text>
      ) : null}
      <Text wrap="truncate-end" {...terminalToneProps("dim")}>
        {fitToWidth("═".repeat(Math.max(0, width)), width)}
      </Text>
    </Box>
  );
}

export function TerminalFooter({
  footer,
  width,
}: {
  footer?: DerivedTagTerminalLine[];
  width: number;
}): React.JSX.Element | null {
  const { hyperlinkSupport } = useDerivedTagTerminalCapabilities();

  if (!footer || footer.length === 0) {
    return null;
  }

  const footerLineCount = getRenderedTerminalLineCount(footer, width, { hyperlinkSupport });
  const renderedFooter = renderRows(footer, width, footerLineCount, { hyperlinkSupport });

  return (
    <Box flexDirection="column" width={width}>
      <TerminalRows lines={renderedFooter} width={width} />
    </Box>
  );
}

export function TerminalPaneView({
  pane,
  width,
  height,
}: {
  pane: DerivedTagTerminalPane;
  width: number;
  height: number;
}): React.JSX.Element {
  const { hyperlinkSupport } = useDerivedTagTerminalCapabilities();
  const bodyHeight = Math.max(0, height - 2);
  const rows = renderRows(pane.lines, width, bodyHeight, { hyperlinkSupport });

  return (
    <Box flexDirection="column" width={width} height={height}>
      <Text wrap="truncate-end" {...terminalToneProps(pane.active ? "selected" : "section")}>
        {fitToWidth(pane.title, width)}
      </Text>
      {height > 1 ? (
        <Text wrap="truncate-end" {...terminalToneProps(pane.active ? "accent" : "dim")}>
          {fitToWidth("─".repeat(Math.max(0, width)), width)}
        </Text>
      ) : null}
      {bodyHeight > 0 ? <TerminalRows lines={rows} width={width} /> : null}
    </Box>
  );
}

export function TerminalInlinePromptPanel({
  title,
  subtitle,
  body,
  footer,
  width,
  height,
  showTopBorder = true,
}: DerivedTagTerminalInlinePromptPanelProps): React.JSX.Element {
  const footerHeight = footer?.length ?? 0;
  const headerHeight = showTopBorder ? 3 : 2;
  const bodyHeight = Math.max(0, height - headerHeight - footerHeight);

  return (
    <Box flexDirection="column" width={width} height={height}>
      {showTopBorder ? (
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>
          {fitToWidth("─".repeat(Math.max(0, width)), width)}
        </Text>
      ) : null}
      <Text wrap="truncate-end" {...terminalToneProps("selected")}>
        {fitToWidth(title, width)}
      </Text>
      <Text wrap="truncate-end" {...terminalToneProps(subtitle ? "accent" : "dim")}>
        {fitToWidth(subtitle ?? "", width)}
      </Text>
      <Box width={width} height={bodyHeight}>
        {body}
      </Box>
      <TerminalFooter footer={footer} width={width} />
    </Box>
  );
}

export function TerminalCenteredOverlayPanel({
  width,
  height,
  children,
}: {
  width: number;
  height: number;
  children: React.ReactNode;
}): React.JSX.Element {
  return (
    <Box width="100%" height="100%" justifyContent="center" alignItems="center">
      <Box width={width} height={height}>
        {children}
      </Box>
    </Box>
  );
}

export function TerminalTextScreen({
  title,
  subtitle,
  body,
  footer,
}: DerivedTagTerminalTextScreenProps): React.JSX.Element {
  const { hyperlinkSupport } = useDerivedTagTerminalCapabilities();
  const { width, height } = useDerivedTagTerminalSize();
  const headerHeight = subtitle ? 3 : 2;
  const footerHeight = footer?.length ?? 0;
  const bodyHeight = Math.max(0, height - headerHeight - footerHeight);
  const rows = renderRows(body, width, bodyHeight, { hyperlinkSupport });

  return (
    <Box flexDirection="column" width={width} height={height}>
      <TerminalHeader title={title} subtitle={subtitle} width={width} />
      <TerminalRows lines={rows} width={width} />
      <TerminalFooter footer={footer} width={width} />
    </Box>
  );
}

export function TerminalPaneScreen({
  title,
  subtitle,
  pane,
  footer,
}: DerivedTagTerminalPaneScreenProps): React.JSX.Element {
  const { width, height } = useDerivedTagTerminalSize();
  const headerHeight = subtitle ? 3 : 2;
  const footerHeight = footer?.length ?? 0;
  const contentHeight = Math.max(0, height - headerHeight - footerHeight);

  return (
    <Box flexDirection="column" width={width} height={height}>
      <TerminalHeader title={title} subtitle={subtitle} width={width} />
      <TerminalPaneView pane={pane} width={width} height={contentHeight} />
      <TerminalFooter footer={footer} width={width} />
    </Box>
  );
}

export function TerminalTwoPaneScreen({
  title,
  subtitle,
  left,
  right,
  footer,
  leftWidth,
}: DerivedTagTerminalTwoPaneScreenProps): React.JSX.Element {
  const size = useDerivedTagTerminalSize();
  const headerHeight = subtitle ? 3 : 2;
  const footerHeight = footer?.length ?? 0;
  const contentHeight = Math.max(0, size.height - headerHeight - footerHeight);
  const dimensions = getTerminalTwoPaneDimensions(size.width, leftWidth);
  const separator = Array.from({ length: Math.max(1, contentHeight) }, () => "│").join("\n");

  return (
    <Box flexDirection="column" width={size.width} height={size.height}>
      <TerminalHeader title={title} subtitle={subtitle} width={size.width} />
      <Box flexDirection="row" width={size.width} height={contentHeight}>
        <TerminalPaneView pane={left} width={dimensions.leftWidth} height={contentHeight} />
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>
          {separator}
        </Text>
        <TerminalPaneView pane={right} width={dimensions.rightWidth} height={contentHeight} />
      </Box>
      <TerminalFooter footer={footer} width={size.width} />
    </Box>
  );
}

export function TerminalThreePaneScreen({
  title,
  subtitle,
  left,
  center,
  right,
  footer,
  leftWidth,
  centerWidth,
}: DerivedTagTerminalThreePaneScreenProps): React.JSX.Element {
  const size = useDerivedTagTerminalSize();
  const headerHeight = subtitle ? 3 : 2;
  const footerHeight = footer?.length ?? 0;
  const contentHeight = Math.max(0, size.height - headerHeight - footerHeight);
  const dimensions = getTerminalThreePaneDimensions(size.width, leftWidth, centerWidth);
  const separator = Array.from({ length: Math.max(1, contentHeight) }, () => "│").join("\n");

  return (
    <Box flexDirection="column" width={size.width} height={size.height}>
      <TerminalHeader title={title} subtitle={subtitle} width={size.width} />
      <Box flexDirection="row" width={size.width} height={contentHeight}>
        <TerminalPaneView pane={left} width={dimensions.leftWidth} height={contentHeight} />
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>
          {separator}
        </Text>
        <TerminalPaneView pane={center} width={dimensions.centerWidth} height={contentHeight} />
        <Text wrap="truncate-end" {...terminalToneProps("dim")}>
          {separator}
        </Text>
        <TerminalPaneView pane={right} width={dimensions.rightWidth} height={contentHeight} />
      </Box>
      <TerminalFooter footer={footer} width={size.width} />
    </Box>
  );
}

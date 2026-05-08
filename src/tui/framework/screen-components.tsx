import React from "react";
import { Box, Text } from "ink";

import {
  useDerivedTagTerminalBackdropActive,
  useDerivedTagTerminalCapabilities,
  useCaptureDerivedTagTerminalPointerEvents,
  useRegisterDerivedTagTerminalPointerRegion,
  useDerivedTagTerminalSize,
  useDerivedTagTerminalViewportSize,
} from "./context.js";
import {
  TerminalRows,
  fitToWidth,
  getRenderedTerminalLineCount,
  renderRows,
} from "./line-rendering.js";
import {
  clampTerminalTwoPaneLeftWidth,
  getTerminalThreePaneDimensions,
  getTerminalTwoPaneDimensions,
} from "./screen-layout.js";
import { terminalBackdropTextProps, terminalSurfaceProps, terminalToneProps } from "./theme.js";
import type {
  DerivedTagTerminalInlinePromptPanelProps,
  DerivedTagTerminalLine,
  DerivedTagTerminalPane,
  DerivedTagTerminalPaneScreenProps,
  DerivedTagTerminalPointerEvent,
  DerivedTagTerminalTextScreenProps,
  DerivedTagTerminalThreePaneScreenProps,
  DerivedTagTerminalTwoPaneScreenProps,
} from "./types.js";

function withBackdropTextProps(
  props: React.ComponentProps<typeof Text>,
  backdropActive: boolean,
): React.ComponentProps<typeof Text> {
  return terminalBackdropTextProps(props, backdropActive);
}

function TerminalPanelSurface({
  width,
  height,
  children,
}: {
  width: number;
  height: number;
  children: React.ReactNode;
}): React.JSX.Element {
  return (
    <Box width={width} height={height} flexDirection="column" {...terminalSurfaceProps("panel")}>
      {children}
    </Box>
  );
}

function useTerminalPanePointerRegion(options: {
  x: number;
  y: number;
  width: number;
  height: number;
  pointerRegion?: DerivedTagTerminalPane["pointerRegion"];
}): void {
  const region = React.useMemo(
    () =>
      options.pointerRegion && options.width > 0 && options.height > 0
        ? {
            rect: {
              x: options.x,
              y: options.y,
              width: options.width,
              height: options.height,
            },
            priority: options.pointerRegion.priority,
            onPointerEvent: options.pointerRegion.onPointerEvent,
          }
        : null,
    [options.height, options.pointerRegion, options.width, options.x, options.y],
  );

  useRegisterDerivedTagTerminalPointerRegion(region);
}

export function TerminalHeader({
  title,
  subtitle,
  width,
}: {
  title: string;
  subtitle?: string;
  width: number;
}): React.JSX.Element {
  const backdropActive = useDerivedTagTerminalBackdropActive();
  return (
    <Box flexDirection="column" width={width}>
      <Text wrap="truncate-end" {...withBackdropTextProps(terminalToneProps("heading"), backdropActive)}>
        {fitToWidth(title, width)}
      </Text>
      {subtitle ? (
        <Text wrap="truncate-end" {...withBackdropTextProps(terminalToneProps("accent"), backdropActive)}>
          {fitToWidth(subtitle, width)}
        </Text>
      ) : null}
      <Text wrap="truncate-end" {...withBackdropTextProps(terminalToneProps("dim"), backdropActive)}>
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
  const backdropActive = useDerivedTagTerminalBackdropActive();
  const { hyperlinkSupport } = useDerivedTagTerminalCapabilities();
  const bodyHeight = Math.max(0, height - 2);
  const rows = renderRows(pane.lines, width, bodyHeight, { hyperlinkSupport });

  return (
    <Box flexDirection="column" width={width} height={height}>
      <Text
        wrap="truncate-end"
        {...withBackdropTextProps(terminalToneProps(pane.active ? "selected" : "section"), backdropActive)}
      >
        {fitToWidth(pane.title, width)}
      </Text>
      {height > 1 ? (
        <Text
          wrap="truncate-end"
          {...withBackdropTextProps(terminalToneProps(pane.active ? "accent" : "dim"), backdropActive)}
        >
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
  const backdropActive = useDerivedTagTerminalBackdropActive();
  const footerHeight = footer?.length ?? 0;
  const headerHeight = showTopBorder ? 3 : 2;
  const bodyHeight = Math.max(0, height - headerHeight - footerHeight);

  return (
    <TerminalPanelSurface width={width} height={height}>
      {showTopBorder ? (
        <Text wrap="truncate-end" {...withBackdropTextProps(terminalToneProps("dim"), backdropActive)}>
          {fitToWidth("─".repeat(Math.max(0, width)), width)}
        </Text>
      ) : null}
      <Text wrap="truncate-end" {...withBackdropTextProps(terminalToneProps("section"), backdropActive)}>
        {fitToWidth(title, width)}
      </Text>
      <Text
        wrap="truncate-end"
        {...withBackdropTextProps(terminalToneProps(subtitle ? "accent" : "dim"), backdropActive)}
      >
        {fitToWidth(subtitle ?? "", width)}
      </Text>
      <Box width={width} height={bodyHeight}>
        {body}
      </Box>
      <TerminalFooter footer={footer} width={width} />
    </TerminalPanelSurface>
  );
}

export function TerminalCenteredOverlayPanel({
  width,
  height,
  children,
  capturePointerEvents = false,
}: {
  width: number;
  height: number;
  children: React.ReactNode;
  capturePointerEvents?: boolean;
}): React.JSX.Element {
  const size = useDerivedTagTerminalViewportSize();
  const topOffset = Math.max(0, Math.floor((Math.max(0, size.height - height)) / 3));
  const leftOffset = Math.max(0, Math.floor((Math.max(0, size.width - width)) / 2));
  const rightOffset = Math.max(0, size.width - leftOffset - width);
  useRegisterDerivedTagTerminalPointerRegion(
    capturePointerEvents
      ? {
          rect: {
            x: leftOffset,
            y: topOffset,
            width,
            height,
          },
          priority: 1000,
          onPointerEvent: () => true,
        }
      : null,
  );

  return (
    <Box position="absolute" flexDirection="column" top={topOffset} left={leftOffset} right={rightOffset}>
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
  useTerminalPanePointerRegion({
    x: 0,
    y: headerHeight,
    width,
    height: contentHeight,
    pointerRegion: pane.pointerRegion,
  });

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
  resize,
}: DerivedTagTerminalTwoPaneScreenProps): React.JSX.Element {
  const backdropActive = useDerivedTagTerminalBackdropActive();
  const capturePointerEvents = useCaptureDerivedTagTerminalPointerEvents();
  const size = useDerivedTagTerminalSize();
  const headerHeight = subtitle ? 3 : 2;
  const footerHeight = footer?.length ?? 0;
  const contentHeight = Math.max(0, size.height - headerHeight - footerHeight);
  const [internalLeftWidth, setInternalLeftWidth] = React.useState(leftWidth);
  const releaseResizeCaptureRef = React.useRef<(() => void) | null>(null);
  const preferredLeftWidth = resize?.onLeftWidthChange ? leftWidth : (internalLeftWidth ?? leftWidth);
  const dimensions = getTerminalTwoPaneDimensions(size.width, preferredLeftWidth, {
    minLeftWidth: resize?.minLeftWidth,
    minRightWidth: resize?.minRightWidth,
  });
  const separator = Array.from({ length: Math.max(1, contentHeight) }, () => "│").join("\n");
  React.useEffect(() => {
    if (!resize?.onLeftWidthChange) {
      setInternalLeftWidth(leftWidth);
    }
  }, [leftWidth, resize?.onLeftWidthChange]);
  const applyLeftWidth = React.useCallback(
    (nextLeftWidth: number) => {
      const clampedLeftWidth = clampTerminalTwoPaneLeftWidth(size.width, nextLeftWidth, {
        minLeftWidth: resize?.minLeftWidth,
        minRightWidth: resize?.minRightWidth,
      });
      if (resize?.onLeftWidthChange) {
        resize.onLeftWidthChange(clampedLeftWidth);
        return;
      }
      setInternalLeftWidth(clampedLeftWidth);
    },
    [resize, size.width],
  );
  const startResize = React.useCallback(
    (event: DerivedTagTerminalPointerEvent) => {
      if (event.kind !== "click") {
        return false;
      }
      releaseResizeCaptureRef.current?.();
      const startX = event.x;
      const startLeftWidth = dimensions.leftWidth;
      const releaseCapture = capturePointerEvents((capturedEvent) => {
        if (capturedEvent.kind === "drag") {
          applyLeftWidth(startLeftWidth + capturedEvent.x - startX);
          return true;
        }
        if (capturedEvent.kind === "release") {
          applyLeftWidth(startLeftWidth + capturedEvent.x - startX);
          releaseResizeCaptureRef.current?.();
          releaseResizeCaptureRef.current = null;
          return true;
        }
        return true;
      });
      releaseResizeCaptureRef.current = releaseCapture;
      return true;
    },
    [applyLeftWidth, capturePointerEvents, dimensions.leftWidth],
  );

  React.useEffect(
    () => () => {
      releaseResizeCaptureRef.current?.();
      releaseResizeCaptureRef.current = null;
    },
    [],
  );

  useTerminalPanePointerRegion({
    x: 0,
    y: headerHeight,
    width: dimensions.leftWidth,
    height: contentHeight,
    pointerRegion: left.pointerRegion,
  });
  useTerminalPanePointerRegion({
    x: dimensions.leftWidth + 1,
    y: headerHeight,
    width: dimensions.rightWidth,
    height: contentHeight,
    pointerRegion: right.pointerRegion,
  });
  useRegisterDerivedTagTerminalPointerRegion({
    rect: {
      x: dimensions.leftWidth,
      y: headerHeight,
      width: dimensions.separatorWidth,
      height: contentHeight,
    },
    priority: 100,
    onPointerEvent: startResize,
  });

  return (
    <Box flexDirection="column" width={size.width} height={size.height}>
      <TerminalHeader title={title} subtitle={subtitle} width={size.width} />
      <Box flexDirection="row" width={size.width} height={contentHeight}>
        <TerminalPaneView pane={left} width={dimensions.leftWidth} height={contentHeight} />
        <Text wrap="truncate-end" {...withBackdropTextProps(terminalToneProps("dim"), backdropActive)}>
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
  const backdropActive = useDerivedTagTerminalBackdropActive();
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
        <Text wrap="truncate-end" {...withBackdropTextProps(terminalToneProps("dim"), backdropActive)}>
          {separator}
        </Text>
        <TerminalPaneView pane={center} width={dimensions.centerWidth} height={contentHeight} />
        <Text wrap="truncate-end" {...withBackdropTextProps(terminalToneProps("dim"), backdropActive)}>
          {separator}
        </Text>
        <TerminalPaneView pane={right} width={dimensions.rightWidth} height={contentHeight} />
      </Box>
      <TerminalFooter footer={footer} width={size.width} />
    </Box>
  );
}

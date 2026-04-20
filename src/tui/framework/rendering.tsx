import React from "react";
import { Box, Text } from "ink";

import { useDerivedTagTerminalSize } from "./context.js";
import type {
  DerivedTagTerminalInlinePromptPanelProps,
  DerivedTagTerminalLine,
  DerivedTagTerminalPane,
  DerivedTagTerminalPaneScreenProps,
  DerivedTagTerminalSegment,
  DerivedTagTerminalTextScreenProps,
  DerivedTagTerminalThreePaneScreenProps,
  DerivedTagTerminalTone,
  DerivedTagTerminalTwoPaneFocus,
  DerivedTagTerminalTwoPaneLayoutMode,
  DerivedTagTerminalTwoPaneScreenProps,
} from "./types.js";

type RenderedTerminalLine = {
  text: string;
  tone: DerivedTagTerminalTone;
  segments?: DerivedTagTerminalSegment[];
};

function normalizeLine(line: DerivedTagTerminalLine): Required<DerivedTagTerminalLine> {
  return {
    text: line.text,
    segments: line.segments ?? [],
    tone: line.tone ?? "default",
    indent: line.indent ?? 0,
    noWrap: line.noWrap ?? false,
  };
}

function segmentText(segments: DerivedTagTerminalSegment[]): string {
  return segments.map((segment) => segment.text).join("");
}

function visibleWidth(text: string): number {
  return [...text].length;
}

function truncateText(text: string, width: number): string {
  if (width <= 0) {
    return "";
  }

  const characters = [...text];
  if (characters.length <= width) {
    return text;
  }

  return characters.slice(0, width).join("");
}

function truncateSegments(segments: DerivedTagTerminalSegment[], width: number): DerivedTagTerminalSegment[] {
  if (width <= 0 || segments.length === 0) {
    return [];
  }

  const truncated: DerivedTagTerminalSegment[] = [];
  let remainingWidth = width;

  for (const segment of segments) {
    if (remainingWidth <= 0) {
      break;
    }
    const segmentWidth = visibleWidth(segment.text);
    if (segmentWidth <= remainingWidth) {
      truncated.push(segment);
      remainingWidth -= segmentWidth;
      continue;
    }
    truncated.push({
      text: truncateText(segment.text, remainingWidth),
      tone: segment.tone,
    });
    break;
  }

  return truncated;
}

export function fitToWidth(text: string, width: number): string {
  const truncated = truncateText(text, width);
  const paddingWidth = Math.max(0, width - visibleWidth(truncated));
  return `${truncated}${" ".repeat(paddingWidth)}`;
}

function wrapPlainText(text: string, width: number): string[] {
  if (width <= 0) {
    return [];
  }

  const normalized = text.replace(/\s+/g, " ").trim();
  if (!normalized) {
    return [""];
  }

  const words = normalized.split(" ");
  const lines: string[] = [];
  let current = "";

  for (const word of words) {
    if (!current) {
      current = word;
      continue;
    }

    const candidate = `${current} ${word}`;
    if (visibleWidth(candidate) <= width) {
      current = candidate;
      continue;
    }

    lines.push(current);
    if (visibleWidth(word) <= width) {
      current = word;
      continue;
    }

    let remaining = word;
    while (visibleWidth(remaining) > width) {
      const segment = truncateText(remaining, width);
      lines.push(segment);
      remaining = [...remaining].slice([...segment].length).join("");
    }
    current = remaining;
  }

  if (current) {
    lines.push(current);
  }

  return lines.length > 0 ? lines : [""];
}

function buildRenderedTerminalLines(lines: DerivedTagTerminalLine[], width: number): RenderedTerminalLine[] {
  const renderedLines: RenderedTerminalLine[] = [];

  for (const rawLine of lines) {
    const line = normalizeLine(rawLine);
    const indent = " ".repeat(Math.max(0, line.indent));
    const usableWidth = Math.max(1, width - indent.length);

    if (line.segments.length > 0) {
      const segmentsWithIndent = indent
        ? [{ text: indent, tone: "default" as const }, ...line.segments]
        : line.segments;
      const renderedSegments = truncateSegments(
        line.noWrap
          ? segmentsWithIndent
          : [{ text: truncateText(segmentText(segmentsWithIndent), width), tone: line.tone }],
        width,
      );
      renderedLines.push({
        text: segmentText(renderedSegments),
        tone: line.tone,
        segments: renderedSegments,
      });
      continue;
    }

    const wrapped = line.noWrap ? [truncateText(line.text, usableWidth)] : wrapPlainText(line.text, usableWidth);

    for (const segment of wrapped) {
      renderedLines.push({
        text: `${indent}${segment}`,
        tone: line.tone,
      });
    }
  }

  return renderedLines;
}

export function renderRows(lines: DerivedTagTerminalLine[], width: number, height: number): RenderedTerminalLine[] {
  const rows = buildRenderedTerminalLines(lines, width);
  const rendered: RenderedTerminalLine[] = [];
  for (let index = 0; index < height; index += 1) {
    rendered.push(rows[index] ?? { text: "", tone: "default" });
  }
  return rendered;
}

export function terminalToneProps(tone: DerivedTagTerminalTone): React.ComponentProps<typeof Text> {
  switch (tone) {
    case "default":
      return {};
    case "heading":
      return { color: "cyan", bold: true };
    case "section":
      return { bold: true };
    case "dim":
      return { dimColor: true };
    case "accent":
      return { color: "cyan" };
    case "success":
      return { color: "green" };
    case "warning":
      return { color: "yellow" };
    case "danger":
      return { color: "red" };
    case "selected":
      return { inverse: true, bold: true };
  }
}

export function TerminalRows({
  lines,
  width,
}: {
  lines: RenderedTerminalLine[];
  width: number;
}): React.JSX.Element {
  return (
    <Box flexDirection="column" width={width}>
      {lines.map((line, index) => (
        <Text key={index} wrap="truncate-end" {...terminalToneProps(line.tone)}>
          {line.segments && line.segments.length > 0
            ? line.segments.map((segment, segmentIndex) => (
                <Text key={segmentIndex} {...terminalToneProps(segment.tone ?? "default")}>
                  {segment.text}
                </Text>
              ))
            : fitToWidth(line.text, width)}
        </Text>
      ))}
    </Box>
  );
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
  if (!footer || footer.length === 0) {
    return null;
  }

  return (
    <Box flexDirection="column" width={width}>
      {footer.map((line, index) => (
        <Text key={index} wrap="truncate-end" {...terminalToneProps(line.tone ?? "default")}>
          {line.segments && line.segments.length > 0
            ? line.segments.map((segment, segmentIndex) => (
                <Text key={segmentIndex} {...terminalToneProps(segment.tone ?? "default")}>
                  {segment.text}
                </Text>
              ))
            : fitToWidth(line.text, width)}
        </Text>
      ))}
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
  const bodyHeight = Math.max(0, height - 2);
  const rows = renderRows(pane.lines, width, bodyHeight);

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

export function getRenderedTerminalLineCount(lines: DerivedTagTerminalLine[], width: number): number {
  return buildRenderedTerminalLines(lines, width).length;
}

export function sliceRenderedTerminalLines(
  lines: DerivedTagTerminalLine[],
  width: number,
  start: number,
  count: number,
): DerivedTagTerminalLine[] {
  return buildRenderedTerminalLines(lines, width)
    .slice(start, start + count)
    .map((line) => ({
      text: line.text,
      tone: line.tone,
      noWrap: true,
    }));
}

export function TerminalTextScreen({
  title,
  subtitle,
  body,
  footer,
}: DerivedTagTerminalTextScreenProps): React.JSX.Element {
  const { width, height } = useDerivedTagTerminalSize();
  const headerHeight = subtitle ? 3 : 2;
  const footerHeight = footer?.length ?? 0;
  const bodyHeight = Math.max(0, height - headerHeight - footerHeight);
  const rows = renderRows(body, width, bodyHeight);

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

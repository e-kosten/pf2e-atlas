import React from "react";
import { Box, Text } from "ink";

import { useDerivedTagTerminalBackdropActive, useDerivedTagTerminalCapabilities } from "./context.js";
import type { DerivedTagTerminalLine, DerivedTagTerminalSegment, DerivedTagTerminalTone } from "./types.js";

export type RenderedTerminalLine = {
  text: string;
  tone: DerivedTagTerminalTone;
  segments?: DerivedTagTerminalSegment[];
};

export type RenderTerminalLineOptions = {
  hyperlinkSupport?: "supported" | "unsupported";
};

type NormalizedTerminalLine = {
  text: string;
  segments: DerivedTagTerminalSegment[];
  tone: DerivedTagTerminalTone;
  indent: number;
  href?: string;
  plainTextFallback?: string;
  noWrap: boolean;
};

function normalizeLine(line: DerivedTagTerminalLine): NormalizedTerminalLine {
  return {
    text: line.text,
    segments: line.segments ?? [],
    tone: line.tone ?? "default",
    indent: line.indent ?? 0,
    href: line.href,
    plainTextFallback: line.plainTextFallback,
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
      href: segment.href,
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

function buildHyperlinkFallbackText(text: string, href: string, plainTextFallback?: string): string {
  return plainTextFallback ?? `${text}: ${href}`;
}

function buildLineSegments(
  line: NormalizedTerminalLine,
  hyperlinkSupport: "supported" | "unsupported",
): DerivedTagTerminalSegment[] {
  if (line.segments.length === 0) {
    if (line.href && hyperlinkSupport === "unsupported") {
      return [
        {
          text: buildHyperlinkFallbackText(line.text, line.href, line.plainTextFallback),
          tone: line.tone,
        },
      ];
    }

    return [
      {
        text: line.text,
        tone: line.tone,
        href: line.href || undefined,
      },
    ];
  }

  return line.segments.map((segment) => {
    const href = segment.href ?? line.href ?? undefined;
    if (href && hyperlinkSupport === "unsupported") {
      return {
        text: buildHyperlinkFallbackText(segment.text, href),
        tone: segment.tone ?? line.tone,
      };
    }

    return {
      text: segment.text,
      tone: segment.tone ?? line.tone,
      href,
    };
  });
}

function buildIndentedSegments(
  segments: DerivedTagTerminalSegment[],
  indent: string,
  lineTone: DerivedTagTerminalTone,
): DerivedTagTerminalSegment[] {
  if (!indent) {
    return segments;
  }

  return [{ text: indent, tone: lineTone }, ...segments];
}

function buildRenderedLineFromSegments(
  segments: DerivedTagTerminalSegment[],
  fallbackTone: DerivedTagTerminalTone,
): RenderedTerminalLine {
  return {
    text: segmentText(segments),
    tone: fallbackTone,
    segments,
  };
}

function buildRenderedTerminalLines(
  lines: DerivedTagTerminalLine[],
  width: number,
  options: RenderTerminalLineOptions = {},
): RenderedTerminalLine[] {
  const renderedLines: RenderedTerminalLine[] = [];
  const hyperlinkSupport = options.hyperlinkSupport ?? "supported";

  for (const rawLine of lines) {
    const line = normalizeLine(rawLine);
    const indent = " ".repeat(Math.max(0, line.indent));
    const usableWidth = Math.max(1, width - indent.length);

    if (line.segments.length > 0) {
      const renderedSegments = truncateSegments(
        buildIndentedSegments(buildLineSegments(line, hyperlinkSupport), indent, "default"),
        width,
      );
      renderedLines.push(buildRenderedLineFromSegments(renderedSegments, line.tone));
      continue;
    }

    const visibleText =
      line.href && hyperlinkSupport === "unsupported"
        ? buildHyperlinkFallbackText(line.text, line.href, line.plainTextFallback)
        : line.text;
    const wrapped = line.noWrap ? [truncateText(visibleText, usableWidth)] : wrapPlainText(visibleText, usableWidth);

    for (const segment of wrapped) {
      if (line.href) {
        const href = hyperlinkSupport === "supported" ? line.href : undefined;
        renderedLines.push(
          buildRenderedLineFromSegments(
            buildIndentedSegments([{ text: segment, tone: line.tone, href }], indent, "default"),
            line.tone,
          ),
        );
        continue;
      }

      renderedLines.push({
        text: `${indent}${segment}`,
        tone: line.tone,
      });
    }
  }

  return renderedLines;
}

export function renderRows(
  lines: DerivedTagTerminalLine[],
  width: number,
  height: number,
  options: RenderTerminalLineOptions = {},
): RenderedTerminalLine[] {
  const rows = buildRenderedTerminalLines(lines, width, options);
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

function withBackdropToneProps(
  props: React.ComponentProps<typeof Text>,
  backdropActive: boolean,
): React.ComponentProps<typeof Text> {
  if (!backdropActive) {
    return props;
  }

  return {
    ...props,
    dimColor: true,
  };
}

const OSC_8_OPEN = "\u001b]8;;";
const OSC_8_TERMINATOR = "\u001b\\";
const OSC_8_CLOSE = `${OSC_8_OPEN}${OSC_8_TERMINATOR}`;

function formatHyperlinkText(
  text: string,
  href: string | undefined,
  hyperlinkSupport: "supported" | "unsupported",
): string {
  if (!href || hyperlinkSupport !== "supported" || text.length === 0) {
    return text;
  }

  return `${OSC_8_OPEN}${href}${OSC_8_TERMINATOR}${text}${OSC_8_CLOSE}`;
}

function terminalSegmentProps(
  tone: DerivedTagTerminalTone,
  href: string | undefined,
  hyperlinkSupport: "supported" | "unsupported",
  backdropActive: boolean,
): React.ComponentProps<typeof Text> {
  const toneProps = withBackdropToneProps(terminalToneProps(tone), backdropActive);
  if (!href || hyperlinkSupport !== "supported") {
    return toneProps;
  }

  return {
    ...toneProps,
    color: toneProps.color ?? "cyan",
    underline: true,
  };
}

function renderTerminalLineContent(
  line: Pick<RenderedTerminalLine, "segments" | "text" | "tone">,
  width: number,
  hyperlinkSupport: "supported" | "unsupported",
  backdropActive: boolean,
): React.ReactNode {
  if (line.segments && line.segments.length > 0) {
    return line.segments.map((segment, segmentIndex) => (
      <Text
        key={segmentIndex}
        {...terminalSegmentProps(segment.tone ?? line.tone, segment.href, hyperlinkSupport, backdropActive)}
      >
        {formatHyperlinkText(segment.text, segment.href, hyperlinkSupport)}
      </Text>
    ));
  }

  return (
    <Text {...terminalSegmentProps(line.tone, undefined, hyperlinkSupport, backdropActive)}>
      {fitToWidth(line.text, width)}
    </Text>
  );
}

function renderTerminalRow(
  line: RenderedTerminalLine,
  index: number,
  width: number,
  hyperlinkSupport: "supported" | "unsupported",
  backdropActive: boolean,
): React.JSX.Element {
  return (
    <Box key={index} width={width}>
      {renderTerminalLineContent(line, width, hyperlinkSupport, backdropActive)}
    </Box>
  );
}

export function TerminalRows({ lines, width }: { lines: RenderedTerminalLine[]; width: number }): React.JSX.Element {
  const { hyperlinkSupport } = useDerivedTagTerminalCapabilities();
  const backdropActive = useDerivedTagTerminalBackdropActive();

  return (
    <Box flexDirection="column" width={width}>
      {lines.map((line, index) => renderTerminalRow(line, index, width, hyperlinkSupport, backdropActive))}
    </Box>
  );
}

export function getRenderedTerminalLineCount(
  lines: DerivedTagTerminalLine[],
  width: number,
  options: RenderTerminalLineOptions = {},
): number {
  return buildRenderedTerminalLines(lines, width, options).length;
}

export function getRenderedTerminalLineStartRows(
  lines: DerivedTagTerminalLine[],
  width: number,
  options: RenderTerminalLineOptions = {},
): number[] {
  const startRows: number[] = [];
  let rowIndex = 0;

  for (const line of lines) {
    startRows.push(rowIndex);
    rowIndex += buildRenderedTerminalLines([line], width, options).length;
  }

  return startRows;
}

export function sliceRenderedTerminalLines(
  lines: DerivedTagTerminalLine[],
  width: number,
  start: number,
  count: number,
  options: RenderTerminalLineOptions = {},
): DerivedTagTerminalLine[] {
  return buildRenderedTerminalLines(lines, width, options)
    .slice(start, start + count)
    .map((line) => ({
      text: line.text,
      segments: line.segments,
      tone: line.tone,
      noWrap: true,
    }));
}

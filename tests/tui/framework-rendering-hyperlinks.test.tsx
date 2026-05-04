import { describe, expect, it } from "vitest";

import {
  getRenderedTerminalLineCount,
  renderRows,
  sliceRenderedTerminalLines,
} from "../../src/tui/framework/line-rendering.js";

describe("terminal hyperlink rendering helpers", () => {
  it("preserves segment hyperlinks when truncating rendered rows", () => {
    const [row] = renderRows(
      [
        {
          text: "",
          noWrap: true,
          segments: [
            { text: "Alpha", href: "https://example.com/alpha" },
            { text: "Beta", tone: "accent" },
          ],
        },
      ],
      7,
      1,
    );

    expect(row).toEqual({
      text: "AlphaBe",
      tone: "default",
      segments: [
        { text: "Alpha", tone: "default", href: "https://example.com/alpha" },
        { text: "Be", tone: "accent", href: undefined },
      ],
    });
  });

  it("wraps styled segments while preserving segment tones", () => {
    const rows = renderRows(
      [
        {
          text: "Alpha Beta Gamma Delta",
          indent: 2,
          segments: [
            { text: "Alpha " },
            { text: "Beta Gamma", tone: "accent" },
            { text: " Delta" },
          ],
        },
      ],
      14,
      3,
    );

    expect(rows.slice(0, 3)).toEqual([
      {
        text: "  Alpha Beta",
        tone: "default",
        segments: [
          { text: "  ", tone: "default" },
          { text: "Alpha", tone: "default", href: undefined },
          { text: " " },
          { text: "Beta", tone: "accent", href: undefined },
        ],
      },
      {
        text: "  Gamma Delta",
        tone: "default",
        segments: [
          { text: "  ", tone: "default" },
          { text: "Gamma", tone: "accent", href: undefined },
          { text: " " },
          { text: "Delta", tone: "default", href: undefined },
        ],
      },
      { text: "", tone: "default" },
    ]);
  });

  it("preserves line hyperlinks through rendered-line slicing", () => {
    const sliced = sliceRenderedTerminalLines(
      [{ text: "Alpha Beta Gamma", href: "https://example.com/browse" }],
      5,
      0,
      3,
      { hyperlinkSupport: "supported" },
    );

    expect(sliced).toEqual([
      {
        text: "Alpha",
        segments: [{ text: "Alpha", tone: "default", href: "https://example.com/browse" }],
        tone: "default",
        noWrap: true,
      },
      {
        text: "Beta",
        segments: [{ text: "Beta", tone: "default", href: "https://example.com/browse" }],
        tone: "default",
        noWrap: true,
      },
      {
        text: "Gamma",
        segments: [{ text: "Gamma", tone: "default", href: "https://example.com/browse" }],
        tone: "default",
        noWrap: true,
      },
    ]);
  });

  it("expands line hyperlinks into visible fallback text when support is unavailable", () => {
    const lines = [
      {
        text: "AoN",
        href: "https://example.com/browse",
        plainTextFallback: "AoN: https://example.com/browse",
      },
    ];

    expect(getRenderedTerminalLineCount(lines, 8, { hyperlinkSupport: "unsupported" })).toBeGreaterThan(
      getRenderedTerminalLineCount(lines, 8, { hyperlinkSupport: "supported" }),
    );

    const [row] = renderRows(lines, 40, 1, { hyperlinkSupport: "unsupported" });
    expect(row).toEqual({
      text: "AoN: https://example.com/browse",
      tone: "default",
      segments: [{ text: "AoN: https://example.com/browse", tone: "default", href: undefined }],
    });
  });
});

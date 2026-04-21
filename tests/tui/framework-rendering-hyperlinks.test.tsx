import { describe, expect, it } from "vitest";

import { getRenderedTerminalLineCount, renderRows, sliceRenderedTerminalLines } from "../../src/tui/framework/rendering.js";

describe("terminal hyperlink rendering helpers", () => {
  it("preserves segment hyperlinks when truncating rendered rows", () => {
    const [row] = renderRows(
      [
        {
          text: "",
          noWrap: true,
          segments: [
            { text: "Alpha", hyperlink: "https://example.com/alpha" },
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
        { text: "Alpha", tone: "default", hyperlink: "https://example.com/alpha" },
        { text: "Be", tone: "accent", hyperlink: undefined },
      ],
    });
  });

  it("preserves line hyperlinks through rendered-line slicing", () => {
    const sliced = sliceRenderedTerminalLines(
      [{ text: "Alpha Beta Gamma", hyperlink: "https://example.com/browse" }],
      5,
      0,
      3,
      { hyperlinkSupport: "supported" },
    );

    expect(sliced).toEqual([
      {
        text: "Alpha",
        segments: [{ text: "Alpha", tone: "default", hyperlink: "https://example.com/browse" }],
        tone: "default",
        noWrap: true,
      },
      {
        text: "Beta",
        segments: [{ text: "Beta", tone: "default", hyperlink: "https://example.com/browse" }],
        tone: "default",
        noWrap: true,
      },
      {
        text: "Gamma",
        segments: [{ text: "Gamma", tone: "default", hyperlink: "https://example.com/browse" }],
        tone: "default",
        noWrap: true,
      },
    ]);
  });

  it("expands line hyperlinks into visible fallback text when support is unavailable", () => {
    const lines = [
      {
        text: "AoN",
        hyperlink: "https://example.com/browse",
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
      segments: [{ text: "AoN: https://example.com/browse", tone: "default", hyperlink: undefined }],
    });
  });
});

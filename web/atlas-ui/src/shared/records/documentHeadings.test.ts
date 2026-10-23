import { describe, it, expect } from "vitest";
import { documentHeadings } from "./documentHeadings";

describe("journal document headings", () => {
  it("keeps source content and controls while making major sections subordinate to identity", () => {
    const original =
      '<h1>Overview</h1><p>Keep <a href="/exact">the exact reference</a>.</p><h2>Detail</h2><p data-atlas-control-id="roll">Roll text</p><h1>Training</h1><h1>Features</h1>';
    const first = documentHeadings(original, "journal/page");
    const parsed = new DOMParser().parseFromString(first.html, "text/html");
    expect([...parsed.querySelectorAll("h3")].map((h) => h.textContent)).toEqual([
      "Overview",
      "Training",
      "Features",
    ]);
    expect(parsed.querySelector("h4")?.textContent).toBe("Detail");
    expect(parsed.querySelector("a")?.getAttribute("href")).toBe("/exact");
    expect(parsed.querySelector('[data-atlas-control-id="roll"]')?.textContent).toBe(
      "Roll text",
    );
    expect(first.outline.map((section) => section.label)).toEqual([
      "Overview",
      "Training",
      "Features",
    ]);
    expect(documentHeadings(original, "journal/page")).toEqual(first);
    expect(documentHeadings(original, "other/page").outline[0].id).not.toBe(
      first.outline[0].id,
    );
  });
});

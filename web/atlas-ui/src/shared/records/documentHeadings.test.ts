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

it("preserves host section destinations and outline when inserted headings are added", () => {
  const host = "<h1>Overview</h1><h1>Training</h1><h1>Features</h1>";
  const composed =
    '<h1>Overview</h1><section data-atlas-inclusion="1"><h1>Training</h1></section><h1>Training</h1><h1>Features</h1>';
  const before = documentHeadings(host, "page");
  const after = documentHeadings(composed, "page");
  expect(after.outline).toEqual(before.outline);
  const parsed = new DOMParser().parseFromString(after.html, "text/html");
  expect(parsed.querySelector("[data-atlas-inclusion] h4")?.textContent).toBe(
    "Training",
  );
  const ids = [...parsed.querySelectorAll("[id]")].map((node) => node.id);
  expect(new Set(ids).size).toBe(ids.length);
});

it("keeps existing section anchors and outline after promoting a standalone action title", () => {
  const original =
    "<h1>Overview</h1><h1>Training</h1><h1>Features</h1><h2>Reactive Strike</h2>";
  const revised =
    original +
    '<h3 data-atlas-inclusion-heading="true"><span data-atlas-reference="1">Reactive Strike</span><span data-atlas-action-glyph="reaction">reaction</span></h3>';
  const before = documentHeadings(original, "fighter");
  const after = documentHeadings(revised, "fighter");
  expect(after.outline).toEqual(before.outline);
  const document = new DOMParser().parseFromString(after.html, "text/html");
  const beforeDocument = new DOMParser().parseFromString(before.html, "text/html");
  expect(document.querySelector("h4")?.id).toEqual(
    beforeDocument.querySelector("h4")?.id,
  );
  expect(document.querySelector("[data-atlas-inclusion-heading]")?.tagName).toBe("H5");
});

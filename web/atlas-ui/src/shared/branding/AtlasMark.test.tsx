import { render } from "@testing-library/react";
// @ts-expect-error Vitest runs in Node; the product build intentionally omits Node types.
import { readFileSync } from "node:fs";
import { AtlasMark } from "./AtlasMark";

const geometryAttributes = ["cx", "cy", "r", "d", "transform"];

function markGeometry(mark: Element) {
  return Array.from(mark.querySelectorAll("circle, path, g"), (element) => [
    element.localName,
    ...geometryAttributes.map((attribute) => element.getAttribute(attribute)),
  ]);
}

describe("Atlas brand mark", () => {
  it("renders the four-point compass as decorative header art", () => {
    const { container } = render(<AtlasMark />);
    const mark = container.querySelector("svg.atlas-mark");

    expect(mark).toHaveAttribute("aria-hidden", "true");
    const circles = mark?.querySelectorAll("circle");
    expect(circles).toHaveLength(1);
    expect(circles?.[0]).toHaveAttribute("fill", "none");
    expect(mark?.querySelectorAll(".atlas-mark__cardinal-points path")).toHaveLength(4);
    expect(mark?.querySelectorAll(".atlas-mark__die-face")).toHaveLength(10);
  });

  it.each(["light", "dark"])(
    "shares the header geometry with the %s favicon",
    (scheme) => {
      const { container } = render(<AtlasMark />);
      const mark = container.querySelector("svg")!;
      const source = readFileSync(`public/favicon-${scheme}.svg`, "utf8") as string;
      const favicon = new DOMParser().parseFromString(
        source,
        "image/svg+xml",
      ).documentElement;

      expect(favicon.querySelector("title")?.textContent).toBe("PF2e Atlas logo");
      expect(favicon.getAttribute("viewBox")).toBe(mark.getAttribute("viewBox"));
      expect(markGeometry(favicon)).toEqual(markGeometry(mark));
      const ring = favicon.querySelector("circle");
      expect(ring?.getAttribute("fill")).toBe("none");
      expect(ring?.getAttribute("stroke-width")).toBe(
        mark.querySelector("circle")?.getAttribute("stroke-width"),
      );
    },
  );
});

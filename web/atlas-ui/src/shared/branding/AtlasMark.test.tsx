import { render } from "@testing-library/react";
// @ts-expect-error Vitest runs in Node; the product build intentionally omits Node types.
import { readFileSync } from "node:fs";
import { AtlasMark } from "./AtlasMark";

describe("Atlas brand mark", () => {
  it("renders the full tucked-under compass treatment as decorative header art", () => {
    const { container } = render(<AtlasMark />);
    const mark = container.querySelector("svg.atlas-mark");

    expect(mark).toHaveAttribute("viewBox", "0 0 160 160");
    expect(mark).toHaveAttribute("aria-hidden", "true");
    expect(mark?.querySelectorAll(".atlas-mark__secondary-points path")).toHaveLength(
      4,
    );
    expect(mark?.querySelectorAll(".atlas-mark__die-face")).toHaveLength(10);
  });

  it.each(["light", "dark"])(
    "keeps the %s favicon small-size asset correctly named",
    (scheme) => {
      const source = readFileSync(`public/favicon-${scheme}.svg`, "utf8") as string;

      expect(source).toContain('<title id="title">PF2e Atlas logo</title>');
      expect(source).not.toContain("Compass over die faces");
      expect(source).toContain('viewBox="0 0 160 160"');
      expect(source).toMatch(
        /<circle cx="80" cy="80" r="54" fill="none" stroke="#[0-9A-F]+" stroke-width="8"\/>/,
      );
      expect(source).not.toContain('r="50"');
    },
  );
});

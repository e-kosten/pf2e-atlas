import { render, screen } from "@testing-library/react";
import type { CreatureSurfaceActionCostView } from "../../generated/atlas";
import { ActionGlyph } from "./ActionGlyph";

describe("ActionGlyph", () => {
  it.each([
    [{ cost_type: "actions", count: 1 }, "1", "One action"],
    [{ cost_type: "actions", count: 2 }, "2", "Two actions"],
    [{ cost_type: "actions", count: 3 }, "3", "Three actions"],
    [{ cost_type: "free_action" }, "F", "Free action"],
    [{ cost_type: "reaction" }, "R", "Reaction"],
  ] as const)(
    "maps %o to the pinned glyph and accessible text",
    (cost, glyph, label) => {
      const { container } = render(
        <ActionGlyph cost={cost as CreatureSurfaceActionCostView} />,
      );

      expect(screen.getByText(label)).toHaveClass("sr-only");
      expect(container.querySelector(".action-glyph__mark")).toHaveTextContent(glyph);
      expect(container.querySelector(".action-glyph__mark")).toHaveAttribute(
        "aria-hidden",
        "true",
      );
      expect(container.querySelector("[tabindex]")).not.toBeInTheDocument();
      expect(container.querySelector("button, a")).not.toBeInTheDocument();
    },
  );

  it.each([
    [{ cost_type: "actions", count: 4 }, "4 actions"],
    [{ cost_type: "passive" }, "Passive"],
    [{ cost_type: "time", value: "1 minute" }, "1 minute"],
    [{ cost_type: "future" }, "The action cost is not recognized."],
    [undefined, "The action cost is not listed."],
  ] as const)("keeps %o as a readable text fallback", (cost, label) => {
    const { container } = render(<ActionGlyph cost={cost} />);

    expect(screen.getByText(label)).toBeVisible();
    expect(screen.getByText(label)).toHaveClass("action-glyph__label");
    expect(container.querySelector(".action-glyph__mark")).not.toBeInTheDocument();
  });

  it.each([
    [{ kind: "actions", count: 2 }, "2", "Two actions"],
    [{ kind: "free_action" }, "F", "Free action"],
    [{ kind: "reaction" }, "R", "Reaction"],
  ] as const)(
    "uses the same glyph presentation for runtime cost %o",
    (cost, glyph, label) => {
      const { container } = render(<ActionGlyph cost={cost} />);

      expect(screen.getByText(label)).toHaveClass("sr-only");
      expect(container.querySelector(".action-glyph__mark")).toHaveTextContent(glyph);
    },
  );
});

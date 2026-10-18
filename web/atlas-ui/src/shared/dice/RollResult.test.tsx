import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";
import type { RollOutcome } from "../../generated/atlas";
import { RollResult } from "./RollResult";
import { DieFace } from "./DieFace";

it("omits decorative parentheses but retains arithmetic grouping", () => {
  const sum: RollOutcome["expression"] = {
    kind: "binary",
    operator: "add",
    total: 7,
    left: { kind: "dice", dice: [{ sides: 6, value: 3, kept: true }], total: 3 },
    right: { kind: "number", value: 4 },
  };
  const { rerender } = render(
    <RollResult
      outcome={{ formula: "1d6+4", total: 7, expression: sum }}
      motion="none"
    />,
  );
  expect(screen.getByRole("status").textContent).not.toMatch(/[()]/);
  rerender(
    <RollResult
      outcome={{
        formula: "(1d6+4)*2",
        total: 14,
        expression: {
          kind: "binary",
          operator: "multiply",
          total: 14,
          left: sum,
          right: { kind: "number", value: 2 },
        },
      }}
      motion="none"
    />,
  );
  expect(screen.getByRole("status").textContent).toContain("(3+4)×2");
});

it("uses the shaded d10 fallback without changing the actual die or outcome", () => {
  const { container, rerender } = render(
    <DieFace sides={78} value={73} motion="none" />,
  );
  const shape = container.querySelector("polygon")?.getAttribute("points");
  expect(shape).toBeTruthy();
  expect(container.querySelector(".atlas-die")).toHaveAccessibleName("d78: 73");
  expect(screen.getByText("d78")).toBeVisible();
  expect(screen.getByText("73")).toBeVisible();
  rerender(<DieFace sides={10} value={7} motion="none" />);
  expect(container.querySelector("polygon")?.getAttribute("points")).toBe(shape);
  expect(container.querySelector(".atlas-die__sides")).toBeNull();
});

it("preserves draw order and only reveals discarded state after settling", () => {
  const outcome: RollOutcome = {
    formula: "2d20kh1",
    total: 17,
    expression: {
      kind: "dice",
      total: 17,
      dice: [
        { sides: 20, value: 3, kept: false },
        { sides: 20, value: 17, kept: true },
      ],
    },
  };
  const { container, rerender } = render(
    <RollResult outcome={outcome} pending motion="none" />,
  );
  expect(screen.getByRole("status")).toHaveAccessibleName("Rolling");
  expect(container.querySelectorAll(".atlas-die--discarded")).toHaveLength(0);
  expect(container.querySelector(".atlas-roll__total")).not.toHaveTextContent("17");
  rerender(<RollResult outcome={outcome} motion="none" />);
  const dice = container.querySelectorAll(".atlas-die");
  expect(dice[0]).toHaveAccessibleName("d20: 3, discarded");
  expect(dice[0]).toHaveClass("atlas-die--discarded");
  expect(dice[1]).toHaveAccessibleName("d20: 17");
  expect(dice[1]).not.toHaveClass("atlas-die--discarded");
  expect(screen.getByRole("status")).toHaveAccessibleName("2d20kh1: 17");
});

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterTurnResources } from "./EncounterTurnResources";
import type { TurnResourcesView } from "../../generated/atlas";
function value(): TurnResourcesView {
  return {
    turn_key: "turn:a",
    round_number: 1n,
    regular_total: 1n,
    regular_remaining: 1n,
    extra_total: 0n,
    extra_remaining: 0n,
    extra_uses: [],
    reactions_total: 1n,
    reactions_remaining: 1n,
    start_results: ["Stunned 2 cleared · 2 actions lost this turn"],
    pending_reason: null,
    forecast: {
      label: "Actions",
      base_value: 3n,
      adjusted_value: 3n,
      segments: [],
      adjustments: [],
      suppressed_adjustments: [],
    },
    can_edit_actions: true,
    can_spend_actions: { available: true },
    can_spend_reactions: { available: true },
  };
}
it("spends only on explicit input and retains the recorded allowance and start result across updates", async () => {
  const onChange = vi.fn().mockResolvedValue(undefined),
    initial = value();
  const { rerender } = render(
    <EncounterTurnResources
      value={initial}
      disabled={false}
      isCurrent
      onChange={onChange}
    />,
  );
  expect(onChange).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Spend action 1" }));
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith({ kind: "spend", resource: "action" }),
  );
  rerender(
    <EncounterTurnResources
      value={{ ...initial, regular_remaining: 0n }}
      disabled={false}
      isCurrent
      onChange={onChange}
    />,
  );
  expect(
    screen.getByRole("button", { name: /^Edit remaining actions:/ }),
  ).toHaveTextContent("0 / 1");
  expect(screen.getByText(initial.start_results[0])).toBeVisible();
  expect(onChange).toHaveBeenCalledTimes(1);
});
it("keeps an off-turn reaction usable and distinguishes spent stock from a condition restriction", async () => {
  const onChange = vi.fn().mockResolvedValue(undefined),
    initial = value();
  const { rerender } = render(
    <EncounterTurnResources
      value={initial}
      disabled={false}
      isCurrent={false}
      onChange={onChange}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Spend reaction" }));
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith({ kind: "spend", resource: "reaction" }),
  );
  rerender(
    <EncounterTurnResources
      value={{
        ...initial,
        reactions_remaining: 0n,
        can_spend_reactions: { available: false, reason: "Stunned." },
      }}
      disabled={false}
      isCurrent={false}
      onChange={onChange}
    />,
  );
  expect(screen.getByRole("button", { name: "Restore reaction" })).toHaveTextContent(
    "Spent",
  );
  expect(screen.getByText("Cannot react · Stunned")).toBeVisible();
});
it("shows mid-turn Quickened as a forecast rather than another current action", () => {
  const current = value();
  current.regular_total = 3n;
  current.regular_remaining = 2n;
  current.forecast.adjusted_value = 4n;
  render(
    <EncounterTurnResources
      value={current}
      disabled={false}
      isCurrent
      onChange={vi.fn()}
    />,
  );
  expect(
    screen.getByRole("button", { name: /^Edit remaining actions:/ }),
  ).toHaveTextContent("2 / 3");
  expect(screen.getByText("Next turn")).toBeVisible();
  expect(screen.queryByRole("button", { name: "Spend extra action 1" })).toBeNull();
});

import { fireEvent, render, screen } from "@testing-library/react";
import { SpellRankControl } from "./SpellRankControl";
import { numberFact, spellFixture } from "../../test/fixtures";

it.each([false, true])(
  "returns to the correct opening rank and retains the trigger and focus (embedded=%s)",
  async (embedded) => {
    const onChange = vi.fn();
    const spell = spellFixture();
    const view = (rank: number) => (
      <SpellRankControl
        spell={{ ...spell, rank: numberFact(rank) }}
        embedded={embedded}
        onChange={onChange}
      />
    );
    const { rerender } = render(view(3));
    const trigger = screen.getByRole("button", { name: "Change rank" });
    rerender(view(4));
    fireEvent.click(trigger);
    const reset = await screen.findByRole("menuitem", {
      name: embedded ? "Return to opening rank 3" : "Rank 1 (base)",
    });
    fireEvent.click(reset);
    const openingRank = embedded ? 3 : 1;
    expect(onChange).toHaveBeenCalledWith(openingRank);
    rerender(view(openingRank));
    expect(screen.getByRole("button", { name: "Change rank" })).toBe(trigger);
    expect(trigger).toHaveFocus();
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("Base rank 1")).toBeNull();
  },
);

it("opens from ArrowDown without its default scrolling behavior and selects an explicit rank", async () => {
  const onChange = vi.fn();
  render(
    <SpellRankControl spell={spellFixture()} embedded={false} onChange={onChange} />,
  );
  const trigger = screen.getByRole("button", { name: "Change rank" });
  trigger.focus();
  expect(fireEvent.keyDown(trigger, { key: "ArrowDown" })).toBe(false);
  fireEvent.click(await screen.findByRole("menuitem", { name: "Rank 5" }));
  expect(onChange).toHaveBeenCalledExactlyOnceWith(5);
  expect(trigger).toHaveFocus();
});

it("closes its menu on Escape and restores the initiating control without selecting a rank", async () => {
  const onChange = vi.fn();
  render(
    <SpellRankControl spell={spellFixture()} embedded={false} onChange={onChange} />,
  );
  const trigger = screen.getByRole("button", { name: "Change rank" });
  fireEvent.click(trigger);
  const item = await screen.findByRole("menuitem", { name: "Rank 5" });
  item.focus();
  fireEvent.keyDown(item, { key: "Escape" });
  expect(trigger).toHaveAttribute("aria-expanded", "false");
  expect(trigger).toHaveFocus();
  expect(onChange).not.toHaveBeenCalled();
});

import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
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
        classification={`Spell ${rank}`}
        embedded={embedded}
        onChange={onChange}
      />
    );
    const { rerender } = render(view(3));
    const trigger = screen.getByRole("button", { name: /Change rank/ });
    rerender(view(4));
    fireEvent.click(trigger);
    const reset = await screen.findByRole("menuitem", {
      name: embedded ? "Return to opening rank 3" : "Rank 1 (base)",
    });
    fireEvent.click(reset);
    const openingRank = embedded ? 3 : 1;
    expect(onChange).toHaveBeenCalledWith(openingRank);
    rerender(view(openingRank));
    expect(screen.getByRole("button", { name: /Change rank/ })).toBe(trigger);
    expect(trigger).toHaveFocus();
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("Base rank 1")).toBeNull();
  },
);

it("opens from ArrowDown without its default scrolling behavior and selects an explicit rank", async () => {
  const onChange = vi.fn();
  render(
    <SpellRankControl
      classification="Spell 1"
      spell={spellFixture()}
      embedded={false}
      onChange={onChange}
    />,
  );
  const trigger = screen.getByRole("button", { name: /Change rank/ });
  trigger.focus();
  expect(fireEvent.keyDown(trigger, { key: "ArrowDown" })).toBe(false);
  await waitFor(() =>
    expect(screen.getByRole("menuitem", { name: "Rank 1 (base)" })).toHaveFocus(),
  );
  fireEvent.click(await screen.findByRole("menuitem", { name: "Rank 5" }));
  expect(onChange).toHaveBeenCalledExactlyOnceWith(5);
  expect(trigger).toHaveFocus();
});

it.each([
  ["Tab", "Following action"],
  ["Shift+Tab", "Previous action"],
])("closes on %s and continues from the trigger", async (key, destination) => {
  const onChange = vi.fn();
  render(
    <div role="dialog" aria-label="Reference">
      <button>Previous action</button>
      <SpellRankControl
        classification="Spell 1"
        spell={spellFixture()}
        embedded={false}
        onChange={onChange}
      />
      <button>Following action</button>
    </div>,
  );
  const trigger = screen.getByRole("button", { name: /Change rank/ });
  fireEvent.click(trigger);
  const item = await screen.findByRole("menuitem", { name: "Rank 1 (base)" });
  expect(
    fireEvent.keyDown(item, {
      key: "Tab",
      shiftKey: key === "Shift+Tab",
    }),
  ).toBe(false);
  expect(trigger).toHaveAttribute("aria-expanded", "false");
  await waitFor(() =>
    expect(screen.getByRole("button", { name: destination })).toHaveFocus(),
  );
  expect(onChange).not.toHaveBeenCalled();
});

it("closes its menu on Escape and restores the initiating control without selecting a rank", async () => {
  const onChange = vi.fn();
  render(
    <SpellRankControl
      classification="Spell 1"
      spell={spellFixture()}
      embedded={false}
      onChange={onChange}
    />,
  );
  const trigger = screen.getByRole("button", { name: /Change rank/ });
  fireEvent.click(trigger);
  const item = await screen.findByRole("menuitem", { name: "Rank 5" });
  item.focus();
  fireEvent.keyDown(item, { key: "Escape" });
  expect(trigger).toHaveAttribute("aria-expanded", "false");
  expect(trigger).toHaveFocus();
  expect(onChange).not.toHaveBeenCalled();
});

it("keeps focus on the trigger after a quick selection and the opening animation frames", async () => {
  vi.useFakeTimers();
  try {
    render(
      <SpellRankControl
        classification="Spell 1"
        spell={spellFixture()}
        embedded={false}
        onChange={vi.fn()}
      />,
    );
    const trigger = screen.getByRole("button", { name: /Change rank/ });
    fireEvent.click(trigger);
    fireEvent.click(screen.getByRole("menuitem", { name: "Rank 5" }));
    await act(() => vi.advanceTimersByTimeAsync(250));
    expect(trigger).toHaveFocus();
  } finally {
    vi.useRealTimers();
  }
});

it("respects focus deliberately moved before the menu's opening frame", async () => {
  vi.useFakeTimers();
  try {
    render(
      <>
        <SpellRankControl
          classification="Spell 1"
          spell={spellFixture()}
          embedded={false}
          onChange={vi.fn()}
        />
        <button>Another action</button>
      </>,
    );
    fireEvent.click(screen.getByRole("button", { name: /Change rank/ }));
    const destination = screen.getByRole("button", { name: "Another action" });
    destination.focus();
    await act(() => vi.advanceTimersByTimeAsync(250));
    expect(destination).toHaveFocus();
  } finally {
    vi.useRealTimers();
  }
});

it("consumes Enter activation when selection returns focus to the button", async () => {
  const onChange = vi.fn();
  render(
    <SpellRankControl
      classification="Spell 1"
      spell={spellFixture()}
      embedded={false}
      onChange={onChange}
    />,
  );
  const trigger = screen.getByRole("button", { name: /Change rank/ });
  fireEvent.click(trigger);
  const item = await screen.findByRole("menuitem", { name: "Rank 5" });
  expect(fireEvent.keyDown(item, { key: "Enter", keyCode: 13 })).toBe(false);
  expect(onChange).toHaveBeenCalledExactlyOnceWith(5);
  expect(trigger).toHaveAttribute("aria-expanded", "false");
  expect(trigger).toHaveFocus();
});

it("keeps selecting rank zero idle and permits the inclusive rank-ten boundary", async () => {
  const onChange = vi.fn();
  render(
    <SpellRankControl
      classification="Spell 0"
      spell={{ ...spellFixture(), base_rank: numberFact(0), rank: numberFact(0) }}
      embedded={false}
      onChange={onChange}
    />,
  );
  const trigger = screen.getByRole("button", { name: /Change rank/ });
  fireEvent.click(trigger);
  fireEvent.click(await screen.findByRole("menuitem", { name: "Rank 0 (base)" }));
  expect(onChange).not.toHaveBeenCalled();
  expect(trigger).toHaveFocus();
  fireEvent.click(trigger);
  fireEvent.click(await screen.findByRole("menuitem", { name: "Rank 10" }));
  expect(onChange).toHaveBeenCalledExactlyOnceWith(10);
  expect(trigger).toHaveFocus();
});

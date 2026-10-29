import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { EncounterAbilityControls } from "./EncounterAbilityControls";
import { EncounterAbilityStates } from "./EncounterAbilityStates";
import type { EncounterAbilityStateView } from "../../generated/atlas";
const ability: EncounterAbilityStateView = {
  navigation: {
    record_key: "actors:abcdefghijklmnop",
    embedded_path: [{ collection: "/items", identity: { Stable: "ability000000000" } }],
    field: null,
    passage: null,
    source_fingerprint: "source",
  },
  label: "Speed Surge",
  rule_ordinal: null,
  state: { kind: "uses", remaining: 1n, maximum: 3n, interval: "day" },
};
it("spends explicitly and retains the control through a canonical update", async () => {
  const onChange = vi.fn().mockResolvedValue(true);
  const { rerender } = render(
    <EncounterAbilityControls ability={ability} disabled={false} onChange={onChange} />,
  );
  const use = screen.getByRole("button", { name: "Use Speed Surge" });
  expect(onChange).not.toHaveBeenCalled();
  use.focus();
  fireEvent.click(use);
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith(ability, { kind: "spend_use" }),
  );
  rerender(
    <EncounterAbilityControls
      ability={{
        ...ability,
        state: { kind: "uses", remaining: 0n, maximum: 3n, interval: "day" },
      }}
      disabled={false}
      onChange={onChange}
    />,
  );
  expect(screen.getByRole("button", { name: "Use Speed Surge" })).toBe(use);
  await waitFor(() => expect(use).toBeDisabled());
  expect(screen.getByText("0 of 3 · per day")).toBeVisible();
  expect(
    screen.getByRole("button", { name: "Restore use of Speed Surge" }),
  ).toBeEnabled();
});
it("keeps a failed update local and allows the same action to be retried", async () => {
  const onChange = vi
    .fn()
    .mockRejectedValueOnce(new Error("Encounter changed"))
    .mockResolvedValue(true);
  render(
    <EncounterAbilityControls ability={ability} disabled={false} onChange={onChange} />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Use Speed Surge" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Could not update ability.",
  );
  fireEvent.click(screen.getByRole("button", { name: "Use Speed Surge" }));
  await waitFor(() => expect(onChange).toHaveBeenCalledTimes(2));
  expect(onChange.mock.calls).toEqual([
    [ability, { kind: "spend_use" }],
    [ability, { kind: "spend_use" }],
  ]);
  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
});
it("changes the authored switch without losing its selected option", async () => {
  const onChange = vi.fn().mockResolvedValue(true);
  const option: EncounterAbilityStateView = {
    ...ability,
    label: "Mounted",
    rule_ordinal: 0,
    state: {
      kind: "option",
      active: true,
      always_active: false,
      selection: null,
      choices: [],
    },
  };
  render(
    <EncounterAbilityControls ability={option} disabled={false} onChange={onChange} />,
  );
  const control = screen.getByRole("switch", { name: "Mounted active" });
  expect(screen.queryByText("Mounted")).toBeNull();
  control.focus();
  fireEvent.click(control);
  await waitFor(() => expect(control).toHaveFocus());
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith(option, {
      kind: "set_option",
      active: false,
      selection: null,
    }),
  );
});

it("selects an authored choice directly from Off in one update", async () => {
  const onChange = vi.fn().mockResolvedValue(true);
  const option = {
    ...ability,
    label: "Go Dark",
    rule_ordinal: 0,
    state: {
      kind: "option" as const,
      active: false,
      always_active: false,
      selection: "Compendium.pf2e.conditionitems.Item.Invisible",
      choices: [
        {
          value: "Compendium.pf2e.conditionitems.Item.Invisible",
          label: "Invisible",
          consequence: null,
        },
        {
          value: "Compendium.pf2e.conditionitems.Item.Hidden",
          label: "Hidden",
          consequence: null,
        },
      ],
    },
  };
  const { rerender } = render(
    <EncounterAbilityControls ability={option} disabled={false} onChange={onChange} />,
  );
  expect(screen.queryByRole("switch")).toBeNull();
  const trigger = screen.getByRole("button", { name: "Go Dark effect: Off" });
  expect(screen.queryByText("Go Dark")).toBeNull();
  trigger.focus();
  fireEvent.click(trigger);
  fireEvent.click(await screen.findByRole("menuitem", { name: "Hidden" }));
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledExactlyOnceWith(option, {
      kind: "set_option",
      active: true,
      selection: "Compendium.pf2e.conditionitems.Item.Hidden",
    }),
  );
  rerender(
    <EncounterAbilityControls
      ability={{
        ...option,
        state: {
          ...option.state,
          active: true,
          selection: "Compendium.pf2e.conditionitems.Item.Hidden",
        },
      }}
      disabled={false}
      onChange={onChange}
    />,
  );
  const selected = screen.getByRole("button", { name: "Go Dark effect: Hidden" });
  expect(selected).toBe(trigger);
  await waitFor(() => expect(selected).toHaveFocus());
  expect(selected).toHaveAttribute("aria-expanded", "false");
  fireEvent.click(selected);
  const menuChoice = await screen.findByRole("menuitem", { name: "Hidden" });
  menuChoice.focus();
  fireEvent.keyDown(menuChoice, { key: "Escape", keyCode: 27 });
  await waitFor(() => expect(selected).toHaveAttribute("aria-expanded", "false"));
  await waitFor(() => expect(selected).toHaveFocus());
  expect(onChange).toHaveBeenCalledTimes(1);
  vi.useFakeTimers();
  try {
    fireEvent.keyDown(selected, { key: "ArrowDown", keyCode: 40 });
    fireEvent.keyDown(selected, { key: "Escape", keyCode: 27 });
    await act(() => vi.advanceTimersByTimeAsync(250));
    expect(selected).toHaveAttribute("aria-expanded", "false");
    expect(selected).toHaveFocus();
    expect(onChange).toHaveBeenCalledTimes(1);
  } finally {
    vi.useRealTimers();
  }
});

it("identifies each inline switch and sends its exact authored ordinal", async () => {
  const onChange = vi.fn().mockResolvedValue(true);
  const stars: EncounterAbilityStateView[] = ["Black", "Gold", "Green", "White"].map(
    (color, ordinal) => ({
      ...ability,
      label: `${color} Aeon Star used`,
      rule_ordinal: ordinal,
      state: {
        kind: "option",
        active: false,
        always_active: false,
        selection: null,
        choices: [],
      },
    }),
  );
  render(
    <section aria-label="Inline abilities">
      {stars.map((star) => (
        <EncounterAbilityControls
          key={star.rule_ordinal}
          ability={star}
          showLabel
          disabled={false}
          onChange={onChange}
        />
      ))}
    </section>,
  );
  const inline = within(screen.getByRole("region", { name: "Inline abilities" }));
  for (const star of stars) {
    expect(inline.getByText(star.label)).toBeVisible();
    fireEvent.click(inline.getByRole("switch", { name: `${star.label} active` }));
    await waitFor(() =>
      expect(onChange).toHaveBeenCalledWith(star, {
        kind: "set_option",
        active: true,
        selection: null,
      }),
    );
  }
  expect(onChange).toHaveBeenCalledTimes(4);
});

it("omits the ability states section when no configurable ability is supplied", () => {
  const { rerender } = render(
    <EncounterAbilityStates
      participantKey="participant"
      disabled={false}
      onReference={vi.fn()}
    />,
  );
  expect(screen.queryByRole("region", { name: "Ability states" })).toBeNull();
  rerender(
    <EncounterAbilityStates
      participantKey="participant"
      abilities={[ability]}
      disabled={false}
      onReference={vi.fn()}
    />,
  );
  expect(screen.queryByRole("region", { name: "Ability states" })).toBeNull();
});

it("keeps a state available and synchronized with its ability after a canonical update", async () => {
  const onChange = vi.fn().mockResolvedValue(true);
  const mounted = {
    ...ability,
    label: "Mounted",
    rule_ordinal: 0,
    state: {
      kind: "option" as const,
      active: true,
      always_active: false,
      selection: null,
      choices: [],
    },
  };
  const view = (current: EncounterAbilityStateView) => (
    <>
      <EncounterAbilityStates
        participantKey="participant"
        abilities={[current]}
        disabled={false}
        onChange={onChange}
        onReference={vi.fn()}
      />
      <section aria-label="Ability description">
        <EncounterAbilityControls
          ability={current}
          disabled={false}
          onChange={onChange}
        />
      </section>
    </>
  );
  const { rerender } = render(view(mounted));
  const states = within(screen.getByRole("region", { name: "Ability states" }));
  const control = states.getByRole("switch", { name: "Mounted active" });
  fireEvent.click(control);
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledExactlyOnceWith(mounted, {
      kind: "set_option",
      active: false,
      selection: null,
    }),
  );
  rerender(view({ ...mounted, state: { ...mounted.state, active: false } }));
  expect(states.getByRole("switch", { name: "Mounted active" })).toBe(control);
  expect(control).not.toBeChecked();
  expect(
    within(screen.getByRole("region", { name: "Ability description" })).getByRole(
      "switch",
      { name: "Mounted active" },
    ),
  ).not.toBeChecked();
});

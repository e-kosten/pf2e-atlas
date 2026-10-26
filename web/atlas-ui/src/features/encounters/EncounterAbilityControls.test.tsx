import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterAbilityControls } from "./EncounterAbilityControls";
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
  expect(screen.getByText("0 / 3 uses · day")).toBeVisible();
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
  expect(await screen.findByRole("alert")).toHaveTextContent("Encounter changed");
  fireEvent.click(screen.getByRole("button", { name: "Use Speed Surge" }));
  await waitFor(() => expect(onChange).toHaveBeenCalledTimes(2));
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
  const option: EncounterAbilityStateView = {
    ...ability,
    label: "Go Dark",
    rule_ordinal: 0,
    state: {
      kind: "option",
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
  render(
    <EncounterAbilityControls ability={option} disabled={false} onChange={onChange} />,
  );
  expect(screen.queryByRole("switch")).toBeNull();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Go Dark option" }));
  fireEvent.click(await screen.findByRole("option", { name: "Hidden" }));
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledExactlyOnceWith(option, {
      kind: "set_option",
      active: true,
      selection: "Compendium.pf2e.conditionitems.Item.Hidden",
    }),
  );
});

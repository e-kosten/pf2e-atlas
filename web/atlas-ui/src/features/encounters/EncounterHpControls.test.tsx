import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { encounterParticipant } from "./encounterTestFixtures";
import { EncounterHpControls } from "./EncounterHpControls";
import type { EncounterParticipantView } from "../../generated/atlas";

it.each([
  ["Current HP", "set_current"],
  ["Temporary HP", "set_temporary"],
])("submits inline %s arithmetic once on Enter", async (label, kind) => {
  const onAction = vi.fn().mockResolvedValue(true);
  render(
    <EncounterHpControls
      current={encounterParticipant({ temporary_hp: 300n })}
      onAction={onAction}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: `Edit ${label}` }));
  const input = screen.getByRole("textbox", { name: label });
  fireEvent.change(input, { target: { value: "300-10" } });
  fireEvent.keyDown(input, { key: "Enter" });
  await waitFor(() =>
    expect(onAction).toHaveBeenCalledExactlyOnceWith({ kind, input: "300-10" }),
  );
  await waitFor(() =>
    expect(screen.queryByRole("textbox", { name: label })).not.toBeInTheDocument(),
  );
});

it("submits a blank temporary HP correction on Enter", async () => {
  const current = encounterParticipant({ temporary_hp: 5n });
  const onAction = vi.fn().mockImplementation(async () => {
    rerender(
      <EncounterHpControls
        current={{ ...current, temporary_hp: 0n }}
        onAction={onAction}
      />,
    );
    return true;
  });
  const { rerender } = render(
    <EncounterHpControls current={current} onAction={onAction} />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Edit Temporary HP" }));
  const input = screen.getByRole("textbox", { name: "Temporary HP" });
  fireEvent.change(input, { target: { value: "" } });
  fireEvent.keyDown(input, { key: "Enter" });
  await waitFor(() =>
    expect(onAction).toHaveBeenCalledExactlyOnceWith({
      kind: "set_temporary",
      input: "",
    }),
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add temp HP" })).toHaveFocus(),
  );
});

it("retains a rejected inline formula for correction and retry", async () => {
  const onAction = vi
    .fn()
    .mockRejectedValueOnce(new Error("Invalid HP formula"))
    .mockResolvedValue(true);
  render(
    <EncounterHpControls
      current={encounterParticipant({ temporary_hp: 300n })}
      onAction={onAction}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Edit Temporary HP" }));
  const input = screen.getByRole("textbox", { name: "Temporary HP" });
  fireEvent.change(input, { target: { value: "300-" } });
  fireEvent.keyDown(input, { key: "Enter" });
  await waitFor(() => expect(onAction).toHaveBeenCalledOnce());
  await waitFor(() => expect(input).not.toHaveAttribute("readonly"));
  expect(input).toHaveValue("300-");
  fireEvent.change(input, { target: { value: "300-10" } });
  fireEvent.keyDown(input, { key: "Enter" });
  await waitFor(() => expect(onAction).toHaveBeenCalledTimes(2));
  expect(onAction).toHaveBeenLastCalledWith({
    kind: "set_temporary",
    input: "300-10",
  });
});

it("shows updated derived HP after a submitted edit and variant capacity change", async () => {
  const current: EncounterParticipantView = {
    revision: 0n,
    knockout_options: [],
    effective_max_hp: { state: "value", value: 20, adjustment: null },
    persistent_damage_groups: [],
    track_dying: false,
    mortality: "alive",
    base_death_threshold: 4n,
    participant_key: "ghoul",
    position: 1n,
    initiative_order: 1n,
    display_name: "Ghoul",
    hp_origin: "derived",
    variant_origin: "default_unadjusted",
    participant_kind: "creature",
    side: "enemy",
    participant_variant: "normal",
    record_key: "pathfinder-bestiary:LHHgGSs0ELCR4CYK",
    status: "active",
    max_hp: 20n,
    current_hp: 20n,
    temporary_hp: 0n,
    defeated: false,
    hazard_disabled: false,
    cast_unavailable_reason: null,
    hidden: false,
    note_hint: "",
    conditions: [],
  };
  const onAction = vi.fn().mockResolvedValue(true);
  const { rerender } = render(
    <EncounterHpControls current={current} onAction={onAction} />,
    {
      wrapper: ({ children }) => (
        <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
      ),
    },
  );
  fireEvent.click(screen.getByRole("button", { name: "Edit HP" }));
  fireEvent.change(screen.getByRole("textbox", { name: "Current HP" }), {
    target: { value: "13" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenCalledWith({
      kind: "set_values",
      current: { input: "13" },
    }),
  );
  await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  rerender(
    <EncounterHpControls
      current={{
        ...current,
        current_hp: 23n,
        max_hp: 30n,
        effective_max_hp: { state: "value", value: 30, adjustment: null },
        participant_variant: "elite",
        variant_origin: "explicit",
      }}
      onAction={onAction}
    />,
  );
  expect(screen.getByLabelText("HP summary")).toHaveTextContent("23/30");
  fireEvent.click(screen.getByRole("button", { name: "Edit HP" }));
  expect(screen.getByRole("textbox", { name: "Current HP" })).toHaveValue("23");
});

it("sets current and temporary HP directly, with Escape cancelling and no damage or healing event", async () => {
  const current = encounterParticipant({
    current_hp: 12n,
    max_hp: 30n,
    effective_max_hp: {
      state: "value",
      value: 20,
      adjustment: {
        authored: 30,
        applied: [
          { source: "drained", label: "Drained", value: -10n, modifier_type: "status" },
        ],
        suppressed: [],
        notes: [],
      },
    },
  });
  const onAction = vi.fn().mockResolvedValue(true);
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterHpControls current={current} onAction={onAction} />
    </ConfigProvider>,
  );
  expect(screen.getByLabelText("HP summary")).toHaveTextContent("12/20");
  fireEvent.click(screen.getByRole("button", { name: "Edit Current HP" }));
  const input = screen.getByRole("textbox", { name: "Current HP" });
  fireEvent.change(input, { target: { value: "9" } });
  fireEvent.keyDown(input, { key: "Escape" });
  expect(onAction).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Edit Current HP" }));
  fireEvent.change(screen.getByRole("textbox", { name: "Current HP" }), {
    target: { value: "8" },
  });
  fireEvent.keyDown(screen.getByRole("textbox", { name: "Current HP" }), {
    key: "Enter",
  });
  await waitFor(() =>
    expect(onAction).toHaveBeenCalledWith({ kind: "set_current", input: "8" }),
  );
  fireEvent.click(screen.getByRole("button", { name: "Add temp HP" }));
  fireEvent.change(screen.getByRole("textbox", { name: "Temporary HP" }), {
    target: { value: "5" },
  });
  fireEvent.blur(screen.getByRole("textbox", { name: "Temporary HP" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenCalledWith({ kind: "set_temporary", input: "5" }),
  );
  expect(onAction).toHaveBeenCalledTimes(2);
});

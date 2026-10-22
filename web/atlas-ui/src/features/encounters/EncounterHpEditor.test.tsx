import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { EncounterHpEditor } from "./EncounterHpEditor";
import type { EncounterParticipantView } from "../../generated/atlas";
const current: EncounterParticipantView = {
  revision: 1n,
  knockout_options: [],
  effective_max_hp: { state: "value", value: 25, adjustment: null },
  persistent_damage_groups: [],
  track_dying: false,
  mortality: "alive",
  base_death_threshold: 4n,
  participant_key: "a",
  position: 1n,
  initiative_order: 1n,
  display_name: "Mage",
  hp_origin: "explicit",
  variant_origin: "explicit",
  participant_kind: "creature",
  side: "enemy",
  participant_variant: "normal",
  status: "active",
  current_hp: 20n,
  max_hp: 25n,
  temporary_hp: 2n,
  defeated: false,
  hazard_disabled: false,
  cast_unavailable_reason: null,
  hidden: false,
  note_hint: null,
  conditions: [],
};
it("sends one atomic raw correction for all edited fields and Cancel does not write", async () => {
  const onAction = vi.fn().mockResolvedValue(true);
  const onCommitted = vi.fn();
  const onCancel = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterHpEditor
        current={current}
        disabled={false}
        pending={false}
        onAction={onAction}
        onCommitted={onCommitted}
        onCancel={onCancel}
      />
    </ConfigProvider>,
  );
  fireEvent.change(screen.getByLabelText("Current HP"), {
    target: { value: " 20 - 3 " },
  });
  fireEvent.change(screen.getByLabelText("Maximum HP"), {
    target: { value: "30 + 2" },
  });
  fireEvent.change(screen.getByLabelText("Temporary HP"), { target: { value: "0" } });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onAction).toHaveBeenCalledOnce());
  expect(onAction).toHaveBeenCalledWith({
    kind: "set_values",
    current: { input: " 20 - 3 " },
    maximum: { input: "30 + 2" },
    temporary: "0",
  });
  expect(onCommitted).toHaveBeenCalledWith(true);
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(onCancel).toHaveBeenCalledOnce();
  expect(onAction).toHaveBeenCalledOnce();
});

it("closes an untouched form without writing and preserves explicit same-value intent", async () => {
  const onAction = vi.fn().mockResolvedValue(true);
  const onCancel = vi.fn();
  render(
    <EncounterHpEditor
      current={current}
      disabled={false}
      pending={false}
      onAction={onAction}
      onCommitted={vi.fn()}
      onCancel={onCancel}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onCancel).toHaveBeenCalledOnce());
  expect(onAction).not.toHaveBeenCalled();
  fireEvent.change(screen.getByLabelText("Maximum HP"), { target: { value: "26" } });
  fireEvent.change(screen.getByLabelText("Maximum HP"), { target: { value: "25" } });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenCalledExactlyOnceWith({
      kind: "set_values",
      maximum: { input: "25" },
    }),
  );
});
it("keeps the raw correction after failure and empty current is explicit unknown", async () => {
  const onAction = vi.fn().mockRejectedValue(new Error("Invalid formula"));
  const onCommitted = vi.fn();
  render(
    <EncounterHpEditor
      current={current}
      disabled={false}
      pending={false}
      onAction={onAction}
      onCommitted={onCommitted}
      onCancel={vi.fn()}
    />,
  );
  fireEvent.change(screen.getByLabelText("Current HP"), {
    target: { value: "2 + bad" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onAction).toHaveBeenCalledOnce());
  expect(screen.getByLabelText("Current HP")).toHaveValue("2 + bad");
  expect(onCommitted).not.toHaveBeenCalled();
  fireEvent.change(screen.getByLabelText("Current HP"), { target: { value: "" } });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenLastCalledWith({
      kind: "set_values",
      current: { input: null },
    }),
  );
});

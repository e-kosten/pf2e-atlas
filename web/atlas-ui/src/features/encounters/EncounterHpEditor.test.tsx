import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { EncounterHpEditor } from "./EncounterHpEditor";
import type { EncounterParticipantView } from "../../generated/atlas";
const current: EncounterParticipantView = {
  revision: 1n,
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
  hidden: false,
  note_hint: null,
  conditions: [],
};
it("sends one raw correction per explicit action and Cancel does not write", async () => {
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
  fireEvent.change(screen.getByLabelText("HP"), { target: { value: " 20 - 3 " } });
  fireEvent.click(screen.getByRole("button", { name: "Set current HP" }));
  await waitFor(() => expect(onAction).toHaveBeenCalledOnce());
  expect(onAction).toHaveBeenCalledWith({ kind: "set_current", input: " 20 - 3 " });
  expect(onCommitted).toHaveBeenCalledWith(true);
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(onCancel).toHaveBeenCalledOnce();
  expect(onAction).toHaveBeenCalledOnce();
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
  fireEvent.change(screen.getByLabelText("HP"), { target: { value: "2 + bad" } });
  fireEvent.click(screen.getByRole("button", { name: "Set current HP" }));
  await waitFor(() => expect(onAction).toHaveBeenCalledOnce());
  expect(screen.getByLabelText("HP")).toHaveValue("2 + bad");
  expect(onCommitted).not.toHaveBeenCalled();
  fireEvent.change(screen.getByLabelText("HP"), { target: { value: "" } });
  fireEvent.click(screen.getByRole("button", { name: "Set current HP" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenLastCalledWith({ kind: "set_current", input: null }),
  );
});

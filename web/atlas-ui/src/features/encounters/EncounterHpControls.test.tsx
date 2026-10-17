import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { EncounterHpControls } from "./EncounterHpControls";
import type { EncounterParticipantView } from "../../generated/atlas";

it("shows updated derived HP after a submitted edit and variant capacity change", async () => {
  const current: EncounterParticipantView = {
    revision: 0n,
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

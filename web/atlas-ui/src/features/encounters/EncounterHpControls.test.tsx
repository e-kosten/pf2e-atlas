import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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
  );
  fireEvent.change(screen.getByRole("textbox", { name: "HP" }), {
    target: { value: "13" },
  });
  fireEvent.click(screen.getAllByRole("button", { name: "Set" })[0]);
  expect(onAction).toHaveBeenCalledWith({ kind: "set_current", input: "13" });
  await waitFor(() =>
    expect(screen.getByRole("textbox", { name: "HP" })).toHaveValue("20"),
  );
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
  expect(screen.getByRole("textbox", { name: "HP" })).toHaveValue("23");
});

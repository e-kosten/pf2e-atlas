import { fireEvent, render, screen } from "@testing-library/react";
import { EncounterHpControls } from "./EncounterHpControls";
import type { EncounterParticipantView } from "../../generated/atlas";

it("shows updated derived HP after a submitted edit and variant capacity change", () => {
  const current: EncounterParticipantView = {
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
  const onUpdate = vi.fn();
  const { rerender } = render(
    <EncounterHpControls current={current} onUpdate={onUpdate} />,
  );
  fireEvent.change(screen.getByRole("textbox", { name: "HP" }), {
    target: { value: "13" },
  });
  fireEvent.click(screen.getAllByRole("button", { name: "Set" })[0]);
  expect(onUpdate).toHaveBeenCalledWith(expect.objectContaining({ current_hp: 13n }));
  rerender(
    <EncounterHpControls
      current={{
        ...current,
        current_hp: 23n,
        max_hp: 30n,
        participant_variant: "elite",
        variant_origin: "explicit",
      }}
      onUpdate={onUpdate}
    />,
  );
  expect(screen.getByRole("textbox", { name: "HP" })).toHaveValue("23");
});

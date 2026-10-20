import { fireEvent, render, screen, within } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { useState } from "react";
import { ConfusionContextFields } from "./ConfusionContextFields";
import { TargetContextFields } from "./TargetContextFields";
import type {
  EncounterActionTargetView,
  EncounterConfusionContextView,
  EncounterParticipantView,
} from "../../generated/atlas";
async function choose(label: string, value: string) {
  fireEvent.mouseDown(screen.getByRole("combobox", { name: label }));
  const combo = screen.getByRole("combobox", { name: label });
  const dropdown = document
    .getElementById(combo.getAttribute("aria-controls")!)!
    .closest<HTMLElement>(".ant-select-dropdown")!;
  fireEvent.click(
    await within(dropdown).findByText(value, {
      selector: ".ant-select-item-option-content",
    }),
  );
}
it("requires an explicit viable list and a table-random target instead of using the roster automatically", async () => {
  const onChange = vi.fn();
  function Harness() {
    const [value, setValue] = useState<EncounterConfusionContextView | null>(null);
    return (
      <ConfusionContextFields
        requirements={{
          source_ids: [1n],
          required_facts: [],
          selected_target_key: null,
          automatic_self_hit: false,
          wastes_action: false,
        }}
        value={value}
        participants={
          [
            { participant_key: "ally", display_name: "Ally" },
            { participant_key: "foe", display_name: "Foe" },
          ] as EncounterParticipantView[]
        }
        onChange={(next) => {
          setValue(next);
          onChange(next);
        }}
      />
    );
  }
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <Harness />
    </ConfigProvider>,
  );
  await choose("Confused action", "Offensive action");
  expect(onChange).toHaveBeenLastCalledWith({
    kind: "offensive",
    viable_target_keys: [],
    random_target_key: null,
    offensive: true,
  });
  await choose("Viable targets", "Ally");
  fireEvent.keyDown(screen.getByRole("combobox", { name: "Viable targets" }), {
    key: "Escape",
  });
  expect(onChange).toHaveBeenLastCalledWith({
    kind: "offensive",
    viable_target_keys: ["ally"],
    random_target_key: null,
    offensive: true,
  });
  await choose("Random target (table result)", "Ally");
  expect(onChange).toHaveBeenLastCalledWith({
    kind: "offensive",
    viable_target_keys: ["ally"],
    random_target_key: "ally",
    offensive: true,
  });
});
it("renders only the requested precision context and retains an explicit false answer", async () => {
  const onChange = vi.fn();
  const target: EncounterActionTargetView = {
    participant_key: "foe",
    mode: "direct",
    vision_only_precise: null,
    distance_feet: null,
    guessed_location_correct: null,
    visual_effect: null,
    auditory_effect: null,
  };
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <TargetContextFields
        target={target}
        targeting={{
          visual_effects_immune: false,
          auditory_effects_immune: false,
          prevents_effect: false,
          flat_check_dc: null,
          source_ids: [],
          requires_guessed_location: false,
          required_facts: ["Confirm the usable precise sense."],
          required_inputs: ["vision_only_precise"],
          secret: false,
        }}
        onChange={onChange}
      />
    </ConfigProvider>,
  );
  expect(
    screen.queryByRole("spinbutton", { name: "Target distance in feet" }),
  ).not.toBeInTheDocument();
  await choose("Vision is the only usable precise sense", "No");
  expect(onChange).toHaveBeenCalledWith({ ...target, vision_only_precise: false });
});

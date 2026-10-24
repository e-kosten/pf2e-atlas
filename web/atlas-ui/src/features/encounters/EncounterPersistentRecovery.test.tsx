import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type {
  PersistentDamageGroupView,
  PersistentRecoveryResultView,
} from "../../generated/atlas";
import { EncounterPersistentRecovery } from "./EncounterPersistentRecovery";
const group: PersistentDamageGroupView = {
  episode_id: "fire-episode",
  damage_type: "fire",
  selected_source_id: 7n,
  sources: [
    { condition_id: 7n, formula: "1d6", multiplier: "normal", recovery_dc: 15n },
  ],
};
const receipt: PersistentRecoveryResultView = {
  encounter_key: "encounter",
  request_id: "receipt",
  episode_id: "fire-episode",
  damage_type: "fire",
  dc: 15n,
  roll: null,
  success: true,
};
function setup(current = group) {
  const onRecover = vi.fn().mockResolvedValue(receipt);
  const onResult = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterPersistentRecovery
        group={current}
        mode="explicit"
        available={true}
        participantKey="actor"
        encounterRevision={11n}
        disabled={false}
        pending={false}
        damageType="Fire"
        onRecover={onRecover}
        onResult={onResult}
      />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Roll Fire recovery" }));
  return { onRecover, onResult };
}
it("rolls recovery without requiring a table result and preserves the episode identity", async () => {
  const { onRecover, onResult } = setup();
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  await waitFor(() =>
    expect(onRecover).toHaveBeenCalledWith({
      resolution: "roll",
      mode: "explicit",
      boundary_key: null,
      expected_encounter_revision: 11n,
      participant_key: "actor",
      episode_id: "fire-episode",
      dc_override: null,
    }),
  );
  expect(onResult).toHaveBeenCalledWith(receipt);
});
it("keeps recovery results inside the popover and offers no separate table-success flow", async () => {
  setup();
  expect(await screen.findByRole("spinbutton", { name: "Recovery DC" })).toHaveValue(
    "15",
  );
  expect(screen.queryByRole("button", { name: "Succeeded" })).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  await screen.findByText("Recovered · DC 15");
  expect(screen.getByRole("button", { name: "Roll" })).toBeDisabled();
  expect(screen.getByRole("spinbutton", { name: "Recovery DC" })).toHaveValue("15");
  expect(screen.getByRole("spinbutton", { name: "Recovery DC" })).toBeDisabled();
});
it("requests a missing DC for the precedence-selected source without another source picker", async () => {
  const { onRecover } = setup({
    ...group,
    sources: [{ ...group.sources[0], recovery_dc: null }],
  });
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  expect(screen.queryByRole("combobox", { name: "Recovery damage source" })).toBeNull();
  await screen.findByText("Enter a whole number.");
  expect(onRecover).not.toHaveBeenCalled();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Recovery DC" }), {
    target: { value: "10" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  await waitFor(() =>
    expect(onRecover).toHaveBeenCalledWith(
      expect.objectContaining({ dc_override: 10n }),
    ),
  );
});

it("uses the edited DC and keeps the previous result's DC when preparing another roll", async () => {
  const { onRecover } = setup();
  onRecover.mockResolvedValueOnce({ ...receipt, dc: 10n, success: false });
  fireEvent.change(await screen.findByRole("spinbutton", { name: "Recovery DC" }), {
    target: { value: "10" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  await waitFor(() =>
    expect(onRecover).toHaveBeenCalledWith(
      expect.objectContaining({ dc_override: 10n }),
    ),
  );
  await screen.findByText("Still active · DC 10");
  fireEvent.change(screen.getByRole("spinbutton", { name: "Recovery DC" }), {
    target: { value: "7" },
  });
  expect(screen.getByText("Still active · DC 10")).toBeVisible();
});

it("presents a recorded end-turn check as a result without asserting the effect is still active", async () => {
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterPersistentRecovery
        group={group}
        mode="end_turn"
        available={false}
        recordedResult={{ ...receipt, success: false }}
        participantKey="actor"
        encounterRevision={12n}
        disabled={false}
        pending={false}
        damageType="Fire"
        onRecover={vi.fn()}
      />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "View Fire recovery" }));
  await waitFor(() =>
    expect(screen.getByText("Recovery failed · DC 15")).toBeVisible(),
  );
  expect(screen.queryByRole("spinbutton", { name: "Recovery DC" })).toBeNull();
  expect(screen.queryByRole("button", { name: "Roll" })).toBeNull();
});

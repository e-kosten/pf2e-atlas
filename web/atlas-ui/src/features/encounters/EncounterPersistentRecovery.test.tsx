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
  table_result: 18n,
  success: true,
};
function setup(current = group) {
  const onRecover = vi.fn().mockResolvedValue(receipt);
  const onResult = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterPersistentRecovery
        group={current}
        mode="assisted"
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
      mode: "assisted",
      boundary_key: null,
      expected_encounter_revision: 11n,
      participant_key: "actor",
      episode_id: "fire-episode",
      selected_source_id: 7n,
      outcome: { kind: "roll" },
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
  expect(screen.queryByRole("button", { name: "Roll" })).not.toBeInTheDocument();
});
it("requests an unresolved source and its missing DC instead of choosing or calculating them", async () => {
  const { onRecover } = setup({
    ...group,
    selected_source_id: null,
    sources: [{ ...group.sources[0], recovery_dc: null }],
  });
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  await screen.findByText("Choose the source with the greater damage.");
  expect(onRecover).not.toHaveBeenCalled();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Recovery damage source" }));
  fireEvent.click(await screen.findByText("1d6"));
  await screen.findByRole("spinbutton", { name: "Recovery DC" });
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  await screen.findByText("Enter a whole number.");
  expect(onRecover).not.toHaveBeenCalled();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Recovery DC" }), {
    target: { value: "10" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  await waitFor(() =>
    expect(onRecover).toHaveBeenCalledWith(
      expect.objectContaining({ selected_source_id: 7n, dc_override: 10n }),
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

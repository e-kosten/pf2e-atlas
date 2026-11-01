import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
const api = vi.hoisted(() => ({
  getEncounterHpLifecycle: vi.fn().mockResolvedValue([]),
}));
vi.mock("../../api/atlasApi", () => api);
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterLifeStatus } from "./EncounterLifeStatus";
import { encounterParticipant } from "./encounterTestFixtures";
const context = {
  lethality: "lethal" as const,
  critical: false,
  enemy_source: null,
  responsible_participant_key: null,
  base_death_threshold: null,
  keep_current_initiative: true,
  attack_or_spell_damage: null,
  natural_rest_healing: null,
  keep_upright: null,
};
it("starts tracking a defeated zero-HP enemy using knockout choices without entering settings", async () => {
  const onSave = vi.fn().mockResolvedValue(true);
  const actor = encounterParticipant({
    track_dying: false,
    current_hp: 0n,
    defeated: true,
    manual_trackers: [],
    knockout_options: [{ label: "Normal", result: "Dying 2 · Unconscious", context }],
  });
  render(
    <QueryClientProvider client={new QueryClient()}>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <EncounterLifeStatus
          encounterRef="encounter"
          revision={1n}
          onResolve={vi.fn()}
          participant={actor}
          participants={[actor]}
          disabled={false}
          onSave={onSave}
          onDefeated={vi.fn()}
        />
      </ConfigProvider>
    </QueryClientProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "More status actions" }));
  fireEvent.click(await screen.findByRole("button", { name: "Track dying" }));
  await screen.findByText("Dying 2 · Unconscious");
  expect(onSave).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith({
      track_dying: true,
      knockout_context: context,
    }),
  );
});
it("marks a dead creature alive without changing its tracking preference or HP", async () => {
  const onSave = vi.fn().mockResolvedValue(true),
    actor = encounterParticipant({
      track_dying: false,
      mortality: "dead",
      current_hp: 0n,
    });
  render(
    <QueryClientProvider client={new QueryClient()}>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <EncounterLifeStatus
          encounterRef="encounter"
          revision={1n}
          onResolve={vi.fn()}
          participant={actor}
          participants={[actor]}
          disabled={false}
          onSave={onSave}
          onDefeated={vi.fn()}
        />
      </ConfigProvider>
    </QueryClientProvider>,
  );
  fireEvent.click(await screen.findByRole("button", { name: "Mark alive" }));
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith({ track_dying: false, mortality: "alive" }),
  );
});

it("clears defeated directly without opening a status form", async () => {
  const onDefeated = vi.fn().mockResolvedValue(true);
  render(
    <QueryClientProvider client={new QueryClient()}>
      <EncounterLifeStatus
        encounterRef="encounter"
        revision={1n}
        onResolve={vi.fn()}
        onSave={vi.fn()}
        participant={encounterParticipant({ defeated: true })}
        participants={[]}
        disabled={false}
        onDefeated={onDefeated}
      />
    </QueryClientProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Clear defeated" }));
  await waitFor(() => expect(onDefeated).toHaveBeenCalledOnce());
  expect(screen.queryByRole("button", { name: "Track dying" })).toBeNull();
});

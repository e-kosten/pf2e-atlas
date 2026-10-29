import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { EncounterLifeStatus } from "./EncounterLifeStatus";
import { encounterParticipant } from "./encounterTestFixtures";
import type { PendingHpLifecycleView } from "../../generated/atlas";
const api = vi.hoisted(() => ({ getEncounterHpLifecycle: vi.fn() }));
vi.mock("../../api/atlasApi", () => api);
const context = {
  lethality: null,
  critical: null,
  enemy_source: null,
  responsible_participant_key: null,
  base_death_threshold: null,
  keep_current_initiative: null,
  attack_or_spell_damage: null,
  natural_rest_healing: null,
  keep_upright: null,
};
const row: PendingHpLifecycleView = {
  consequence_key: "damage-1",
  participant_key: "actor",
  before_hp: 8n,
  after_hp: 0n,
  initiative_only: false,
  required_facts: ["lethality"],
  knockout_options: [],
  context,
};
const actor = encounterParticipant({
  current_hp: 0n,
  knockout_options: [
    {
      label: "Normal",
      result: "Dying 1 · Unconscious",
      context: {
        ...context,
        lethality: "lethal",
        critical: false,
        keep_current_initiative: true,
      },
    },
  ],
});
function setup(value = row) {
  api.getEncounterHpLifecycle.mockResolvedValue([
    {
      ...value,
      knockout_options: value.unavailable_reason ? [] : actor.knockout_options,
    },
  ]);
  const onResolve = vi
    .fn()
    .mockRejectedValueOnce(new Error("Connection lost"))
    .mockResolvedValue(true);
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <EncounterLifeStatus
          encounterRef="encounter"
          participant={actor}
          onSave={vi.fn()}
          onDefeated={vi.fn()}
          revision={4n}
          participants={[
            actor,
            encounterParticipant({
              participant_key: "ogre",
              display_name: "Ogre",
              initiative: 20n,
            }),
          ]}
          currentTurnKey="ogre"
          disabled={false}
          onResolve={onResolve}
        />
      </ConfigProvider>
    </QueryClientProvider>,
  );
  return onResolve;
}
it("combines the projected knockout outcome and suggested initiative and retains them after a failed save", async () => {
  const onResolve = setup();
  fireEvent.click(await screen.findByRole("button", { name: "More status actions" }));
  await screen.findByText("Dying 1 · Unconscious");
  await screen.findByText("Initiative · Before Ogre");
  expect(screen.queryByRole("combobox", { name: "Damage lethality" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await screen.findByText("Could not save knockout.");
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await waitFor(() => expect(onResolve).toHaveBeenCalledTimes(2));
  expect(onResolve.mock.calls[0][0]).toEqual(onResolve.mock.calls[1][0]);
  expect(onResolve).toHaveBeenLastCalledWith(
    expect.objectContaining({
      consequence_key: "damage-1",
      resolution: "apply",
      context: expect.objectContaining({
        lethality: "lethal",
        critical: false,
        responsible_participant_key: "ogre",
        keep_current_initiative: false,
      }),
    }),
  );
});
it("presents an initiative-only task without asking for a knockout outcome", async () => {
  const onResolve = setup({
    ...row,
    initiative_only: true,
    context: { ...context, keep_current_initiative: true },
  });
  fireEvent.click(await screen.findByRole("button", { name: "More status actions" }));
  await screen.findByText("Initiative · Keep current position");
  expect(screen.queryByRole("combobox", { name: "Knockout outcome" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await waitFor(() =>
    expect(onResolve).toHaveBeenCalledWith(
      expect.objectContaining({
        context: expect.objectContaining({
          keep_current_initiative: true,
          responsible_participant_key: null,
        }),
      }),
    ),
  );
});
it("closing leaves the pending knockout available without changing encounter state", async () => {
  const onResolve = setup();
  const trigger = await screen.findByRole("button", { name: "More status actions" });
  fireEvent.click(trigger);
  await screen.findByText("Dying 1 · Unconscious");
  expect(screen.queryByRole("button", { name: "Skip" })).toBeNull();
  fireEvent.click(trigger);
  await waitFor(() =>
    expect(screen.queryByRole("button", { name: "Apply" })).toBeNull(),
  );
  expect(onResolve).not.toHaveBeenCalled();
  expect(trigger).toBeVisible();
  fireEvent.click(trigger);
  await screen.findByRole("button", { name: "Apply" });
});

it("explains uncertain knockout facts while leaving unrelated status controls usable", async () => {
  const onResolve = setup({
    ...row,
    unavailable_reason:
      "Check the Doomed value and source before resolving this event. Source: Maintained condition.",
  });
  fireEvent.click(await screen.findByRole("button", { name: "More status actions" }));
  expect(await screen.findByRole("status")).toHaveTextContent("Doomed");
  expect(screen.getByRole("status")).toHaveTextContent("Maintained condition");
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Mark defeated" })).toBeEnabled();
  expect(onResolve).not.toHaveBeenCalled();
});

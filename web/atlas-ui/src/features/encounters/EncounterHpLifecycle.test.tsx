import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { EncounterHpLifecycle } from "./EncounterHpLifecycle";
import type { PendingHpLifecycleView } from "../../generated/atlas";
const api = vi.hoisted(() => ({
  getEncounterHpLifecycle: vi.fn(),
  previewEncounterHpLifecycle: vi.fn(),
}));
vi.mock("../../api/atlasApi", () => api);
const row: PendingHpLifecycleView = {
  consequence_key: "damage-1",
  participant_key: "actor",
  before_hp: 8n,
  after_hp: 0n,
  initiative_only: false,
  required_facts: ["lethality"],
  context: {
    lethality: null,
    critical: null,
    enemy_source: null,
    responsible_participant_key: null,
    base_death_threshold: null,
    keep_current_initiative: false,
    attack_or_spell_damage: null,
    natural_rest_healing: false,
    keep_upright: false,
  },
};
function setup(value = row) {
  api.getEncounterHpLifecycle.mockResolvedValue([value]);
  const onResolve = vi
    .fn()
    .mockRejectedValueOnce(new Error("Connection lost"))
    .mockResolvedValue(true);
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <EncounterHpLifecycle
          encounterRef="encounter"
          revision={4n}
          participants={[]}
          disabled={false}
          onResolve={onResolve}
        />
      </ConfigProvider>
    </QueryClientProvider>,
  );
  return onResolve;
}
async function choose(label: string, option: string) {
  fireEvent.mouseDown(screen.getByRole("combobox", { name: label }));
  fireEvent.click(
    await screen.findByText(option, { selector: ".ant-select-item-option-content" }),
  );
}
it("asks only backend-required damage facts and retains them across a failed guarded save", async () => {
  api.previewEncounterHpLifecycle.mockImplementation(async (_ref, request) => ({
    ...row,
    context: request.context,
    required_facts:
      request.context.lethality === null
        ? ["lethality"]
        : request.context.critical === null
          ? ["critical"]
          : [],
  }));
  const onResolve = setup();
  expect(
    await screen.findByRole("combobox", { name: "Damage lethality" }),
  ).toBeInTheDocument();
  expect(
    screen.queryByRole("combobox", { name: "Critical damage" }),
  ).not.toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Apply follow-up" })).toBeDisabled();
  await choose("Damage lethality", "Lethal");
  await screen.findByRole("combobox", { name: "Critical damage" });
  await choose("Critical damage", "Not a critical hit");
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply follow-up" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply follow-up" }));
  await screen.findByText("Connection lost");
  fireEvent.click(screen.getByRole("button", { name: "Apply follow-up" }));
  await waitFor(() => expect(onResolve).toHaveBeenCalledTimes(2));
  expect(onResolve.mock.calls[0][0]).toEqual(onResolve.mock.calls[1][0]);
  expect(onResolve.mock.calls[1][0]).toEqual({
    expected_encounter_revision: 4n,
    consequence_key: "damage-1",
    resolution: "apply" as const,
    context: { ...row.context, lethality: "lethal", critical: false },
  });
  expect(
    screen.queryByRole("combobox", { name: "Enemy damage source" }),
  ).not.toBeInTheDocument();
});
it("records keeping initiative as an explicit choice without replaying HP", async () => {
  const initiative = {
    ...row,
    initiative_only: true,
    required_facts: [
      "responsible_participant",
    ] as PendingHpLifecycleView["required_facts"],
  };
  api.previewEncounterHpLifecycle.mockImplementation(async (_ref, request) => ({
    ...initiative,
    context: request.context,
    required_facts: request.context.keep_current_initiative
      ? []
      : ["responsible_participant"],
  }));
  const onResolve = setup(initiative);
  await screen.findByRole("combobox", { name: "Initiative follows" });
  await choose("Initiative follows", "Keep current initiative");
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply follow-up" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply follow-up" }));
  await waitFor(() =>
    expect(onResolve).toHaveBeenCalledWith(
      expect.objectContaining({
        context: { ...row.context, keep_current_initiative: true },
      }),
    ),
  );
});
it("can retain the current table state without fabricating historical damage facts", async () => {
  api.previewEncounterHpLifecycle.mockImplementation(async (_ref, request) => ({
    ...row,
    context: request.context,
    required_facts: request.resolution === "keep_current_state" ? [] : ["lethality"],
  }));
  const onResolve = setup();
  await screen.findByRole("combobox", { name: "Damage lethality" });
  await choose("HP follow-up resolution", "Keep current state");
  await waitFor(() =>
    expect(
      screen.queryByRole("combobox", { name: "Damage lethality" }),
    ).not.toBeInTheDocument(),
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply follow-up" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply follow-up" }));
  await waitFor(() =>
    expect(onResolve).toHaveBeenCalledWith({
      expected_encounter_revision: 4n,
      consequence_key: "damage-1",
      context: row.context,
      resolution: "keep_current_state",
    }),
  );
});

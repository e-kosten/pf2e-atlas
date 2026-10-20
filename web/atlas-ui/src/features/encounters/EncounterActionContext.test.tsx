import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { EncounterActionContext } from "./EncounterActionContext";
import type {
  EncounterParticipantView,
  PreviewEncounterActionRequest,
} from "../../generated/atlas";
const api = vi.hoisted(() => ({ previewEncounterAction: vi.fn() }));
vi.mock("../../api/atlasApi", () => api);
const actor: EncounterParticipantView = {
  participant_key: "actor",
  display_name: "Actor",
  participant_kind: "creature",
  revision: 1n,
  hp_origin: "explicit",
  variant_origin: "explicit",
  participant_variant: "normal",
  status: "active",
  position: 1n,
  side: "enemy",
  initiative_order: 1n,
  temporary_hp: 0n,
  defeated: false,
  hazard_disabled: false,
  cast_unavailable_reason: null,
  hidden: false,
  note_hint: null,
  track_dying: false,
  mortality: "alive",
  base_death_threshold: 4n,
  persistent_damage_groups: [],
  conditions: [],
};
const empty = {
  encounter_key: "encounter",
  participant_key: "actor",
  expected_revision: 4n,
  expected_participant_revision: 1n,
  issues: [],
  disruption: null,
  considers_target_ally: null,
  can_flank: true,
  targeting: null,
  confusion: null,
  quickened_use: null,
  request: null,
  sensory: null,
  forced_movement: null,
  prone_cover: null,
};
function setup(fixed = false, subjectActor = actor) {
  const onEdit = vi.fn();
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <EncounterActionContext
          encounterRef="encounter"
          revision={4n}
          participant={subjectActor}
          participants={[
            subjectActor,
            { ...actor, participant_key: "recipient", display_name: "Recipient" },
          ]}
          disabled={false}
          onEditCondition={onEdit}
          initialMode={fixed ? "request" : "action"}
          fixedOther={fixed ? "recipient" : undefined}
          initiallyOpen={fixed}
        />
      </ConfigProvider>
    </QueryClientProvider>,
  );
  return onEdit;
}
async function choose(label: string, value: string) {
  fireEvent.mouseDown(await screen.findByRole("combobox", { name: label }));
  const dropdown = document.querySelector<HTMLElement>(
    ".ant-select-dropdown:not(.ant-select-dropdown-hidden)",
  );
  expect(dropdown).not.toBeNull();
  fireEvent.click(
    await within(dropdown!).findByText(value, {
      selector: ".ant-select-item-option-content",
    }),
  );
}
it("queries optional contexts and renders backend extra-action, sensory and cover decisions", async () => {
  api.previewEncounterAction.mockImplementation(
    async (_ref, request: PreviewEncounterActionRequest) => ({
      ...empty,
      quickened_use: request.extra_action
        ? { allowed: true, allowing_source_ids: [], unknown_source_ids: [] }
        : null,
      sensory: request.sensory
        ? {
            perception: 3,
            critical_failure: true,
            visual_effects_immune: false,
            auditory_effects_immune: true,
            all_terrain_difficult: false,
            required_facts: [],
            required_inputs: [],
          }
        : null,
      prone_cover: request.prone_cover ? { armor_class: 27, required_facts: [] } : null,
    }),
  );
  setup();
  expect(api.previewEncounterAction).not.toHaveBeenCalled();
  fireEvent.click(screen.getByText("Action context"));
  await choose("Action context type", "Extra action");
  await choose("Extra action to check", "Strike");
  expect(await screen.findByText("Extra action: Allowed")).toBeVisible();
  expect(api.previewEncounterAction).toHaveBeenLastCalledWith(
    "encounter",
    expect.objectContaining({ extra_action: "strike" }),
  );
  await choose("Action context type", "Perception and senses");
  await choose("Perception basis", "Auditory only");
  fireEvent.change(screen.getByRole("spinbutton", { name: "Sensory distance" }), {
    target: { value: "60" },
  });
  expect(await screen.findByText("Perception: 3")).toBeVisible();
  expect(screen.getByText("Critical failure")).toBeVisible();
  await waitFor(() =>
    expect(api.previewEncounterAction).toHaveBeenLastCalledWith(
      "encounter",
      expect.objectContaining({
        sensory: { basis: "auditory_only", distance_feet: 60, requires_movement: null },
      }),
    ),
  );
  await choose("Action context type", "Prone cover");
  await choose("Cover against ranged attack", "Yes");
  expect(await screen.findByText("AC: 27")).toBeVisible();
  expect(api.previewEncounterAction).toHaveBeenLastCalledWith(
    "encounter",
    expect.objectContaining({ prone_cover: { against_ranged_attack: true } }),
  );
});
it("keeps a Request tied to requester and recipient and displays the server's GM guidance", async () => {
  api.previewEncounterAction.mockResolvedValue({
    ...empty,
    request: {
      outcome: "check_required",
      reason: "GM chooses the Request DC.",
      required_facts: [],
    },
  });
  setup(true);
  await choose("Reasonable request", "Yes");
  await choose("Against recipient goals", "No");
  expect(await screen.findByText("GM chooses the Request DC.")).toBeVisible();
  await waitFor(() =>
    expect(api.previewEncounterAction).toHaveBeenLastCalledWith(
      "encounter",
      expect.objectContaining({
        participant_key: "actor",
        action: "request",
        target: {
          participant_key: "recipient",
          mode: "direct",
          vision_only_precise: null,
          distance_feet: null,
          guessed_location_correct: null,
          visual_effect: null,
          auditory_effect: null,
        },
        request_context: { reasonable: true, against_goals: false },
      }),
    ),
  );
});

it("clears target-specific sensory answers when the selected participant changes", async () => {
  api.previewEncounterAction.mockResolvedValue({
    ...empty,
    targeting: {
      flat_check_dc: null,
      source_ids: [],
      requires_guessed_location: false,
      required_facts: [],
      required_inputs: ["vision_only_precise"],
      secret: false,
      visual_effects_immune: false,
      auditory_effects_immune: false,
      prevents_effect: false,
    },
  });
  setup();
  fireEvent.click(screen.getByText("Action context"));
  await choose("Action to check", "Strike");
  await choose("Action other participant", "Recipient");
  await choose("Vision is the only usable precise sense", "Yes");
  await waitFor(() =>
    expect(api.previewEncounterAction).toHaveBeenLastCalledWith(
      "encounter",
      expect.objectContaining({
        target: expect.objectContaining({
          participant_key: "recipient",
          vision_only_precise: true,
        }),
      }),
    ),
  );
  await choose("Action other participant", "Actor");
  await waitFor(() =>
    expect(api.previewEncounterAction).toHaveBeenLastCalledWith(
      "encounter",
      expect.objectContaining({
        target: expect.objectContaining({
          participant_key: "actor",
          vision_only_precise: null,
        }),
      }),
    ),
  );
});

it("shows projected ally and flanking facts without inferring them from roster sides", async () => {
  api.previewEncounterAction.mockResolvedValue({
    ...empty,
    can_flank: false,
    considers_target_ally: false,
  });
  setup();
  fireEvent.click(screen.getByText("Action context"));
  await choose("Action context type", "Allies and flanking");
  expect(await screen.findByText("Can flank: No")).toBeInTheDocument();
  expect(api.previewEncounterAction).toHaveBeenLastCalledWith(
    "encounter",
    expect.objectContaining({ target: null, normally_ally: null }),
  );
  await choose("Action other participant", "Recipient");
  await choose("Normally an ally", "Yes");
  await waitFor(() =>
    expect(api.previewEncounterAction).toHaveBeenLastCalledWith(
      "encounter",
      expect.objectContaining({
        normally_ally: true,
        target: expect.objectContaining({ participant_key: "recipient" }),
      }),
    ),
  );
  expect(screen.getByText("Considers target an ally: No")).toBeInTheDocument();
});

it("distinguishes independent named threats in accessible questions and falls back to a source ID", async () => {
  const reason = "Does this further your escape?";
  const issue = {
    condition_name: "Fleeing",
    controller_key: null,
    instructions: null,
    kind: "needs_fact" as const,
    fact: "furthers_escape" as const,
    reason,
  };
  api.previewEncounterAction.mockResolvedValue({
    ...empty,
    issues: [
      {
        ...issue,
        source_condition_id: 1n,
        subject: { kind: "participant", participant_key: "recipient" },
      },
      {
        ...issue,
        source_condition_id: 2n,
        subject: { kind: "other", description: "Dragon" },
      },
      {
        ...issue,
        kind: "needs_context",
        fact: null,
        source_condition_id: 3n,
        subject: null,
        reason: "Choose the threat.",
      },
    ],
  });
  const subjectActor = {
    ...actor,
    conditions: [1n, 2n, 3n].map((condition_id) => ({
      condition_id,
      condition_key: "conditions:fleeing",
      name: "Fleeing",
      parameters: { kind: "fleeing" as const, subject: null },
      scope: { kind: "everyone" as const },
      extra_action: { kind: "unspecified" as const },
      immunity_override: false,
      created_at: "now",
      updated_at: "now",
    })),
  };
  const onEdit = setup(false, subjectActor);
  fireEvent.click(screen.getByText("Action context"));
  await choose("Action to check", "Strike");
  const first = `${reason} · Subject: Recipient`;
  const second = `${reason} · Subject: Dragon`;
  expect(await screen.findByRole("combobox", { name: first })).toBeInTheDocument();
  expect(await screen.findByRole("combobox", { name: second })).toBeInTheDocument();
  await choose(first, "Yes");
  expect(screen.getByRole("combobox", { name: first })).toBeInTheDocument();
  expect(await screen.findByRole("combobox", { name: second })).toBeInTheDocument();
  expect(
    screen.getByText("Fleeing · Source 3: Choose the threat."),
  ).toBeInTheDocument();
  fireEvent.click(
    screen.getByRole("button", { name: "Edit Fleeing · Source 3 source" }),
  );
  expect(onEdit).toHaveBeenCalledWith("actor", 3n);
});

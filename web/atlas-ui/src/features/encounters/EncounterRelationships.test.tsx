import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { EncounterRelationships } from "./EncounterRelationships";
import type {
  EncounterParticipantView,
  EncounterRelationshipsView,
} from "../../generated/atlas";
const api = vi.hoisted(() => ({ getEncounterRelationships: vi.fn() }));
vi.mock("../../api/atlasApi", () => api);
const actor: EncounterParticipantView = {
  participant_key: "subject",
  display_name: "Mira",
  participant_kind: "pc",
  revision: 1n,
  hp_origin: "explicit",
  variant_origin: "explicit",
  participant_variant: "normal",
  status: "active",
  position: 1n,
  side: "pc",
  initiative_order: 1n,
  temporary_hp: 0n,
  defeated: false,
  hazard_disabled: false,
  cast_unavailable_reason: null,
  hidden: false,
  note_hint: null,
  track_dying: true,
  mortality: "alive",
  base_death_threshold: 4n,
  persistent_damage_groups: [],
  conditions: [],
};
const other = { ...actor, participant_key: "observer", display_name: "Guard" };
const pair: EncounterRelationshipsView = {
  encounter_key: "encounter",
  expected_revision: 4n,
  relationships: [
    {
      subject_key: "subject",
      other_key: "observer",
      property: "detection",
      source_condition_ids: [],
      adjudication: null,
      baseline: {
        kind: "known",
        value: { kind: "detection", value: "hidden" },
        provenance: { kind: "table" },
      },
      knowledge: {
        kind: "known",
        value: { kind: "detection", value: "hidden" },
        provenance: { kind: "table" },
      },
    },
    {
      subject_key: "subject",
      other_key: "observer",
      property: "control",
      baseline: null,
      adjudication: null,
      source_condition_ids: [],
      knowledge: { kind: "unknown", required_facts: [] },
    },
  ],
};
function setup(
  onEvent?: React.ComponentProps<typeof EncounterRelationships>["onEvent"],
  onWorsen?: React.ComponentProps<typeof EncounterRelationships>["onWorsen"],
  sourceActor = actor,
  onEditCondition?: React.ComponentProps<
    typeof EncounterRelationships
  >["onEditCondition"],
) {
  const onSave = vi.fn().mockResolvedValue(true);
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <EncounterRelationships
          encounterRef="encounter"
          revision={4n}
          subject={sourceActor}
          participants={[
            sourceActor,
            other,
            { ...other, participant_key: "third", display_name: "Watcher" },
          ]}
          disabled={false}
          onSave={onSave}
          onEvent={onEvent}
          onWorsen={onWorsen}
          onEditCondition={onEditCondition}
        />
      </ConfigProvider>
    </QueryClientProvider>,
  );
  return onSave;
}
async function choose(label: string, value: string) {
  fireEvent.mouseDown(await screen.findByRole("combobox", { name: label }));
  fireEvent.click(
    await screen.findByText(value, { selector: ".ant-select-item-option-content" }),
  );
}
it("keeps observer context optional and writes known knowledge in the directed pair", async () => {
  api.getEncounterRelationships.mockResolvedValue(pair);
  const onSave = setup();
  expect(api.getEncounterRelationships).not.toHaveBeenCalled();
  fireEvent.click(screen.getByText("Relationships"));
  await choose("Relative to participant", "Guard");
  await screen.findByText("Seen by Guard");
  expect(screen.getByText("Hidden")).toBeVisible();
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Edit detection baseline" }));
  const modal = await screen.findByRole("dialog", { name: "Detection baseline" });
  await choose("Relationship state", "Observed");
  fireEvent.click(within(modal).getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith({
      expected_encounter_revision: 4n,
      subject_key: "subject",
      other_key: "observer",
      property: "detection",
      layer: "baseline",
      knowledge: {
        kind: "known",
        value: { kind: "detection", value: "observed" },
        provenance: { kind: "table" },
      },
    }),
  );
});
it("distinguishes explicit not-controlled from unknown and does not reuse another observer's facts", async () => {
  api.getEncounterRelationships.mockImplementation((_ref, _subject, other) =>
    other === "observer" ? Promise.resolve(pair) : new Promise(() => {}),
  );
  const onSave = setup();
  fireEvent.click(screen.getByText("Relationships"));
  await choose("Relative to participant", "Guard");
  fireEvent.click(await screen.findByRole("button", { name: "Edit control baseline" }));
  const modal = await screen.findByRole("dialog", { name: "Control baseline" });
  await choose("Relationship state", "Not controlled by Guard");
  fireEvent.click(within(modal).getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        property: "control",
        knowledge: {
          kind: "known",
          value: { kind: "no_control" },
          provenance: { kind: "table" },
        },
      }),
    ),
  );
  await waitFor(() =>
    expect(
      screen.queryByRole("dialog", { name: "Control baseline" }),
    ).not.toBeInTheDocument(),
  );
  await choose("Relative to participant", "Watcher");
  expect(screen.queryByText("Seen by Guard")).not.toBeInTheDocument();
  expect(screen.queryByText("Hidden")).not.toBeInTheDocument();
});

it("records Seek as an observer-specific event with an explicit table outcome", async () => {
  api.getEncounterRelationships.mockResolvedValue(pair);
  const onEvent = vi.fn().mockResolvedValue(true);
  const onSave = setup(onEvent);
  fireEvent.click(screen.getByText("Relationships"));
  await choose("Relative to participant", "Guard");
  fireEvent.click(await screen.findByRole("button", { name: "Seek" }));
  const modal = await screen.findByRole("dialog", { name: "Guard seeks Mira" });
  await choose("Seek result", "Success");
  fireEvent.click(within(modal).getByRole("checkbox", { name: "Can observe Mira" }));
  fireEvent.click(within(modal).getByRole("button", { name: "Record" }));
  await waitFor(() =>
    expect(onEvent).toHaveBeenCalledWith({
      expected_encounter_revision: 4n,
      subject_key: "subject",
      observer_key: "observer",
      event: { kind: "seek", prior: "hidden", outcome: "success", can_observe: true },
    }),
  );
  expect(onSave).not.toHaveBeenCalled();
});

it("resolves active source uncertainty without editing the saved baseline", async () => {
  api.getEncounterRelationships.mockResolvedValue({
    ...pair,
    relationships: [
      {
        ...pair.relationships[0],
        source_condition_ids: [9n],
        knowledge: {
          kind: "unknown",
          required_facts: ["Choose the observer's detection state."],
        },
      },
    ],
  });
  const onSave = setup();
  fireEvent.click(screen.getByText("Relationships"));
  await choose("Relative to participant", "Guard");
  fireEvent.click(await screen.findByRole("button", { name: "Resolve detection" }));
  const modal = await screen.findByRole("dialog", { name: "Detection resolution" });
  await choose("Relationship state", "Observed");
  fireEvent.click(within(modal).getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        layer: "adjudication",
        knowledge: {
          kind: "known",
          value: { kind: "detection", value: "observed" },
          provenance: { kind: "table" },
        },
      }),
    ),
  );
});

it("records the table's worse attitude with its guarded previous state", async () => {
  api.getEncounterRelationships.mockResolvedValue({
    ...pair,
    relationships: [
      {
        ...pair.relationships[0],
        property: "attitude",
        knowledge: {
          kind: "known",
          value: { kind: "attitude", value: "friendly" },
          provenance: { kind: "table" },
        },
      },
    ],
  });
  const onWorsen = vi.fn().mockResolvedValue(true);
  const onSave = setup(undefined, onWorsen);
  fireEvent.click(screen.getByText("Relationships"));
  await choose("Relative to participant", "Guard");
  fireEvent.click(await screen.findByRole("button", { name: "Worsen attitude" }));
  const modal = await screen.findByRole("dialog", {
    name: "Worsen attitude toward Guard",
  });
  await choose("Worse attitude", "Unfriendly");
  fireEvent.click(within(modal).getByRole("button", { name: "Worsen attitude" }));
  await waitFor(() =>
    expect(onWorsen).toHaveBeenCalledWith({
      expected_encounter_revision: 4n,
      subject_key: "subject",
      other_key: "observer",
      previous: "friendly",
      attitude: "unfriendly",
    }),
  );
  expect(onSave).not.toHaveBeenCalled();
});

it("opens the attributed condition source without replacing independent relationship causes", async () => {
  const sourceActor: EncounterParticipantView = {
    ...actor,
    conditions: [
      {
        condition_id: 9n,
        condition_key: "conditions:hidden",
        name: "Hidden",
        scope: { kind: "selected_participants", participant_keys: ["observer"] },
        parameters: { kind: "none" },
        extra_action: { kind: "unspecified" },
        immunity_override: false,
        created_at: "now",
        updated_at: "now",
      },
    ],
  };
  api.getEncounterRelationships.mockResolvedValue({
    ...pair,
    relationships: [
      {
        ...pair.relationships[0],
        source_condition_ids: [9n],
        knowledge: {
          kind: "known",
          value: { kind: "detection", value: "hidden" },
          provenance: {
            kind: "source",
            condition_id: 9n,
            record_key: null,
            application_id: null,
          },
        },
      },
    ],
  });
  const onEdit = vi.fn();
  const onSave = setup(undefined, undefined, sourceActor, onEdit);
  fireEvent.click(screen.getByText("Relationships"));
  await choose("Relative to participant", "Guard");
  fireEvent.click(await screen.findByRole("button", { name: "Edit Hidden source" }));
  expect(onEdit).toHaveBeenCalledWith("subject", 9n);
  expect(onSave).not.toHaveBeenCalled();
});

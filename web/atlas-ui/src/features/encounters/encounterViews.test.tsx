import {
  detailFixture,
  summaryFixture,
  actorFixture,
  numberFact,
  knownFact,
  unavailableFact,
  activityFixture,
} from "../../test/fixtures";
import { App, ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { AtlasApiError } from "../../api/atlasApi";
import type { ReactNode } from "react";
import type {
  EncounterDetailView as EncounterDetailViewDto,
  EncounterIndexView as EncounterIndexViewDto,
  EncounterParticipantView,
  RecordDetailView,
  RecordPresentationView,
  RecordSummaryView,
  ResultWindowPage,
} from "../../generated/atlas";
import { EncounterDetailView } from "./EncounterDetailView";
import { EncounterEditView } from "./EncounterEditView";
import { EncounterIndexView } from "./EncounterIndexView";

const apiMocks = vi.hoisted(() => ({
  addEncounterManualParticipant: vi.fn(),
  addEncounterParticipantCondition: vi.fn(),
  addEncounterRecordParticipant: vi.fn(),
  createEncounter: vi.fn(),
  deleteEncounter: vi.fn(),
  getEncounter: vi.fn(),
  getEncounterConditionDefinitions: vi.fn(),
  getEncounters: vi.fn(),
  getRecordDetail: vi.fn(),
  getEncounterReference: vi.fn(),
  getOwnedReading: vi.fn(),
  openResultWindow: vi.fn(),
  removeEncounterParticipant: vi.fn(),
  removeEncounterParticipantCondition: vi.fn(),
  reorderEncounterParticipant: vi.fn(),
  setEncounterTurn: vi.fn(),
  updateEncounter: vi.fn(),
  updateEncounterParticipant: vi.fn(),
  updateEncounterParticipantCondition: vi.fn(),
}));

vi.mock("../../api/atlasApi", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../api/atlasApi")>()),
  addEncounterManualParticipant: apiMocks.addEncounterManualParticipant,
  addEncounterParticipantCondition: apiMocks.addEncounterParticipantCondition,
  addEncounterRecordParticipant: apiMocks.addEncounterRecordParticipant,
  createEncounter: apiMocks.createEncounter,
  deleteEncounter: apiMocks.deleteEncounter,
  getEncounter: apiMocks.getEncounter,
  getEncounterConditionDefinitions: apiMocks.getEncounterConditionDefinitions,
  getEncounters: apiMocks.getEncounters,
  getRecordDetail: apiMocks.getRecordDetail,
  getEncounterReference: apiMocks.getEncounterReference,
  getOwnedReading: apiMocks.getOwnedReading,
  openResultWindow: apiMocks.openResultWindow,
  removeEncounterParticipant: apiMocks.removeEncounterParticipant,
  removeEncounterParticipantCondition: apiMocks.removeEncounterParticipantCondition,
  reorderEncounterParticipant: apiMocks.reorderEncounterParticipant,
  setEncounterTurn: apiMocks.setEncounterTurn,
  updateEncounter: apiMocks.updateEncounter,
  updateEncounterParticipant: apiMocks.updateEncounterParticipant,
  updateEncounterParticipantCondition: apiMocks.updateEncounterParticipantCondition,
}));

let serverEncounter: EncounterDetailViewDto;
function setEncounterDetail(view: EncounterDetailViewDto) {
  serverEncounter = view;
  apiMocks.getEncounter.mockImplementation(() => Promise.resolve(serverEncounter));
}

describe("encounter views", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    history.replaceState(null, "", "/encounters");
    apiMocks.getEncounters.mockResolvedValue(encounterIndexFixture());
    setEncounterDetail(encounterDetailFixture());
    apiMocks.getEncounterConditionDefinitions.mockResolvedValue(
      conditionDefinitionsFixture(),
    );
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) =>
      Promise.resolve(recordDetailFixture(recordKey)),
    );
    apiMocks.openResultWindow.mockResolvedValue(resultWindowFixture());
    const receipt = {
      encounter_key: encounterIndexFixture().encounters[0].encounter_key,
    };
    for (const mutation of [
      apiMocks.setEncounterTurn,
      apiMocks.updateEncounter,
      apiMocks.updateEncounterParticipant,
      apiMocks.addEncounterRecordParticipant,
      apiMocks.addEncounterManualParticipant,
      apiMocks.reorderEncounterParticipant,
      apiMocks.removeEncounterParticipant,
      apiMocks.addEncounterParticipantCondition,
      apiMocks.updateEncounterParticipantCondition,
      apiMocks.removeEncounterParticipantCondition,
    ])
      mutation.mockResolvedValue(receipt);
    apiMocks.updateEncounterParticipant.mockImplementation((_ref, request) => {
      serverEncounter = {
        ...serverEncounter,
        participants: serverEncounter.participants.map((p) =>
          p.participant_key === request.participant_key
            ? { ...p, ...request, revision: p.revision + 1n }
            : p,
        ),
      };
      return Promise.resolve(receipt);
    });
    apiMocks.deleteEncounter.mockResolvedValue({
      ...receipt,
      slug: "ambush",
      deleted: true,
    });
  });

  it("renders active encounter rows, hides archived rows, and routes to edit", async () => {
    render(<EncounterIndexView route={{ kind: "encounters" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByText("Ambush")).toBeInTheDocument();
    expect(screen.queryByText("Old Fight")).not.toBeInTheDocument();

    const encounterRow = screen.getByText("Ambush").closest("tr");
    if (!encounterRow) {
      throw new Error("encounter row was not rendered");
    }
    fireEvent.click(encounterRow);
    await waitFor(() => expect(window.location.pathname).toBe("/encounters/ambush"));

    history.replaceState(null, "", "/encounters");
    fireEvent.click(screen.getByRole("checkbox", { name: "Archived" }));
    expect(await screen.findByText("Old Fight")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Edit Ambush" }));
    await waitFor(() =>
      expect(window.location.pathname).toBe("/encounters/ambush/edit"),
    );
  });

  it("edits encounter metadata without exposing slug", async () => {
    render(<EncounterEditView route={{ kind: "encounterEdit", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByDisplayValue("Ambush")).toBeInTheDocument();
    expect(screen.queryByLabelText("Slug")).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Renamed Ambush" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(apiMocks.updateEncounter).toHaveBeenCalled());
    expect(apiMocks.updateEncounter.mock.calls[0][0]).toEqual(
      expect.objectContaining({
        encounter_key: "encounter_ambush",
        slug: "ambush",
        name: "Renamed Ambush",
      }),
    );
  });

  it("selects roster participants without setting the turn, then supports explicit turn", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const kyraRow = (await screen.findByText("Kyra")).closest('[role="button"]');
    if (!kyraRow) {
      throw new Error("Kyra roster row was not rendered");
    }
    fireEvent.click(kyraRow);
    await screen.findByText("PC");
    expect(apiMocks.setEncounterTurn).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Set turn to Kyra" }));
    await waitFor(() =>
      expect(apiMocks.setEncounterTurn).toHaveBeenCalledWith({
        encounter_ref: "encounter_ambush",
        participant_key: "participant_b",
      }),
    );
  });

  it("advances turns from the roster play control", async () => {
    const { unmount } = render(
      <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />,
      {
        wrapper: queryClientWrapper(),
      },
    );

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Next" })).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Next" }));
    await waitFor(() =>
      expect(apiMocks.setEncounterTurn).toHaveBeenCalledWith({
        encounter_ref: "encounter_ambush",
      }),
    );

    unmount();
    vi.clearAllMocks();
    setEncounterDetail({
      ...encounterDetailFixture(),
      current_turn_participant_key: undefined,
    });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Play" })).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Play" }));
    await waitFor(() =>
      expect(apiMocks.setEncounterTurn).toHaveBeenCalledWith({
        encounter_ref: "encounter_ambush",
      }),
    );
  });

  it("selects the roster by keyboard and commits initiative without setting turn", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await screen.findByText("Kyra");
    const kyra = rosterRow("Kyra");
    kyra.focus();
    fireEvent.keyDown(kyra, { key: "Enter" });
    expect(await screen.findByLabelText("HP")).toHaveValue("24");
    expect(apiMocks.setEncounterTurn).not.toHaveBeenCalled();
    const initiative = within(kyra).getByLabelText("Kyra initiative");
    fireEvent.change(initiative, { target: { value: "18" } });
    fireEvent.keyDown(initiative, { key: "Enter" });
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_b",
          initiative: 18n,
          hp_edit: false,
          variant_edit: false,
          max_hp_edit: false,
        }),
      ),
    );
    expect(apiMocks.setEncounterTurn).not.toHaveBeenCalled();
  });

  it("marks the current roster row and omits max HP from roster text", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await screen.findByText("Initiative set");
    const goblinRow = rosterRow("Goblin");
    expect(goblinRow).toHaveClass("encounter-roster__row--current");
    expect(within(goblinRow).getByText("Current turn")).toBeInTheDocument();
    expect(within(goblinRow).getByLabelText("Goblin current HP")).toHaveValue("10");
    expect(within(goblinRow).queryByText("/ 12")).not.toBeInTheDocument();
  });

  it("renders runtime action budget and speed projections", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await screen.findByText("Actions");
    const runtimeSection = screen
      .getByText("Actions")
      .closest(".actor-reference__combat-budget");
    if (!(runtimeSection instanceof HTMLElement)) {
      throw new Error("Combat action budget was not rendered");
    }

    expect(within(runtimeSection).getAllByText("Actions").length).toBeGreaterThan(0);
    expect(within(runtimeSection).getByText("2")).toBeInTheDocument();
    expect(
      within(runtimeSection).getByLabelText("Show explanation for Actions"),
    ).toBeInTheDocument();
    expect(within(runtimeSection).getAllByText("Reactions").length).toBeGreaterThan(0);
    const movementSection = screen.getByText("Speed").closest(".record-fact-list");
    if (!(movementSection instanceof HTMLElement)) {
      throw new Error("Movement section was not rendered");
    }
    expect(within(movementSection).getAllByText("Speed").length).toBeGreaterThan(0);
    expect(within(movementSection).getByText("15 feet")).toBeInTheDocument();
    expect(
      within(movementSection).getByLabelText("Show explanation for Speed"),
    ).toBeInTheDocument();
    const activitiesSection = screen
      .getByText("Strikes")
      .closest(".actor-reference__activities");
    if (!(activitiesSection instanceof HTMLElement)) {
      throw new Error("Activities section was not rendered");
    }
    expect(within(activitiesSection).getByText("Claw")).toBeInTheDocument();
    expect(within(activitiesSection).getByText("1d6+2")).toBeInTheDocument();
    expect(within(activitiesSection).getByText(/slashing/)).toBeInTheDocument();
  });

  it("renders manual PC runtime state without inferred speed rows", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click((await screen.findByText("Kyra")).closest('[role="button"]')!);

    await screen.findByText("Actions");
    const runtimeSection = screen
      .getByText("Actions")
      .closest(".actor-reference__combat-budget");
    if (!(runtimeSection instanceof HTMLElement)) {
      throw new Error("Combat action budget was not rendered");
    }
    expect(within(runtimeSection).getAllByText("Actions").length).toBeGreaterThan(0);
    expect(within(runtimeSection).getByText("3")).toBeInTheDocument();
    expect(within(runtimeSection).getAllByText("Reactions").length).toBeGreaterThan(0);
    expect(screen.queryByText("Speed")).not.toBeInTheDocument();
  });

  it("edits the selected participant in the participant sheet", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const kyraRow = (await screen.findByText("Kyra")).closest('[role="button"]');
    if (!kyraRow) {
      throw new Error("Kyra roster row was not rendered");
    }
    fireEvent.click(kyraRow);

    const hpInput = await screen.findByLabelText("HP");
    expect(hpInput).toHaveValue("24");
    fireEvent.change(hpInput, { target: { value: "20" } });
    fireEvent.keyDown(hpInput, { key: "Enter" });

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_b",
          current_hp: 20n,
        }),
      ),
    );
  });

  it("starts with combat controls and keeps detailed edits in keyboard disclosures", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const hp = await screen.findByLabelText("HP");
    expect(screen.getByRole("button", { name: "Damage" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Heal" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add Condition" })).toBeInTheDocument();
    expect(screen.queryByLabelText("Participant name")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Maximum HP")).not.toBeInTheDocument();
    expect(
      screen.queryByText("Derived maximum; current HP edited"),
    ).not.toBeInTheDocument();

    const details = screen.getByRole("button", { name: "Edit details" });
    expect(details).toHaveAttribute("aria-expanded", "false");
    expect(
      hp.compareDocumentPosition(screen.getByText("Strikes")) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    details.focus();
    fireEvent.keyDown(details, { key: "Enter", keyCode: 13 });
    expect(await screen.findByLabelText("Participant name")).toHaveValue("Goblin");
    expect(details).toHaveAttribute("aria-expanded", "true");

    const name = screen.getByLabelText("Participant name");
    fireEvent.change(name, { target: { value: "Goblin Scout" } });
    fireEvent.keyDown(name, { key: "Enter" });
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          display_name: "Goblin Scout",
          hp_edit: false,
          max_hp_edit: false,
          variant_edit: false,
        }),
      ),
    );
    expect(screen.getByLabelText("Participant side")).toBeInTheDocument();
  });

  it("preserves maximum override and adjustment intent through secondary controls", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const hpDetails = await screen.findByRole("button", { name: "HP details" });
    hpDetails.focus();
    fireEvent.keyDown(hpDetails, { key: "Enter", keyCode: 13 });
    const maximum = await screen.findByLabelText("Maximum HP");
    fireEvent.change(maximum, { target: { value: "30" } });
    fireEvent.click(screen.getByRole("button", { name: "Override maximum" }));
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          max_hp: 30n,
          max_hp_edit: true,
          hp_edit: false,
          variant_edit: false,
        }),
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "Use authored maximum" }));
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          use_derived_max: true,
          max_hp_edit: false,
        }),
      ),
    );

    await selectOption(conditionCombobox("Variant"), "Elite");
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          participant_variant: "elite",
          variant_edit: true,
          hp_edit: false,
          max_hp_edit: false,
        }),
      ),
    );
  });

  it("reloads saved current HP without reapplying edit intent", async () => {
    const { unmount } = render(
      <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />,
      {
        wrapper: queryClientWrapper(),
      },
    );
    const hp = await screen.findByLabelText("HP");
    fireEvent.change(hp, { target: { value: "8" } });
    fireEvent.keyDown(hp, { key: "Enter" });
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          current_hp: 8n,
          hp_edit: true,
          max_hp_edit: false,
          variant_edit: false,
        }),
      ),
    );
    unmount();
    setEncounterDetail(
      encounterDetailFixture("participant_a", {
        current_hp: 8n,
        hp_origin: "derived_edited",
      }),
    );
    apiMocks.updateEncounterParticipant.mockClear();
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    expect(await screen.findByLabelText("HP")).toHaveValue("8");
    expect(apiMocks.updateEncounterParticipant).not.toHaveBeenCalled();
    expect(within(rosterRow("Goblin")).getByLabelText("Goblin current HP")).toHaveValue(
      "8",
    );
  });

  it("resets participant disclosures and drafts when selecting another participant", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "HP details" }));
    fireEvent.change(await screen.findByLabelText("Maximum HP"), {
      target: { value: "999" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Edit details" }));
    fireEvent.click(rosterRow("Kyra"));
    expect(await screen.findByLabelText("HP")).toHaveValue("24");
    expect(screen.queryByLabelText("Participant name")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Maximum HP")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "HP details" }));
    expect(await screen.findByLabelText("Maximum HP")).toHaveValue("24");
    expect(apiMocks.updateEncounterParticipant).not.toHaveBeenCalled();
  });

  it("keeps encounter lifecycle actions in the edit modal", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Edit encounter" })).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Edit encounter" }));
    const dialog = await screen.findByRole("dialog", { name: "Edit encounter" });
    fireEvent.click(within(dialog).getByText("Complete"));

    await waitFor(() => expect(apiMocks.updateEncounter).toHaveBeenCalled());
    expect(apiMocks.updateEncounter.mock.calls[0][0]).toEqual(
      expect.objectContaining({
        encounter_key: "encounter_ambush",
        slug: "ambush",
        status: "complete",
      }),
    );
  });

  it("opens and dismisses linked record previews inside the encounter record pane", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(await screen.findByText("Source content"));
    const linkedRuleButton = (await screen.findByText("Linked Rule")).closest(
      "[role=button]",
    );
    if (!linkedRuleButton) {
      throw new Error("Linked Rule button was not rendered");
    }
    fireEvent.click(linkedRuleButton);

    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith("rules:linked"),
    );
    expect(await screen.findByLabelText("Reference preview")).toBeInTheDocument();

    const kyraRow = (await screen.findByText("Kyra")).closest('[role="button"]');
    if (!kyraRow) {
      throw new Error("Kyra roster row was not rendered");
    }
    fireEvent.click(kyraRow);
    await waitFor(() =>
      expect(screen.queryByLabelText("Reference preview")).not.toBeInTheDocument(),
    );

    fireEvent.click(await screen.findByText("Goblin"));

    fireEvent.click(await screen.findByText("Source content"));
    const linkedRuleButtonAfterReselect = (
      await screen.findByText("Linked Rule")
    ).closest("[role=button]");
    if (!linkedRuleButtonAfterReselect) {
      throw new Error("Linked Rule button was not rendered after reselection");
    }
    fireEvent.click(linkedRuleButtonAfterReselect);
    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith("rules:linked"),
    );

    const closePreview = screen.getByRole("button", {
      name: "Close reference preview",
    });
    closePreview.focus();
    fireEvent.keyDown(closePreview, { key: "Escape" });

    await waitFor(() =>
      expect(screen.queryByLabelText("Reference preview")).not.toBeInTheDocument(),
    );
  });

  it("quick lookup renders complete participant detail and refetches after runtime updates", async () => {
    const initial = encounterDetailFixture();
    const activity = initial.participants[0].presentation!.body;
    if (activity.kind !== "creature") throw new Error("expected creature fixture");
    const claw = activity.value.activities.value![0];
    const selected = recordDetailFixture("actors:goblin");
    selected.selected = claw.navigation;
    selected.presentation.identity = { ...selected.record, title: claw.title };
    selected.presentation.body = { kind: "activity", value: claw };
    selected.presentation.content = [
      {
        locator: {
          record: "actors:goblin",
          owners: claw.navigation.owners,
          field: "/system/description/value",
        },
        role: "description",
        source_fingerprint: null,
        body: { kind: "plain", text: "Customized participant Claw prose" },
      },
    ];
    apiMocks.getEncounterReference.mockResolvedValue({
      ...selected,
      artifact_generation: initial.artifact_generation,
    });
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });
    render(
      <QueryClientProvider client={client}>
        <ConfigProvider theme={{ token: { motion: false } }}>
          <App>
            <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />
          </App>
        </ConfigProvider>
      </QueryClientProvider>,
    );
    fireEvent.click(await screen.findByRole("button", { name: "Claw" }));
    const preview = await screen.findByLabelText("Reference preview");
    await waitFor(() =>
      expect(
        within(preview).getByText("Customized participant Claw prose"),
      ).toBeVisible(),
    );
    expect(within(preview).getByText("+12")).toBeVisible();
    expect(apiMocks.getEncounterReference).toHaveBeenCalledWith(
      "encounter_ambush",
      "participant_a",
      expect.objectContaining({
        artifact_generation: initial.artifact_generation,
        selection: {
          record_key: claw.navigation.record_key,
          owners: claw.navigation.owners,
          fields: [],
          passage: claw.navigation.passage,
          source_fingerprint: claw.navigation.source_fingerprint,
        },
      }),
      expect.any(AbortSignal),
    );
    expect(apiMocks.getRecordDetail).not.toHaveBeenCalled();
    const updated = encounterDetailFixture();
    if (updated.participants[0].presentation!.body.kind !== "creature")
      throw new Error("expected creature fixture");
    updated.participants[0].presentation!.body.value.activities.value![0].attack =
      numberFact(19);
    updated.participants[0].presentation!.body.value.activities.value![0].notes = [
      {
        source: "Elite",
        label: "Elite",
        reason: "Current source adjustment retained.",
      },
    ];
    apiMocks.getEncounterReference.mockResolvedValue({
      ...selected,
      presentation: {
        ...selected.presentation,
        body: {
          kind: "activity",
          value: updated.participants[0].presentation!.body.value.activities.value![0],
        },
      },
    });
    act(() => client.setQueryData(["encounter", "ambush"], updated));
    await waitFor(() => expect(within(preview).getByText("+19")).toBeVisible());
    expect(within(preview).queryByText("+12")).toBeNull();
    expect(
      within(preview).getByText("Elite: Current source adjustment retained."),
    ).toBeVisible();
    expect(apiMocks.getEncounterReference).toHaveBeenCalledTimes(2);
    expect(apiMocks.getOwnedReading).not.toHaveBeenCalled();
    expect(apiMocks.getRecordDetail).not.toHaveBeenCalled();
  });

  it("opens condition reference previews from canonical condition rows", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(await screen.findByRole("button", { name: "Frightened" }));

    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        "conditionitems:TBSHQspnbcqxsmjL",
      ),
    );
    expect(await screen.findByLabelText("Reference preview")).toBeInTheDocument();
    expect(await screen.findByText("Frightened Condition")).toBeInTheDocument();
  });

  it("renders HP meter segments and threshold states", async () => {
    setEncounterDetail(
      encounterDetailFixture("participant_a", {
        current_hp: 6n,
        temporary_hp: 5n,
      }),
    );
    const { unmount } = render(
      <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />,
      {
        wrapper: queryClientWrapper(),
      },
    );

    const meter = await screen.findByLabelText("HP remaining: 6/12 +5");
    const currentSegment = meter.querySelector<HTMLElement>(
      ".encounter-hp-meter__current",
    );
    const missingSegment = meter.querySelector<HTMLElement>(
      ".encounter-hp-meter__missing",
    );
    const temporarySegment = meter.querySelector<HTMLElement>(
      ".encounter-hp-meter__temporary",
    );
    expect(currentSegment).not.toBeNull();
    expect(missingSegment).not.toBeNull();
    expect(temporarySegment).not.toBeNull();
    expect(currentSegment).toHaveClass("encounter-hp-meter__current--bloodied");
    expect(currentSegment).not.toHaveClass("encounter-hp-meter__current--critical");
    expect(currentSegment).toHaveStyle({ width: "35.294117647058826%" });
    expect(missingSegment).toHaveStyle({
      left: "35.294117647058826%",
      width: "35.294117647058826%",
    });
    expect(temporarySegment).toHaveStyle({
      left: "70.58823529411765%",
      width: "29.411764705882355%",
    });

    unmount();
    setEncounterDetail(
      encounterDetailFixture("participant_a", {
        current_hp: 3n,
        temporary_hp: 0n,
      }),
    );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const criticalMeter = await screen.findAllByLabelText(/HP remaining:/);
    expect(
      criticalMeter[criticalMeter.length - 1].querySelector(
        ".encounter-hp-meter__current",
      ),
    ).toHaveClass("encounter-hp-meter__current--critical");
  });

  it("applies HP formulas and consumes temporary HP before current HP", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const hpInput = await screen.findByLabelText("HP");
    expect(hpInput).toHaveValue("10");
    fireEvent.change(hpInput, { target: { value: "50 - 7" } });
    fireEvent.keyDown(hpInput, { key: "Enter" });

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          current_hp: 12n,
        }),
      ),
    );

    const tempHpInput = screen.getByLabelText("Temp HP");
    expect(tempHpInput).toHaveValue("5");
    fireEvent.change(tempHpInput, { target: { value: "4 + 2" } });
    fireEvent.keyDown(tempHpInput, { key: "Enter" });

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          temporary_hp: 6n,
          current_hp: 12n,
        }),
      ),
    );

    fireEvent.change(screen.getByLabelText("HP change"), { target: { value: "8" } });
    fireEvent.click(screen.getByRole("button", { name: "Damage" }));

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          temporary_hp: 0n,
          current_hp: 10n,
        }),
      ),
    );

    const hpChangeInput = screen.getByLabelText("HP change");
    fireEvent.change(hpChangeInput, { target: { value: "-3" } });
    fireEvent.keyDown(hpChangeInput, { key: "Enter" });

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          temporary_hp: 0n,
          current_hp: 7n,
        }),
      ),
    );

    fireEvent.change(hpChangeInput, { target: { value: "10" } });
    fireEvent.keyDown(hpChangeInput, { key: "Enter" });

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          current_hp: 12n,
        }),
      ),
    );
  });

  it("consumes temporary HP before current HP in participant controls", async () => {
    setEncounterDetail(
      encounterDetailFixture("participant_a", {
        presentation: participantPresentationFixture(),
      }),
    );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const hpChangeInput = await screen.findByLabelText("HP change");
    fireEvent.change(hpChangeInput, { target: { value: "8" } });
    fireEvent.click(screen.getByRole("button", { name: "Damage" }));

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          temporary_hp: 0n,
          current_hp: 7n,
        }),
      ),
    );

    fireEvent.change(hpChangeInput, { target: { value: "3" } });
    fireEvent.click(screen.getByRole("button", { name: "Heal" }));
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          current_hp: 10n,
          temporary_hp: 0n,
          hp_edit: true,
          variant_edit: false,
          max_hp_edit: false,
        }),
      ),
    );
  });

  it("searches before adding a record-backed participant", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Add Creature" })).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add Creature" }));
    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();

    const dialog = await screen.findByRole("dialog", {
      name: "Add creature or hazard",
    });
    fireEvent.change(within(dialog).getByLabelText("Search"), {
      target: { value: "goblin" },
    });
    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1));

    const goblinOption = await within(dialog).findByText("Goblin Warrior");
    fireEvent.click(goblinOption.closest('[role="button"]') ?? goblinOption);
    await screen.findByText("Selected: Goblin Warrior");
    fireEvent.change(within(dialog).getByLabelText("Quantity"), {
      target: { value: "2" },
    });
    fireEvent.change(within(dialog).getByLabelText("Initiative"), {
      target: { value: "18" },
    });
    fireEvent.click(screen.getByRole("button", { name: "OK" }));

    await waitFor(() =>
      expect(apiMocks.addEncounterRecordParticipant).toHaveBeenCalledWith({
        encounter_ref: "encounter_ambush",
        record_ref: "actors:goblin",
        quantity: 2,
        initiative: 18n,
      }),
    );
  }, 10_000);

  it("adds conditions with compact fields and details", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await screen.findByText("Frightened");

    fireEvent.click(screen.getByRole("button", { name: "Add Condition" }));
    await selectOption(conditionCombobox("Add condition"), "Sickened");
    expect(await screen.findByLabelText("Condition value")).toHaveValue("1");
    fireEvent.change(screen.getByLabelText("Condition value"), {
      target: { value: "2" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Condition details" }));
    fireEvent.change(await screen.findByLabelText("Duration rounds"), {
      target: { value: "3" },
    });
    fireEvent.change(screen.getByLabelText("Condition note"), {
      target: { value: "poison" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() =>
      expect(apiMocks.addEncounterParticipantCondition).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          condition_ref: "conditionitems:fesd1n5eVhpCSS18",
          value: 2n,
          duration_rounds: 3n,
          note: "poison",
        }),
      ),
    );
    expect(
      apiMocks.addEncounterParticipantCondition.mock.calls[0][1],
    ).not.toHaveProperty("name");
  }, 10_000);

  it("edits and removes conditions for the current participant", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await screen.findByText("Frightened");

    const frightenedValue = screen.getByLabelText("Frightened value");
    fireEvent.change(frightenedValue, {
      target: { value: "2" },
    });
    fireEvent.blur(frightenedValue);

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipantCondition).toHaveBeenCalledWith(
        "encounter_ambush",
        "participant_a",
        expect.objectContaining({
          condition_id: 7n,
          name: "Frightened",
          value: 2n,
        }),
      ),
    );
    expect(
      apiMocks.updateEncounterParticipantCondition.mock.calls[0][2],
    ).not.toHaveProperty("condition_key");

    fireEvent.click(screen.getByRole("button", { name: "Edit Frightened details" }));
    fireEvent.change(lastFieldByAriaLabel("Duration rounds"), {
      target: { value: "4" },
    });
    fireEvent.change(lastFieldByAriaLabel("Condition note"), {
      target: { value: "aura" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipantCondition).toHaveBeenCalledWith(
        "encounter_ambush",
        "participant_a",
        expect.objectContaining({
          condition_id: 7n,
          name: "Frightened",
          duration_rounds: 4n,
          note: "aura",
        }),
      ),
    );

    const frightenedRow = screen
      .getByText("Frightened")
      .closest(".encounter-condition-row");
    expect(frightenedRow).not.toBeNull();
    fireEvent.click(
      within(frightenedRow as HTMLElement).getByLabelText("Remove Frightened"),
    );

    await waitFor(() =>
      expect(apiMocks.removeEncounterParticipantCondition).toHaveBeenCalledWith(
        "encounter_ambush",
        "participant_a",
        7n,
      ),
    );
  }, 10_000);
  it("keeps server HP truthful, disables duplicate edits, and retains failed HP input", async () => {
    const pending = deferred<{ encounter_key: string }>();
    apiMocks.updateEncounterParticipant.mockReturnValue(pending.promise);
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const hp = await screen.findByLabelText("HP");
    fireEvent.change(hp, { target: { value: "3" } });
    fireEvent.keyDown(hp, { key: "Enter" });
    await waitFor(() => expect(hp).toBeDisabled());
    expect(screen.getByText("10/12 +5")).toBeInTheDocument();
    expect(screen.getByLabelText("Goblin initiative")).toBeDisabled();
    expect(screen.getByRole("button", { name: "Next" })).toBeDisabled();
    expect(rosterRow("Goblin")).toHaveAttribute("draggable", "false");
    fireEvent.keyDown(hp, { key: "Enter" });
    fireEvent.drop(rosterRow("Kyra"));
    await act(async () => pending.reject(new Error("Write unavailable")));
    expect(await screen.findByText("Write unavailable")).toBeInTheDocument();
    expect(hp).toHaveValue("3");
    expect(screen.getByText("10/12 +5")).toBeInTheDocument();
    expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledOnce();
    expect(apiMocks.reorderEncounterParticipant).not.toHaveBeenCalled();
  });

  it("retains failed conflict drafts and requires an explicit new edit", async () => {
    serverEncounter = encounterDetailFixture("participant_a", {
      current_hp: 9n,
      revision: 1n,
    });
    apiMocks.getEncounter
      .mockResolvedValueOnce(encounterDetailFixture())
      .mockImplementation(() => Promise.resolve(serverEncounter));
    apiMocks.updateEncounterParticipant.mockRejectedValue(
      new AtlasApiError(409, "Stale", {
        code: "encounter_participant_conflict",
        message: "Stale",
        retryable: false,
      }),
    );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const hp = await screen.findByLabelText("HP");
    fireEvent.change(hp, { target: { value: "3" } });
    fireEvent.keyDown(hp, { key: "Enter" });
    expect(await screen.findByText(/Participant changed. Review/)).toBeInTheDocument();
    expect(hp).toHaveValue("3");
    expect(screen.getByText("9/12 +5")).toBeInTheDocument();
    expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledOnce();
    expect(apiMocks.getEncounter).toHaveBeenCalledTimes(2);
  });

  it("shows committed refresh failure and retries the read without another write", async () => {
    apiMocks.getEncounter
      .mockResolvedValueOnce(encounterDetailFixture())
      .mockRejectedValueOnce(new Error("Artifact unavailable"))
      .mockResolvedValue(
        encounterDetailFixture("participant_a", { current_hp: 3n, revision: 1n }),
      );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const hp = await screen.findByLabelText("HP");
    fireEvent.change(hp, { target: { value: "3" } });
    fireEvent.keyDown(hp, { key: "Enter" });
    expect(
      await screen.findByText("Saved, but the encounter could not refresh"),
    ).toBeInTheDocument();
    expect(hp).toBeDisabled();
    expect(screen.getByText("10/12 +5")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Retry encounter refresh" }));
    await waitFor(() => expect(hp).not.toBeDisabled());
    expect(hp).toHaveValue("3");
    expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledOnce();
  });

  it("keeps delayed and rejected Save open with its draft", async () => {
    const saving = deferred<{ encounter_key: string }>();
    apiMocks.updateEncounter.mockReturnValue(saving.promise);
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() =>
      expect(screen.getByLabelText("Edit encounter")).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByLabelText("Edit encounter"));
    const dialog = await screen.findByRole("dialog", { name: "Edit encounter" });
    const name = within(dialog).getByLabelText("Name");
    fireEvent.change(name, { target: { value: "Unsaved ambush" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(name).toBeDisabled());
    expect(dialog).toBeInTheDocument();
    await act(async () => saving.reject(new Error("Save unavailable")));
    await waitFor(() => expect(name).not.toBeDisabled());
    expect(name).toHaveValue("Unsaved ambush");
    expect(within(dialog).getByText("Save unavailable")).toBeInTheDocument();
    apiMocks.updateEncounter.mockResolvedValue({ encounter_key: "encounter_ambush" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Edit encounter" }),
      ).not.toBeInTheDocument(),
    );
  });

  it("preserves touched encounter fields across changed server data and resets identity", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>
        <ConfigProvider theme={{ token: { motion: false } }}>
          <App component={false}>{children}</App>
        </ConfigProvider>
      </QueryClientProvider>
    );
    const { rerender } = render(
      <EncounterEditView route={{ kind: "encounterEdit", slug: "ambush" }} />,
      { wrapper },
    );
    const name = await screen.findByLabelText("Name");
    fireEvent.change(name, { target: { value: "My draft" } });
    act(() =>
      client.setQueryData(["encounter", "ambush"], {
        ...encounterDetailFixture(),
        note: "Server note",
        encounter: { ...encounterIndexFixture().encounters[0], name: "Server name" },
      }),
    );
    expect(name).toHaveValue("My draft");
    await waitFor(() =>
      expect(screen.getByLabelText("Note")).toHaveValue("Server note"),
    );
    const other = {
      ...encounterDetailFixture(),
      encounter: { ...encounterIndexFixture().encounters[1], name: "Other encounter" },
    };
    setEncounterDetail(other);
    rerender(<EncounterEditView route={{ kind: "encounterEdit", slug: "other" }} />);
    expect(await screen.findByLabelText("Name")).toHaveValue("Other encounter");
  });

  it("reports and retries initial encounter and condition catalog failures", async () => {
    apiMocks.getEncounter.mockRejectedValueOnce(new Error("Encounter unavailable"));
    apiMocks.getEncounterConditionDefinitions.mockRejectedValueOnce(
      new Error("Catalog unavailable"),
    );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    expect(await screen.findByText("Encounter could not load")).toBeInTheDocument();
    expect(screen.getByLabelText("Edit encounter")).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Retry encounter" }));
    expect(await screen.findByText("Conditions could not load")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add Condition" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Retry conditions" }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Add Condition" })).not.toBeDisabled(),
    );
  });

  it("does not carry an old route's pending failure into a different encounter", async () => {
    const pending = deferred<{ encounter_key: string }>();
    apiMocks.updateEncounterParticipant.mockReturnValue(pending.promise);
    const { rerender } = render(
      <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />,
      { wrapper: queryClientWrapper() },
    );
    const hp = await screen.findByLabelText("HP");
    fireEvent.change(hp, { target: { value: "3" } });
    fireEvent.keyDown(hp, { key: "Enter" });
    await waitFor(() => expect(hp).toBeDisabled());
    setEncounterDetail({
      ...encounterDetailFixture(),
      encounter: { ...encounterIndexFixture().encounters[1], name: "Other" },
    });
    rerender(<EncounterDetailView route={{ kind: "encounter", slug: "other" }} />);
    await waitFor(() => expect(screen.getByLabelText("HP")).not.toBeDisabled());
    await act(async () => pending.reject(new Error("Old failure")));
    expect(screen.queryByText("Old failure")).not.toBeInTheDocument();
    expect(screen.getByLabelText("HP")).toHaveValue("10");
  });

  it("keeps rejected index deletions open and waits for success", async () => {
    const deleting = deferred<{
      encounter_key: string;
      slug: string;
      deleted: boolean;
    }>();
    apiMocks.deleteEncounter.mockReturnValueOnce(deleting.promise);
    render(<EncounterIndexView route={{ kind: "encounters" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByLabelText("Delete Ambush"));
    const dialog = await screen.findByRole("dialog", { name: "Delete Ambush?" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete" }));
    await waitFor(() =>
      expect(apiMocks.deleteEncounter).toHaveBeenCalledExactlyOnceWith(
        "encounter_ambush",
      ),
    );
    expect(dialog).toBeInTheDocument();
    expect(screen.getByLabelText("Delete Ambush")).toBeDisabled();
    await act(async () => deleting.reject(new Error("Delete unavailable")));
    await waitFor(() =>
      expect(screen.getByLabelText("Delete Ambush")).not.toBeDisabled(),
    );
    expect(dialog).toBeInTheDocument();
    expect(screen.getAllByText("Delete unavailable").length).toBeGreaterThan(0);
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  });

  it("preserves a failed add-PC draft until a successful receipt", async () => {
    const adding = deferred<{ encounter_key: string }>();
    apiMocks.addEncounterManualParticipant.mockReturnValueOnce(adding.promise);
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Add PC" })).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add PC" }));
    const dialog = await screen.findByRole("dialog", { name: "Add PC" });
    const name = within(dialog).getByLabelText("Name");
    fireEvent.change(name, { target: { value: "My PC" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "OK" }));
    await waitFor(() => expect(name).toBeDisabled());
    await act(async () => adding.reject(new Error("Add unavailable")));
    await waitFor(() => expect(name).not.toBeDisabled());
    expect(name).toHaveValue("My PC");
    expect(within(dialog).getByText("Add unavailable")).toBeInTheDocument();
    fireEvent.click(within(dialog).getByRole("button", { name: "OK" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Add PC" })).not.toBeInTheDocument(),
    );
  });

  it("keeps failed condition details open and preserves the note", async () => {
    apiMocks.updateEncounterParticipantCondition.mockRejectedValueOnce(
      new Error("Condition unavailable"),
    );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByLabelText("Edit Frightened details"));
    const note = await screen.findByLabelText("Condition note");
    fireEvent.change(note, { target: { value: "My note" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(screen.getAllByText("Condition unavailable").length).toBeGreaterThan(0),
    );
    expect(note).toHaveValue("My note");
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
  });
  it("does not navigate an old successful Save over a newly selected editor", async () => {
    const saving = deferred<{ encounter_key: string }>();
    apiMocks.updateEncounter.mockReturnValueOnce(saving.promise);
    const original = encounterDetailFixture();
    const other = {
      ...encounterDetailFixture(),
      encounter: { ...encounterIndexFixture().encounters[1], name: "Other" },
    };
    apiMocks.getEncounter.mockImplementation((ref) =>
      Promise.resolve(ref === "other" ? other : original),
    );
    const { rerender } = render(
      <EncounterEditView route={{ kind: "encounterEdit", slug: "ambush" }} />,
      { wrapper: queryClientWrapper() },
    );
    fireEvent.change(await screen.findByLabelText("Name"), {
      target: { value: "Saved name" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(apiMocks.updateEncounter).toHaveBeenCalledOnce());
    history.replaceState(null, "", "/encounters/other/edit");
    rerender(<EncounterEditView route={{ kind: "encounterEdit", slug: "other" }} />);
    await waitFor(() => expect(screen.getByLabelText("Name")).toHaveValue("Other"));
    await act(async () => saving.resolve({ encounter_key: "encounter_ambush" }));
    await waitFor(() =>
      expect(apiMocks.getEncounter).toHaveBeenCalledWith("encounter_ambush"),
    );
    expect(window.location.pathname).toBe("/encounters/other/edit");
    expect(screen.getByLabelText("Name")).toHaveValue("Other");
  });
});

async function selectOption(input: HTMLElement, option: string) {
  fireEvent.mouseDown(input);
  const options = await screen.findAllByText(option);
  const visibleOption = options.find((element) =>
    element.closest(".atlas-select-option"),
  );
  fireEvent.click(visibleOption?.closest(".atlas-select-option") ?? options[0]);
}

function conditionCombobox(label: string): HTMLElement {
  return screen
    .getAllByLabelText(label)
    .find((element) => element.getAttribute("role") === "combobox")!;
}

function lastFieldByAriaLabel(label: string): HTMLInputElement | HTMLTextAreaElement {
  const fields = Array.from(
    document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>(
      `input[aria-label="${label}"], textarea[aria-label="${label}"]`,
    ),
  );
  const field = fields[fields.length - 1];
  if (!field) {
    throw new Error(`${label} field was not rendered`);
  }
  return field;
}

function rosterRow(displayName: string): HTMLElement {
  const row = screen
    .getAllByText(displayName)
    .map((element) => element.closest('[role="button"]'))
    .find((element): element is HTMLElement => element instanceof HTMLElement);
  if (!row) {
    throw new Error(`${displayName} roster row was not rendered`);
  }
  return row;
}

function queryClientWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <ConfigProvider
          theme={{ token: { motion: false } }}
          select={{ classNames: { popup: { listItem: "atlas-select-option" } } }}
        >
          <App component={false}>{children}</App>
        </ConfigProvider>
      </QueryClientProvider>
    );
  };
}

function encounterIndexFixture(): EncounterIndexViewDto {
  return {
    encounters: [
      {
        encounter_key: "encounter_ambush",
        slug: "ambush",
        name: "Ambush",
        description: "Road fight",
        status: "draft",
        round_number: 1n,
        participant_count: 2,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
      {
        encounter_key: "encounter_old",
        slug: "old-fight",
        name: "Old Fight",
        status: "archived",
        round_number: 1n,
        participant_count: 1,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
    ],
  };
}

function conditionDefinitionsFixture() {
  return {
    conditions: [
      conditionDefinitionFixture(
        "conditionitems:TBSHQspnbcqxsmjL",
        "Frightened",
        true,
        "automated",
      ),
      conditionDefinitionFixture(
        "conditionitems:fesd1n5eVhpCSS18",
        "Sickened",
        true,
        "automated",
      ),
      conditionDefinitionFixture(
        "conditionitems:AJh5ex99aV6VTggg",
        "Off-Guard",
        false,
        "automated",
      ),
      conditionDefinitionFixture(
        "conditionitems:j91X7x0XSomq8d60",
        "Prone",
        false,
        "tracked",
      ),
    ],
  };
}

function conditionDefinitionFixture(
  conditionRef: string,
  name: string,
  hasValue: boolean,
  automationLevel: "automated" | "tracked",
) {
  return {
    condition_ref: conditionRef,
    name,
    automation_level: automationLevel,
    applies_to: ["creature"],
    categories: [automationLevel === "automated" ? "stat_modifier" : "runtime_state"],
    has_value: hasValue,
    ...(hasValue ? { default_value: 1n } : {}),
  };
}

function encounterDetailFixture(
  currentTurnParticipantKey: string | undefined = "participant_a",
  firstParticipantOverrides: Partial<EncounterParticipantView> = {},
): EncounterDetailViewDto {
  return {
    artifact_generation: "test-generation",
    encounter: encounterIndexFixture().encounters[0],
    note: "Encounter note",
    current_turn_participant_key: currentTurnParticipantKey,
    participants: [
      participantFixture({
        participant_key: "participant_a",
        display_name: "Goblin",
        record_key: "actors:goblin",
        hp_origin: "explicit",
        variant_origin: "explicit",
        participant_kind: "creature",
        side: "enemy",
        initiative: 18n,
        max_hp: 12n,
        current_hp: 10n,
        temporary_hp: 5n,
        record: recordSummaryFixture("actors:goblin", "Goblin Warrior"),
        presentation: participantPresentationFixture({
          actions: 2n,
          actionBase: 3n,
          actionAdjustment: -1n,
          speed: 15n,
          speedBase: 25n,
          speedAdjustment: -10n,
        }),

        conditions: [
          {
            condition_id: 7n,
            condition_key: "conditionitems:TBSHQspnbcqxsmjL",
            name: "Frightened",
            value: 1n,
            duration_rounds: 2n,
            created_at: "2026-01-01T00:00:00Z",
            updated_at: "2026-01-01T00:00:00Z",
          },
        ],
        ...firstParticipantOverrides,
      }),
      participantFixture({
        participant_key: "participant_b",
        display_name: "Kyra",
        hp_origin: "explicit",
        variant_origin: "explicit",
        participant_kind: "pc",
        status: "manual",
        side: "pc",
        initiative: 15n,
        max_hp: 24n,
        current_hp: 24n,

        presentation: participantPresentationFixture({
          kind: "pc",
          kindLabel: "PC",
          levelLabel: undefined,
          recordKey: "participant_b",
          title: "Kyra",
          traits: [],
          actions: 3n,
          reactions: 1n,
        }),
      }),
    ],
  };
}

function participantFixture(
  overrides: Partial<EncounterParticipantView>,
): EncounterParticipantView {
  return {
    revision: 0n,
    participant_key: "participant",
    hp_origin: "explicit",
    variant_origin: "explicit",
    participant_kind: "creature",
    participant_variant: "normal",
    status: "active",
    position: 1n,
    display_name: "Participant",
    side: "enemy",
    initiative_order: 1n,
    temporary_hp: 0n,
    defeated: false,
    hidden: false,
    note_hint: null,
    conditions: [],
    ...overrides,
  };
}

function resultWindowFixture(): ResultWindowPage {
  return {
    window_id: 1n,
    mode: { kind: "text_search", query: "goblin", mode: "hybrid" },
    page: {
      number: 1,
      size: 25,
      count: 1,
      total: 1n,
      has_more: false,
      next_page: null,
    },
    coverage: null,
    rows: [
      {
        record: recordSummaryFixture("actors:goblin", "Goblin Warrior"),
        matches: [],
      },
    ],
  };
}

function recordSummaryFixture(recordKey: string, title: string): RecordSummaryView {
  return {
    ...summaryFixture(recordKey, title),
    kind: "creature",
    kind_label: "Creature",
    level_label: "1",
    pack: "Bestiary",
  };
}

function participantPresentationFixture({
  actionAdjustment,
  actionBase,
  actions,
  kind = "creature",
  kindLabel = "Creature",
  includeActivities = kind === "creature",
  levelLabel = "1",
  reactions = 1n,
  recordKey = "actors:goblin",
  speed,
  speedAdjustment,
  speedBase,
  title = "Goblin Warrior",
  traits = [
    { kind: "trait", label: "Goblin", value: "goblin" },
    { kind: "trait", label: "Humanoid", value: "humanoid" },
  ],
}: {
  actionAdjustment?: bigint;
  actionBase?: bigint;
  actions?: bigint;
  kind?: string;
  kindLabel?: string;
  includeActivities?: boolean;
  levelLabel?: string;
  reactions?: bigint;
  recordKey?: string;
  speed?: bigint;
  speedAdjustment?: bigint;
  speedBase?: bigint;
  title?: string;
  traits?: Array<{ kind: string; label: string; value: string }>;
} = {}) {
  const actor = actorFixture();
  actor.land_speed =
    speed === undefined
      ? { state: "not_applicable", value: null, adjustment: null }
      : numberFact(Number(speed));
  if (speedBase !== undefined)
    actor.land_speed.adjustment = {
      authored: Number(speedBase),
      applied:
        speedAdjustment === undefined
          ? []
          : [
              {
                source: "Encumbered",
                label: "Speed penalty",
                modifier_type: "status",
                value: speedAdjustment,
              },
            ],
      suppressed: [],
      notes: [],
    };
  const count = (label: string, value: bigint, base = value) => ({
    label,
    base_value: base,
    adjusted_value: value,
    segments: [],
    adjustments: [],
    suppressed_adjustments: [],
  });
  if (actions !== undefined)
    actor.runtime = {
      action_budget: {
        actions: {
          ...count("Actions", actions, actionBase),
          adjustments:
            actionAdjustment === undefined
              ? []
              : [
                  {
                    source: "Slowed 1",
                    label: "Reduced actions regained",
                    value: actionAdjustment,
                    reason: "Applied to the next action-regain step.",
                  },
                ],
        },
        reactions: count("Reactions", reactions),
        can_act: { available: true },
        can_react: { available: true },
        notes: [],
      },
      unapplied_effects: [],
    };
  actor.activities = knownFact(
    includeActivities
      ? [
          {
            ...activityFixture("Claw", recordKey),
            attack: numberFact(12),
            damage: knownFact([
              {
                id: "main",
                formula: knownFact("1d6+2"),
                damage_type: knownFact("slashing"),
                kinds: knownFact(["damage"]),
                category: unavailableFact("not_applicable"),
                materials: knownFact([]),
                apply_modifier: knownFact(false),
              },
            ]),
          },
        ]
      : [],
  );
  const presentation: RecordPresentationView = {
    classification: kindLabel,
    identity: {
      ...summaryFixture(recordKey, title),
      kind,
      kind_label: kindLabel,
      level_label: kind === "pc" ? null : (levelLabel ?? null),
      traits,
    },
    content: [],
    owned: [],
    body: { kind: "creature", value: actor },
  };
  return presentation;
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  const result = detailFixture(
    recordKey,
    recordKey.startsWith("conditionitems:") ? "Frightened Condition" : "Goblin Warrior",
    "rules:linked",
  );
  const body = result.presentation.content[0]?.body;
  if (body?.kind === "html")
    body.html = body.html.replace("Nested Rule", "Linked Rule");
  return result;
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

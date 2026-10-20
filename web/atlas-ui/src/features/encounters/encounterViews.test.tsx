import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import {
  detailFixture,
  summaryFixture,
  actorFixture,
  numberFact,
  knownFact,
  unavailableFact,
  activityFixture,
} from "../../test/fixtures";
import { App, ConfigProvider, Grid } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { retireImmutableReferenceQueries } from "../../shared/queries/immutableReferenceQueries";
import {
  act,
  cleanup,
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
  SearchResultsPage,
} from "../../generated/atlas";
import { EncounterDetailView } from "./EncounterDetailView";
import { EncounterEditView } from "./EncounterEditView";
import { EncounterIndexView } from "./EncounterIndexView";
import { EditEncounterForm } from "./EncounterModals";
import { atlasRoutePath, currentAtlasRoute } from "../../app/routes";

const apiMocks = vi.hoisted(() => ({
  applyEncounterHpAction: vi.fn(),
  getEncounterTurnReview: vi.fn(),
  advanceEncounterTurn: vi.fn(),
  addEncounterManualParticipant: vi.fn(),
  addEncounterParticipantCondition: vi.fn(),
  addEncounterRecordParticipant: vi.fn(),
  createEncounter: vi.fn(),
  deleteEncounter: vi.fn(),
  getEncounter: vi.fn(),
  getEncounterConditionDefinitions: vi.fn(),
  getEncounters: vi.fn(),
  getParties: vi.fn(),
  getRecordDetail: vi.fn(),
  getEncounterReference: vi.fn(),
  getEmbeddedReading: vi.fn(),
  searchPage: vi.fn(),
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
  applyEncounterHpAction: apiMocks.applyEncounterHpAction,
  getEncounterTurnReview: apiMocks.getEncounterTurnReview,
  advanceEncounterTurn: apiMocks.advanceEncounterTurn,
  addEncounterManualParticipant: apiMocks.addEncounterManualParticipant,
  addEncounterParticipantCondition: apiMocks.addEncounterParticipantCondition,
  addEncounterRecordParticipant: apiMocks.addEncounterRecordParticipant,
  createEncounter: apiMocks.createEncounter,
  deleteEncounter: apiMocks.deleteEncounter,
  getEncounter: apiMocks.getEncounter,
  getEncounterConditionDefinitions: apiMocks.getEncounterConditionDefinitions,
  getEncounters: apiMocks.getEncounters,
  getParties: apiMocks.getParties,
  getRecordDetail: apiMocks.getRecordDetail,
  getEncounterReference: apiMocks.getEncounterReference,
  getEmbeddedReading: apiMocks.getEmbeddedReading,
  searchPage: apiMocks.searchPage,
  removeEncounterParticipant: apiMocks.removeEncounterParticipant,
  removeEncounterParticipantCondition: apiMocks.removeEncounterParticipantCondition,
  reorderEncounterParticipant: apiMocks.reorderEncounterParticipant,
  setEncounterTurn: apiMocks.setEncounterTurn,
  updateEncounter: apiMocks.updateEncounter,
  updateEncounterParticipant: apiMocks.updateEncounterParticipant,
  updateEncounterParticipantCondition: apiMocks.updateEncounterParticipantCondition,
}));

let serverEncounter: EncounterDetailViewDto;
const testQueryClients = new Set<QueryClient>();
function setEncounterDetail(view: EncounterDetailViewDto) {
  serverEncounter = view;
  apiMocks.getEncounter.mockImplementation(() => Promise.resolve(serverEncounter));
}

describe("encounter views", () => {
  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.clearAllMocks();
    Object.values(apiMocks).forEach((mock) => mock.mockReset());
    apiMocks.getParties.mockResolvedValue({ parties: [] });
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ md: true, xl: true });
    history.replaceState(null, "", "/encounters");
    apiMocks.getEncounters.mockResolvedValue(encounterIndexFixture());
    setEncounterDetail(encounterDetailFixture());
    apiMocks.getEncounterConditionDefinitions.mockResolvedValue(
      conditionDefinitionsFixture(),
    );
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) =>
      Promise.resolve(recordDetailFixture(recordKey)),
    );
    apiMocks.searchPage.mockResolvedValue(searchResultsFixture());
    apiMocks.getEncounterTurnReview.mockImplementation(() =>
      Promise.resolve({
        encounter_key: "encounter_ambush",
        expected_revision: serverEncounter.revision,
        current_participant_key: serverEncounter.current_turn_participant_key ?? null,
        next_participant_key: "participant_b",
        reminders: [],
      }),
    );
    const receipt = {
      encounter_key: encounterIndexFixture().encounters[0].encounter_key,
    };
    for (const mutation of [
      apiMocks.applyEncounterHpAction,
      apiMocks.advanceEncounterTurn,
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

  afterEach(async () => {
    try {
      await disposeEncounterTest();
    } finally {
      // Timer spies capture the fake clock; restore them before restoring time.
      vi.restoreAllMocks();
      vi.useRealTimers();
    }
  });

  it("renders active encounter rows, hides archived rows, and routes to edit", async () => {
    render(<EncounterIndexView route={{ kind: "encounters" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByText("Ambush")).toBeInTheDocument();
    const indexTable = screen.getByRole("table");
    expect(indexTable).toHaveStyle({ tableLayout: "auto" });
    expect(indexTable.querySelector("colgroup col")).toHaveStyle({ minWidth: "240px" });
    expect(screen.queryByText("Old Fight")).not.toBeInTheDocument();

    const encounterRow = screen.getByText("Ambush").closest("tr");
    if (!encounterRow) {
      throw new Error("encounter row was not rendered");
    }
    fireEvent.click(encounterRow);
    await waitFor(() => expect(window.location.pathname).toBe("/encounters/ambush"));

    history.replaceState(null, "", "/encounters");
    fireEvent.click(screen.getByRole("tab", { name: "Archived" }));
    expect(await screen.findByText("Old Fight")).toBeInTheDocument();
    expect(screen.queryByText("Ambush")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("tab", { name: "Active" }));
    expect(await screen.findByText("Ambush")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("link", { name: "Edit Ambush" }));
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

    const kyraRow = (await screen.findByText("Kyra")).closest("button");
    if (!kyraRow) {
      throw new Error("Kyra roster row was not rendered");
    }
    fireEvent.click(kyraRow);
    await screen.findByText("PC");
    expect(apiMocks.setEncounterTurn).not.toHaveBeenCalled();

    fireEvent.click(rosterRow("Goblin").querySelector(".encounter-roster__select")!);
    await waitFor(() =>
      expect(screen.getByLabelText("HP summary")).toHaveTextContent("10/12"),
    );
    apiMocks.setEncounterTurn.mockImplementationOnce(async () => {
      serverEncounter = {
        ...serverEncounter,
        current_turn_participant_key: "participant_b",
      };
      return { encounter_key: "encounter_ambush" };
    });
    fireEvent.click(screen.getByRole("button", { name: "Actions for Kyra" }));
    fireEvent.click(await screen.findByRole("menuitem", { name: "Set turn to Kyra" }));
    await waitFor(() =>
      expect(apiMocks.setEncounterTurn).toHaveBeenCalledWith({
        encounter_ref: "encounter_ambush",
        participant_key: "participant_b",
      }),
    );
    await waitFor(() =>
      expect(screen.getByLabelText("HP summary")).toHaveTextContent("24/24"),
    );
    expect(rosterRow("Kyra")).toHaveClass("encounter-roster__row--selected");
    expect(rosterRow("Kyra")).toHaveClass("encounter-roster__row--current");
  });

  it("advances turns from the roster play control", async () => {
    const { unmount } = render(
      <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />,
      {
        wrapper: queryClientWrapper(),
      },
    );

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Next turn" })).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Next turn" }));
    await waitFor(() =>
      expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledWith("encounter_ambush", {
        expected_revision: 0n,
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
      expect(screen.getByRole("button", { name: "Start turn" })).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Start turn" }));
    await waitFor(() =>
      expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledWith("encounter_ambush", {
        expected_revision: 0n,
      }),
    );
  });

  it("reveals the canonical next participant at the top of the reader after refresh", async () => {
    vi.mocked(Grid.useBreakpoint).mockReturnValue({ xs: true, md: false, xl: false });
    apiMocks.advanceEncounterTurn.mockImplementation(async () => {
      serverEncounter = {
        ...serverEncounter,
        current_turn_participant_key: "participant_b",
      };
      return { encounter_key: "encounter_ambush" };
    });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Next turn" })).toBeEnabled(),
    );
    fireEvent.click(document.querySelector('[data-participant-key="participant_a"]')!);
    const back = await screen.findByRole("button", { name: "Back to roster" });
    const reader = document.querySelector<HTMLElement>(".encounter-record-pane")!;
    reader.scrollTop = 883;
    fireEvent.click(
      within(back.parentElement!).getByRole("button", { name: "Next turn" }),
    );
    await waitFor(() =>
      expect(screen.getByLabelText("HP summary")).toHaveTextContent("24/24"),
    );
    expect(reader.scrollTop).toBe(0);
    expect(within(reader).getByRole("heading", { name: "Kyra" })).toBeVisible();
    expect(rosterRow("Kyra")).toHaveClass("encounter-roster__row--selected");
  });

  it("returns to the original roster position after scrolling the participant and advancing turn", async () => {
    vi.mocked(Grid.useBreakpoint).mockReturnValue({ xs: true, md: false, xl: false });
    const scrollTo = vi.spyOn(window, "scrollTo").mockImplementation(() => {});
    const scrollY = vi.spyOn(window, "scrollY", "get").mockReturnValue(160);
    try {
      apiMocks.advanceEncounterTurn.mockImplementation(async () => {
        serverEncounter = {
          ...serverEncounter,
          current_turn_participant_key: "participant_b",
        };
        return { encounter_key: "encounter_ambush" };
      });
      const { container } = render(
        <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />,
        { wrapper: queryClientWrapper() },
      );
      await screen.findByLabelText("HP summary");
      const roster = container.querySelector<HTMLElement>(".encounter-roster")!;
      roster.scrollTop = 85;
      fireEvent.click(roster.querySelector('[data-participant-key="participant_a"]')!);
      const back = await screen.findByRole("button", { name: "Back to roster" });
      scrollY.mockReturnValue(940);
      roster.scrollTop = 0;
      container.querySelector<HTMLElement>(".encounter-record-pane")!.scrollTop = 600;
      fireEvent.click(
        within(back.parentElement!).getByRole("button", { name: "Next turn" }),
      );
      await waitFor(() =>
        expect(screen.getByLabelText("HP summary")).toHaveTextContent("24/24"),
      );
      fireEvent.click(back);
      expect(scrollTo).toHaveBeenLastCalledWith({ top: 160 });
      expect(roster.scrollTop).toBe(85);
      expect(
        roster.querySelector('[data-participant-key="participant_b"]'),
      ).toHaveFocus();
    } finally {
      scrollY.mockRestore();
      scrollTo.mockRestore();
    }
  });

  it("follows the server current participant when a committed Next turn refresh is retried", async () => {
    vi.mocked(Grid.useBreakpoint).mockReturnValue({ xs: true, md: false, xl: false });
    const initial = encounterDetailFixture();
    apiMocks.getEncounter
      .mockResolvedValueOnce(initial)
      .mockRejectedValueOnce(new Error("Artifact unavailable"))
      .mockResolvedValue({ ...initial, current_turn_participant_key: "participant_b" });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Next turn" })).toBeEnabled(),
    );
    fireEvent.click(document.querySelector('[data-participant-key="participant_a"]')!);
    const back = await screen.findByRole("button", { name: "Back to roster" });
    fireEvent.click(
      within(back.parentElement!).getByRole("button", { name: "Next turn" }),
    );
    await screen.findByText("Saved, but the encounter could not refresh");
    expect(screen.getByLabelText("HP summary")).toHaveTextContent("10/12");
    fireEvent.click(screen.getByRole("button", { name: "Retry encounter refresh" }));
    await waitFor(() =>
      expect(screen.getByLabelText("HP summary")).toHaveTextContent("24/24"),
    );
    expect(rosterRow("Kyra")).toHaveClass("encounter-roster__row--selected");
    expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledOnce();
    expect(apiMocks.getEncounter).toHaveBeenCalledTimes(3);
  });

  it("reorders from the keyboard drag handle using the existing participant reorder contract", async () => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(
      function (this: HTMLElement) {
        const top = this.dataset.participantKey === "participant_b" ? 100 : 0;
        return {
          x: 0,
          y: top,
          top,
          bottom: top + 80,
          left: 0,
          right: 300,
          width: 300,
          height: 80,
          toJSON() {},
        };
      },
    );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const handle = await screen.findByRole("button", { name: "Reorder Kyra" });
    handle.focus();
    fireEvent.keyDown(handle, { code: "Space", key: " " });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });
    fireEvent.keyDown(document, { code: "ArrowUp", key: "ArrowUp" });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });
    fireEvent.keyDown(document, { code: "Space", key: " " });
    await waitFor(() =>
      expect(apiMocks.reorderEncounterParticipant).toHaveBeenCalledWith(
        "encounter_ambush",
        {
          participant_key: "participant_b",
          target_participant_key: "participant_a",
          placement: "before",
        },
      ),
    );
  });

  it("uses distinct form identities after Add PC cancellation and encounter settings", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Add participants" })).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add participants" }));
    fireEvent.click(await screen.findByRole("menuitem", { name: "Add PC…" }));
    const add = await screen.findByRole("dialog", { name: "Add PC" });
    const previousId = within(add).getByLabelText("Name").id;
    fireEvent.click(within(add).getByRole("button", { name: "Cancel" }));
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    const edit = await screen.findByRole("dialog", { name: "Settings" });
    const input = within(edit).getByLabelText("Name");
    expect(input.id).not.toBe(previousId);
    const label = within(edit).getByText("Name", {
      selector: "label",
    }) as HTMLLabelElement;
    expect(label.control).toBe(input);
    expect(document.querySelectorAll(`[id="${input.id}"]`)).toHaveLength(1);
  });

  it("selects the roster by keyboard and commits initiative without setting turn", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await screen.findByText("Kyra");
    const kyra = rosterRow("Kyra");
    const select = kyra.querySelector(".encounter-roster__select") as HTMLElement;
    select.focus();
    fireEvent.click(select);
    expect(await screen.findByLabelText("HP summary")).toHaveTextContent("24/24");
    expect(apiMocks.setEncounterTurn).not.toHaveBeenCalled();
    const initiative = within(kyra).getByLabelText("Kyra initiative total");
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

  it("marks the current roster row and shows current plus temporary HP", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await screen.findByLabelText("Goblin HP");
    const goblinRow = rosterRow("Goblin");
    expect(goblinRow).toHaveClass("encounter-roster__row--current");
    expect(within(goblinRow).queryByText("Current turn")).not.toBeInTheDocument();
    expect(
      within(goblinRow).getByRole("button", { name: "Goblin, Current turn" }),
    ).toHaveAttribute("aria-current", "true");
    expect(within(goblinRow).getByLabelText("Goblin HP")).toHaveTextContent(/^15$/);
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
    expect(
      within(activitiesSection).getByRole("button", { name: "Roll Claw damage" }),
    ).toHaveTextContent("1d6+2");
    expect(within(activitiesSection).getByText(/slashing/)).toBeInTheDocument();
  });

  it("renders manual PC runtime state without inferred speed rows", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click((await screen.findByText("Kyra")).closest("button")!);

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

    const kyraRow = (await screen.findByText("Kyra")).closest("button");
    if (!kyraRow) {
      throw new Error("Kyra roster row was not rendered");
    }
    fireEvent.click(kyraRow);

    const hpInput = await openHpEditor();
    expect(hpInput).toHaveValue("24");
    fireEvent.change(hpInput, { target: { value: "20" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_b",
          action: { kind: "set_values", current: { input: "20" } },
        }),
      ),
    );
  });

  it("keeps combat controls visible and saves participant settings once", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const summary = await screen.findByLabelText("HP summary");
    expect(screen.getByRole("button", { name: "Damage" })).toBeVisible();
    expect(screen.queryByLabelText("HP")).toBeNull();
    expect(screen.queryByLabelText("Participant name")).toBeNull();
    expect(
      summary.compareDocumentPosition(screen.getByText("Strikes")) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    fireEvent.click(
      await screen.findByRole("button", { name: "Participant settings" }),
    );
    const dialog = await screen.findByRole("dialog", { name: "Participant settings" });
    fireEvent.change(within(dialog).getByLabelText("Participant name"), {
      target: { value: "Goblin Scout" },
    });
    expect(apiMocks.updateEncounterParticipant).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledExactlyOnceWith(
        "encounter_ambush",
        expect.objectContaining({
          display_name: "Goblin Scout",
          expected_revision: 0n,
          hp_edit: false,
          max_hp_edit: false,
          variant_edit: false,
        }),
      ),
    );
    await waitFor(() => expect(dialog).not.toBeVisible());
  });

  it("preserves maximum override and adjustment intent through secondary controls", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const maximum = await openHpEditor("Maximum HP");
    fireEvent.change(maximum, { target: { value: "30" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          action: { kind: "set_values", maximum: { input: "30" } },
        }),
      ),
    );
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Edit HP" })).not.toBeInTheDocument(),
    );
    await openHpEditor("Maximum HP");
    fireEvent.click(screen.getByRole("button", { name: "Use creature default" }));
    await waitFor(() =>
      expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          action: { kind: "use_derived_maximum" },
        }),
      ),
    );

    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Edit HP" })).not.toBeInTheDocument(),
    );
    fireEvent.click(
      await screen.findByRole("button", { name: "Participant settings" }),
    );
    await selectOption(conditionCombobox("Adjustment"), "Elite");
    fireEvent.click(
      within(screen.getByRole("dialog", { name: "Participant settings" })).getByRole(
        "button",
        { name: "Save" },
      ),
    );
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
    const hp = await openHpEditor();
    fireEvent.change(hp, { target: { value: "8" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          action: { kind: "set_values", current: { input: "8" } },
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
    apiMocks.applyEncounterHpAction.mockClear();
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    expect(await screen.findByLabelText("HP summary")).toHaveTextContent("8/12");
    expect(apiMocks.applyEncounterHpAction).not.toHaveBeenCalled();
    expect(within(rosterRow("Goblin")).getByLabelText("Goblin HP")).toHaveTextContent(
      /^13$/,
    );
  });

  it("resets participant editor drafts when selecting another participant", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.change(await openHpEditor("Maximum HP"), { target: { value: "999" } });
    fireEvent.click(rosterRow("Kyra").querySelector(".encounter-roster__select")!);
    expect(await screen.findByLabelText("HP summary")).toHaveTextContent("24/24");
    expect(screen.queryByLabelText("Maximum HP")).toBeNull();
    expect(await openHpEditor("Maximum HP")).toHaveValue("24");
    expect(apiMocks.applyEncounterHpAction).not.toHaveBeenCalled();
  });

  it("keeps encounter lifecycle actions in the edit modal", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Settings" })).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    const dialog = await screen.findByRole("dialog", { name: "Settings" });
    await selectOption(within(dialog).getByRole("combobox"), "complete");
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));

    await waitFor(() => expect(apiMocks.updateEncounter).toHaveBeenCalled());
    expect(apiMocks.updateEncounter.mock.calls[0][0]).toEqual(
      expect.objectContaining({
        encounter_key: "encounter_ambush",
        slug: "ambush",
        status: "complete",
      }),
    );
  });

  it("dismisses linked previews on participant changes and allows reopening", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(
      await screen.findByRole("button", { name: "Participant settings" }),
    );
    fireEvent.click(await screen.findByRole("button", { name: "Original record" }));
    const linkedRuleButton = (await screen.findByText("Linked Rule")).closest(
      "a[href]",
    );
    if (!linkedRuleButton) {
      throw new Error("Linked Rule native link was not rendered");
    }
    fireEvent.click(linkedRuleButton);

    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        "rules:linked",
        undefined,
        expect.any(AbortSignal),
      ),
    );
    expect(await screen.findByLabelText("Reference preview")).toBeInTheDocument();

    const kyraRow = (await screen.findByText("Kyra")).closest("button");
    if (!kyraRow) {
      throw new Error("Kyra roster row was not rendered");
    }
    fireEvent.click(kyraRow);
    await waitFor(() =>
      expect(screen.queryByLabelText("Reference preview")).not.toBeInTheDocument(),
    );

    fireEvent.click(
      within(rosterRow("Goblin")).getByRole("button", {
        name: /^Goblin(?:, Current turn)?$/,
      }),
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Participant settings" }),
    );
    fireEvent.click(await screen.findByRole("button", { name: "Original record" }));
    const linkedRuleButtonAfterReselect = (
      await screen.findByText("Linked Rule")
    ).closest("a[href]");
    if (!linkedRuleButtonAfterReselect) {
      throw new Error("Linked Rule native link was not rendered after reselection");
    }
    fireEvent.click(linkedRuleButtonAfterReselect);
    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        "rules:linked",
        undefined,
        expect.any(AbortSignal),
      ),
    );

    expect(await screen.findByLabelText("Reference preview")).toBeInTheDocument();
  });

  it("dismisses a linked record preview with Escape", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(
      await screen.findByRole("button", { name: "Participant settings" }),
    );
    fireEvent.click(await screen.findByRole("button", { name: "Original record" }));
    const linkedRule = (await screen.findByText("Linked Rule")).closest("a[href]");
    if (!linkedRule) throw new Error("Linked Rule native link was not rendered");
    fireEvent.click(linkedRule);
    expect(await screen.findByLabelText("Reference preview")).toBeInTheDocument();

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
          embedded_path: claw.navigation.embedded_path,
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
    testQueryClients.add(client);
    render(
      <QueryClientProvider client={client}>
        <SearchProfileProvider>
          <ConfigProvider theme={{ token: { motion: false } }}>
            <App>
              <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />
            </App>
          </ConfigProvider>
        </SearchProfileProvider>
      </QueryClientProvider>,
    );
    fireEvent.click(await screen.findByRole("link", { name: "Claw" }));
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
          embedded_path: claw.navigation.embedded_path,
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
    expect(apiMocks.getEmbeddedReading).not.toHaveBeenCalled();
    expect(apiMocks.getRecordDetail).not.toHaveBeenCalled();
  });

  it("omits redundant owner breadcrumbs in narrow previews while preserving native selected-record links", async () => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ xs: true, md: false, xl: false });
    const initial = encounterDetailFixture();
    const actor = initial.participants[0].presentation!.body;
    if (actor.kind !== "creature") throw new Error("expected creature fixture");
    const claw = actor.value.activities.value![0];
    const selected = recordDetailFixture("actors:goblin");
    selected.selected = claw.navigation;
    selected.presentation.identity = { ...selected.record, title: claw.title };
    selected.presentation.body = { kind: "activity", value: claw };
    const parent = recordDetailFixture("actors:goblin");
    parent.presentation = initial.participants[0].presentation!;
    apiMocks.getEncounterReference.mockImplementation(
      (_encounter, _participant, request) =>
        Promise.resolve(request.selection.embedded_path.length ? selected : parent),
    );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("link", { name: "Claw" }));
    const preview = await screen.findByLabelText("Reference content");
    expect(
      within(preview).queryByRole("link", {
        name: `Return to ${selected.record.title}`,
      }),
    ).not.toBeInTheDocument();
    const full = screen.getByRole("link", { name: "Open reference full page" });
    const expected = atlasRoutePath({
      kind: "record",
      recordKey: selected.selected.record_key,
      selection: selected.selected,
    });
    expect(full).toHaveAttribute("href", expected);
    const before = window.location.href;
    for (const modifiers of [{ metaKey: true }, { ctrlKey: true }, { button: 1 }])
      expect(fireEvent.click(full, modifiers)).toBe(true);
    expect(window.location.href).toBe(before);
    fireEvent.click(full);
    expect(currentAtlasRoute()).toMatchObject({
      kind: "record",
      recordKey: selected.selected.record_key,
      selection: selected.selected,
    });
  });

  it("opens condition reference previews from canonical condition rows", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(await screen.findByRole("link", { name: "Frightened" }));

    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        "conditionitems:TBSHQspnbcqxsmjL",
        undefined,
        expect.any(AbortSignal),
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

  it("passes raw HP settings formulas to Rust", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const hp = await openHpEditor();
    fireEvent.change(hp, { target: { value: "50 - 7" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledWith("encounter_ambush", {
        participant_key: "participant_a",
        expected_revision: 0n,
        action: { kind: "set_values", current: { input: "50 - 7" } },
      }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Edit HP" })).not.toBeInTheDocument(),
    );
    const temp = await openHpEditor("Temporary HP");
    fireEvent.change(temp, { target: { value: "4 + 2" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(apiMocks.applyEncounterHpAction).toHaveBeenLastCalledWith(
        "encounter_ambush",
        expect.objectContaining({ action: { kind: "set_values", temporary: "4 + 2" } }),
      ),
    );
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Edit HP" })).not.toBeInTheDocument(),
    );
    expect(apiMocks.updateEncounterParticipant).not.toHaveBeenCalled();
  });

  it("passes raw HP changes and Enter healing to Rust", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const amount = await screen.findByLabelText("HP change");
    fireEvent.change(amount, { target: { value: "8 + 2" } });
    fireEvent.keyDown(amount, { key: "Enter" });
    await waitFor(() =>
      expect(apiMocks.applyEncounterHpAction).toHaveBeenLastCalledWith(
        "encounter_ambush",
        expect.objectContaining({ action: { kind: "heal", amount: "8 + 2" } }),
      ),
    );
    await waitFor(() => expect(amount).toHaveValue(""));
    fireEvent.change(amount, { target: { value: "8 + 2" } });
    fireEvent.click(screen.getByRole("button", { name: "Damage" }));
    await waitFor(() =>
      expect(apiMocks.applyEncounterHpAction).toHaveBeenLastCalledWith(
        "encounter_ambush",
        expect.objectContaining({ action: { kind: "damage", amount: "8 + 2" } }),
      ),
    );
    await waitFor(() => expect(amount).not.toBeDisabled());
    fireEvent.change(amount, { target: { value: "3" } });
    fireEvent.click(screen.getByRole("button", { name: "Heal" }));
    await waitFor(() =>
      expect(apiMocks.applyEncounterHpAction).toHaveBeenLastCalledWith(
        "encounter_ambush",
        expect.objectContaining({ action: { kind: "heal", amount: "3" } }),
      ),
    );
    expect(apiMocks.updateEncounterParticipant).not.toHaveBeenCalled();
  });

  it("searches before adding a record-backed participant", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Add participants" }),
      ).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add participants" }));
    fireEvent.click(
      await screen.findByRole("menuitem", {
        name: "Add creature or hazard…",
      }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Add creature or hazard",
    });
    const search = within(dialog).getByRole("searchbox", { name: "Search records" });
    fireEvent.change(search, {
      target: { value: "goblin" },
    });
    fireEvent.keyDown(search, { key: "Enter", keyCode: 13 });
    await waitFor(() =>
      expect(apiMocks.searchPage).toHaveBeenCalledWith(
        expect.objectContaining({
          criteria: expect.objectContaining({ query: "goblin" }),
        }),
        expect.any(AbortSignal),
      ),
    );

    const goblinOption = await within(dialog).findByText("Goblin Warrior");
    fireEvent.click(goblinOption.closest("a")!);
    expect(within(dialog).getByRole("button", { name: "Add" })).toBeEnabled();
    fireEvent.change(within(dialog).getByLabelText("Quantity"), {
      target: { value: "2" },
    });
    fireEvent.change(within(dialog).getByLabelText("Initiative"), {
      target: { value: "1.5" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    await within(dialog).findByText(/whole number/);
    expect(apiMocks.addEncounterRecordParticipant).not.toHaveBeenCalled();
    fireEvent.change(within(dialog).getByLabelText("Initiative"), {
      target: { value: "18" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));

    await waitFor(() =>
      expect(apiMocks.addEncounterRecordParticipant).toHaveBeenCalledWith({
        encounter_ref: "encounter_ambush",
        record_ref: "actors:goblin",
        quantity: 2,
        initiative: 18n,
      }),
    );
    await waitFor(() =>
      expect(within(dialog).getByLabelText("Quantity")).toHaveValue("1"),
    );
    expect(within(dialog).getByLabelText("Initiative")).toHaveValue("");
    expect(search).toHaveValue("goblin");
    expect(within(dialog).getByRole("status")).toHaveTextContent(
      "Added 2 × Goblin Warrior.",
    );
    expect(within(dialog).getByRole("button", { name: "Add" })).toBeEnabled();
    fireEvent.change(within(dialog).getByLabelText("Quantity"), {
      target: { value: "3" },
    });
    fireEvent.change(within(dialog).getByLabelText("Initiative"), {
      target: { value: "12" },
    });
    fireEvent.change(within(dialog).getByLabelText("Initiative"), {
      target: { value: "" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    await waitFor(() =>
      expect(apiMocks.addEncounterRecordParticipant).toHaveBeenLastCalledWith({
        encounter_ref: "encounter_ambush",
        record_ref: "actors:goblin",
        quantity: 3,
      }),
    );
    await waitFor(() =>
      expect(within(dialog).getByRole("button", { name: "Done" })).toBeEnabled(),
    );
    fireEvent.click(within(dialog).getByRole("button", { name: "Done" }));
    await waitFor(() => expect(dialog).not.toBeVisible());
  }, 10_000);

  it("resets Add once after committed refresh recovery and preserves the next draft across unrelated mutations", async () => {
    apiMocks.getEncounter
      .mockResolvedValueOnce(encounterDetailFixture())
      .mockRejectedValueOnce(new Error("Artifact unavailable"))
      .mockImplementation(() => Promise.resolve(serverEncounter));
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Add participants" })).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add participants" }));
    fireEvent.click(
      await screen.findByRole("menuitem", {
        name: "Add creature or hazard…",
      }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Add creature or hazard",
    });
    const search = within(dialog).getByRole("searchbox", { name: "Search records" });
    fireEvent.change(search, { target: { value: "goblin" } });
    fireEvent.keyDown(search, { key: "Enter", keyCode: 13 });
    fireEvent.click((await within(dialog).findByText("Goblin Warrior")).closest("a")!);
    fireEvent.change(within(dialog).getByLabelText("Quantity"), {
      target: { value: "2" },
    });
    fireEvent.change(within(dialog).getByLabelText("Initiative"), {
      target: { value: "18" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    await within(dialog).findByText("Saved, but the encounter could not refresh");
    expect(within(dialog).getByLabelText("Quantity")).toHaveValue("2");
    expect(within(dialog).getByLabelText("Initiative")).toHaveValue("18");
    expect(within(dialog).getByRole("button", { name: /Add$/ })).toBeDisabled();
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Retry encounter refresh" }),
    );
    await waitFor(() =>
      expect(within(dialog).getByLabelText("Quantity")).toHaveValue("1"),
    );
    expect(within(dialog).getByLabelText("Initiative")).toHaveValue("");
    expect(within(dialog).getByRole("status")).toHaveTextContent(
      "Added 2 × Goblin Warrior.",
    );
    expect(apiMocks.addEncounterRecordParticipant).toHaveBeenCalledOnce();
    fireEvent.change(within(dialog).getByLabelText("Quantity"), {
      target: { value: "4" },
    });
    fireEvent.change(within(dialog).getByLabelText("Initiative"), {
      target: { value: "9" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Done" }));
    await waitFor(() => expect(dialog).not.toBeVisible());
    const initiative = screen.getByLabelText("Kyra initiative total");
    fireEvent.change(initiative, { target: { value: "17" } });
    fireEvent.keyDown(initiative, { key: "Enter" });
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledOnce(),
    );
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Add participants" })).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add participants" }));
    fireEvent.click(
      await screen.findByRole("menuitem", {
        name: "Add creature or hazard…",
      }),
    );
    await waitFor(() => expect(dialog).toBeVisible());
    expect(within(dialog).getByLabelText("Quantity")).toHaveValue("4");
    expect(within(dialog).getByLabelText("Initiative")).toHaveValue("9");
    expect(apiMocks.getEncounter).toHaveBeenCalledTimes(4);
    expect(apiMocks.addEncounterRecordParticipant).toHaveBeenCalledOnce();
  }, 10_000);

  it("reviews outgoing reminders, saves exact actions, and keeps advancement explicit after removal", async () => {
    let removed = false;
    apiMocks.getEncounterTurnReview.mockImplementation(() =>
      Promise.resolve({
        encounter_key: "encounter_ambush",
        expected_revision: serverEncounter.revision,
        current_participant_key: "participant_a",
        next_participant_key: "participant_b",
        reminders: removed
          ? []
          : [
              {
                kind: "persistent_damage",
                participant_key: "participant_a",
                participant_name: "Goblin",
                condition_id: 7n,
                condition_name: "Persistent Damage",
                note: "1d6 fire",
              },
            ],
      }),
    );
    apiMocks.applyEncounterHpAction.mockImplementation(async () => {
      serverEncounter = { ...serverEncounter, revision: 1n };
      return { encounter_key: "encounter_ambush" };
    });
    apiMocks.removeEncounterParticipantCondition.mockImplementation(async () => {
      removed = true;
      serverEncounter = { ...serverEncounter, revision: 2n };
      return { encounter_key: "encounter_ambush" };
    });
    apiMocks.advanceEncounterTurn.mockImplementation(async () => {
      serverEncounter = {
        ...serverEncounter,
        revision: 3n,
        current_turn_participant_key: "participant_b",
      };
      return { encounter_key: "encounter_ambush" };
    });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await screen.findByLabelText("HP summary");
    fireEvent.click(within(rosterRow("Kyra")).getByRole("button", { name: /^Kyra/ }));
    fireEvent.click(screen.getAllByRole("button", { name: "Next turn" })[0]);
    const review = await screen.findByRole("dialog", { name: "Review turn" });
    expect(apiMocks.advanceEncounterTurn).not.toHaveBeenCalled();
    fireEvent.change(within(review).getByLabelText("HP change"), {
      target: { value: "6 - 2" },
    });
    fireEvent.click(within(review).getByRole("button", { name: "Damage" }));
    await waitFor(() =>
      expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledWith("encounter_ambush", {
        participant_key: "participant_a",
        expected_revision: 0n,
        action: { kind: "damage", amount: "6 - 2" },
      }),
    );
    expect(await within(review).findByText("HP saved for Goblin.")).toBeInTheDocument();
    await waitFor(() =>
      expect(
        within(review).getByRole("button", { name: "Recovery succeeded: remove" }),
      ).toBeEnabled(),
    );
    fireEvent.click(
      within(review).getByRole("button", { name: "Recovery succeeded: remove" }),
    );
    fireEvent.click(
      within(
        await screen.findByRole("dialog", {
          name: "Remove Persistent Damage from Goblin?",
        }),
      ).getByRole("button", { name: "Remove condition" }),
    );
    await waitFor(() =>
      expect(apiMocks.removeEncounterParticipantCondition).toHaveBeenCalledWith(
        "encounter_ambush",
        "participant_a",
        7n,
      ),
    );
    expect(
      await within(review).findByText("No remaining reminders."),
    ).toBeInTheDocument();
    expect(
      within(review).getByText("Persistent Damage removed from Goblin."),
    ).toBeInTheDocument();
    expect(apiMocks.advanceEncounterTurn).not.toHaveBeenCalled();
    fireEvent.click(
      within(review).getByRole("button", { name: "Continue to next turn" }),
    );
    await waitFor(() =>
      expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledWith("encounter_ambush", {
        expected_revision: 2n,
      }),
    );
    expect(apiMocks.setEncounterTurn).not.toHaveBeenCalled();
  }, 10_000);

  it("Cancel after a saved review action keeps that action without advancing", async () => {
    apiMocks.getEncounterTurnReview.mockResolvedValue({
      encounter_key: "encounter_ambush",
      expected_revision: 0n,
      current_participant_key: "participant_a",
      next_participant_key: "participant_b",
      reminders: [
        {
          kind: "persistent_damage",
          participant_key: "participant_a",
          participant_name: "Goblin",
          condition_id: 7n,
          condition_name: "Persistent Damage",
          note: null,
        },
      ],
    });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
    const review = await screen.findByRole("dialog", { name: "Review turn" });
    fireEvent.change(within(review).getByLabelText("HP change"), {
      target: { value: "2" },
    });
    fireEvent.click(within(review).getByRole("button", { name: "Damage" }));
    await within(review).findByText("HP saved for Goblin.");
    await waitFor(() =>
      expect(within(review).getByRole("button", { name: "Cancel" })).toBeEnabled(),
    );
    fireEvent.click(within(review).getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Review turn" }),
      ).not.toBeInTheDocument(),
    );
    expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledOnce();
    expect(apiMocks.advanceEncounterTurn).not.toHaveBeenCalled();
  });

  it("retains a stale condition draft and requires explicit reload before saving the latest remaining rounds", async () => {
    apiMocks.updateEncounterParticipantCondition.mockImplementationOnce(async () => {
      serverEncounter = {
        ...serverEncounter,
        revision: 9n,
        participants: serverEncounter.participants.map((p) => ({
          ...p,
          conditions: p.conditions.map((c) => ({
            ...c,
            duration: c.duration ? { ...c.duration, amount: 1n } : undefined,
          })),
        })),
      };
      throw new AtlasApiError(409, "Stale", {
        code: "encounter_turn_conflict",
        message: "Stale",
        retryable: false,
      });
    });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Edit Frightened" }));
    const dialog = await screen.findByRole("dialog", { name: "Edit Frightened" });
    fireEvent.change(within(dialog).getByLabelText("Condition note"), {
      target: { value: "My stale note" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipantCondition).toHaveBeenCalledWith(
        "encounter_ambush",
        "participant_a",
        expect.objectContaining({
          expected_encounter_revision: 0n,
          note: "My stale note",
        }),
      ),
    );
    const reload = await within(dialog).findByRole("button", {
      name: "Reload condition",
    });
    expect(within(dialog).getByLabelText("Condition note")).toHaveValue(
      "My stale note",
    );
    expect(within(dialog).getByRole("button", { name: "Save" })).toBeDisabled();
    fireEvent.click(reload);
    expect(within(dialog).getByLabelText("Duration amount")).toHaveValue("1");
    fireEvent.change(within(dialog).getByLabelText("Condition note"), {
      target: { value: "Reviewed note" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipantCondition).toHaveBeenLastCalledWith(
        "encounter_ambush",
        "participant_a",
        expect.objectContaining({
          expected_encounter_revision: 9n,
          note: "Reviewed note",
          duration: expect.objectContaining({ amount: 1n }),
        }),
      ),
    );
  });

  it("validates consecutive Add drafts and allows optional duration to be cleared", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Add Condition" }));
    await selectOption(conditionCombobox("Add condition"), "Sickened");
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    await waitFor(() =>
      expect(apiMocks.addEncounterParticipantCondition).toHaveBeenCalledOnce(),
    );
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Add condition" }),
      ).not.toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add Condition" }));
    await selectOption(conditionCombobox("Add condition"), "Frightened");
    const dialog = screen.getByRole("dialog", { name: "Add condition" });
    const value = await within(dialog).findByLabelText("Condition value");
    for (const invalid of ["", "1.5"]) {
      fireEvent.change(value, { target: { value: invalid } });
      fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
      await within(dialog).findByText(
        invalid ? "Enter a whole number, zero or greater." : "Enter a whole number.",
      );
      expect(apiMocks.addEncounterParticipantCondition).toHaveBeenCalledOnce();
    }
    fireEvent.change(value, { target: { value: "2" } });
    await selectOption(conditionCombobox("Duration unit"), "minutes");
    const duration = within(dialog).getByLabelText("Duration amount");
    fireEvent.change(duration, { target: { value: "3" } });
    fireEvent.change(duration, { target: { value: "" } });
    await selectOption(conditionCombobox("Duration unit"), "No duration");
    await waitFor(() => expect(duration).toBeDisabled());
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    await waitFor(() =>
      expect(apiMocks.addEncounterParticipantCondition).toHaveBeenCalledTimes(2),
    );
    expect(apiMocks.addEncounterParticipantCondition.mock.calls[1][1]).toMatchObject({
      condition_ref: "conditionitems:TBSHQspnbcqxsmjL",
      value: 2n,
      duration: undefined,
    });
  }, 10_000);

  it("retains a committed condition editor through failed owner read and retries only the read", async () => {
    apiMocks.addEncounterParticipantCondition.mockImplementationOnce(async () => {
      apiMocks.getEncounter.mockRejectedValueOnce(new Error("Owner read unavailable"));
      return { encounter_key: "encounter_ambush" };
    });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Add Condition" }));
    await selectOption(conditionCombobox("Add condition"), "Sickened");
    const dialog = screen.getByRole("dialog", { name: "Add condition" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    const retry = await within(dialog).findByRole("button", {
      name: "Retry encounter refresh",
    });
    expect(within(dialog).getByLabelText("Condition value")).toHaveValue("1");
    expect(within(dialog).getByRole("button", { name: "Add" })).toBeDisabled();
    fireEvent.click(retry);
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Add condition" }),
      ).not.toBeInTheDocument(),
    );
    expect(apiMocks.addEncounterParticipantCondition).toHaveBeenCalledOnce();
    expect(apiMocks.getEncounter).toHaveBeenCalledTimes(3);
  });

  it("blocks Continue after a saved reminder action cannot refresh and retries reads without duplicate damage", async () => {
    apiMocks.getEncounterTurnReview.mockImplementation(() =>
      Promise.resolve({
        encounter_key: "encounter_ambush",
        expected_revision: serverEncounter.revision,
        current_participant_key: "participant_a",
        next_participant_key: "participant_b",
        reminders: [
          {
            kind: "persistent_damage",
            participant_key: "participant_a",
            participant_name: "Goblin",
            condition_id: 7n,
            condition_name: "Persistent Damage",
            note: null,
          },
        ],
      }),
    );
    apiMocks.applyEncounterHpAction.mockImplementationOnce(async () => {
      serverEncounter = { ...serverEncounter, revision: 4n };
      apiMocks.getEncounter.mockRejectedValueOnce(new Error("Owner read unavailable"));
      return { encounter_key: "encounter_ambush" };
    });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
    const dialog = await screen.findByRole("dialog", { name: "Review turn" });
    const amount = within(dialog).getByLabelText("HP change");
    fireEvent.change(amount, { target: { value: "3" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Damage" }));
    const retry = await within(dialog).findByRole("button", {
      name: "Retry encounter refresh",
    });
    expect(
      within(dialog).getByRole("button", { name: "Continue to next turn" }),
    ).toBeDisabled();
    expect(await within(dialog).findByText("HP saved for Goblin.")).toBeInTheDocument();
    fireEvent.click(retry);
    await waitFor(() =>
      expect(
        within(dialog).getByRole("button", { name: "Continue to next turn" }),
      ).toBeEnabled(),
    );
    expect(amount).toHaveValue("");
    expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledOnce();
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Continue to next turn" }),
    );
    await waitFor(() =>
      expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledExactlyOnceWith(
        "encounter_ambush",
        { expected_revision: 4n },
      ),
    );
  });

  it.each(["damage", "remove"] as const)(
    "recovers a failed review read after saved %s without replaying its write",
    async (action) => {
      const reviewResult = () => ({
        encounter_key: "encounter_ambush",
        expected_revision: serverEncounter.revision,
        current_participant_key: "participant_a",
        next_participant_key: "participant_b",
        reminders: [
          {
            kind: "persistent_damage",
            participant_key: "participant_a",
            participant_name: "Goblin",
            condition_id: 7n,
            condition_name: "Persistent Damage",
            note: null,
          },
        ],
      });
      apiMocks.getEncounterTurnReview.mockImplementation(() =>
        Promise.resolve(reviewResult()),
      );
      const save = async () => {
        serverEncounter = { ...serverEncounter, revision: 6n };
        apiMocks.getEncounterTurnReview.mockRejectedValueOnce(
          new Error("Review unavailable"),
        );
        return { encounter_key: "encounter_ambush" };
      };
      if (action === "damage")
        apiMocks.applyEncounterHpAction.mockImplementationOnce(save);
      else apiMocks.removeEncounterParticipantCondition.mockImplementationOnce(save);
      render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
        wrapper: queryClientWrapper(),
      });
      fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
      const dialog = await screen.findByRole("dialog", { name: "Review turn" });
      const amount = within(dialog).getByLabelText("HP change");
      if (action === "damage") {
        fireEvent.change(amount, { target: { value: "4" } });
        fireEvent.click(within(dialog).getByRole("button", { name: "Damage" }));
      } else {
        fireEvent.click(
          within(dialog).getByRole("button", { name: "Recovery succeeded: remove" }),
        );
        fireEvent.click(
          within(
            await screen.findByRole("dialog", {
              name: "Remove Persistent Damage from Goblin?",
            }),
          ).getByRole("button", { name: "Remove condition" }),
        );
      }
      const retry = await within(dialog).findByRole("button", {
        name: "Retry turn review",
      });
      expect(
        within(dialog).getByText(
          action === "damage"
            ? "HP saved for Goblin."
            : "Persistent Damage removed from Goblin.",
        ),
      ).toBeInTheDocument();
      await waitFor(() => expect(amount).toHaveValue(""));
      expect(
        within(dialog).getByRole("button", { name: "Continue to next turn" }),
      ).toBeDisabled();
      expect(within(dialog).getByRole("button", { name: "Damage" })).toBeDisabled();
      expect(
        within(dialog).getByRole("button", { name: "Recovery succeeded: remove" }),
      ).toBeDisabled();
      expect(apiMocks.getEncounter).toHaveBeenCalledTimes(2);
      expect(apiMocks.getEncounterTurnReview).toHaveBeenCalledTimes(2);
      fireEvent.click(retry);
      await waitFor(() =>
        expect(
          within(dialog).getByRole("button", { name: "Continue to next turn" }),
        ).toBeEnabled(),
      );
      expect(apiMocks.getEncounter).toHaveBeenCalledTimes(2);
      expect(apiMocks.getEncounterTurnReview).toHaveBeenCalledTimes(3);
      expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledTimes(
        action === "damage" ? 1 : 0,
      );
      expect(apiMocks.removeEncounterParticipantCondition).toHaveBeenCalledTimes(
        action === "remove" ? 1 : 0,
      );
      expect(apiMocks.advanceEncounterTurn).not.toHaveBeenCalled();
      fireEvent.click(
        within(dialog).getByRole("button", { name: "Continue to next turn" }),
      );
      await waitFor(() =>
        expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledExactlyOnceWith(
          "encounter_ambush",
          { expected_revision: 6n },
        ),
      );
    },
  );

  it("reconciles stale advancement without retrying or following until explicit fresh Continue", async () => {
    apiMocks.getEncounterTurnReview.mockImplementation(() =>
      Promise.resolve({
        encounter_key: "encounter_ambush",
        expected_revision: serverEncounter.revision,
        current_participant_key: serverEncounter.current_turn_participant_key,
        next_participant_key: "participant_b",
        reminders: [
          {
            kind: "duration_ended",
            participant_key: "participant_a",
            participant_name: "Goblin",
            condition_id: 7n,
            condition_name: "Frightened",
          },
        ],
      }),
    );
    apiMocks.advanceEncounterTurn
      .mockImplementationOnce(async () => {
        serverEncounter = {
          ...serverEncounter,
          revision: 8n,
          current_turn_participant_key: "participant_b",
        };
        throw new AtlasApiError(409, "Stale turn", {
          code: "encounter_turn_conflict",
          message: "Stale turn",
          retryable: false,
        });
      })
      .mockImplementationOnce(async () => ({ encounter_key: "encounter_ambush" }));
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await screen.findByLabelText("HP summary");
    fireEvent.click(
      within(rosterRow("Goblin")).getByRole("button", {
        name: /^Goblin(?:, Current turn)?$/,
      }),
    );
    fireEvent.click(screen.getAllByRole("button", { name: "Next turn" })[0]);
    const dialog = await screen.findByRole("dialog", { name: "Review turn" });
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Continue to next turn" }),
    );
    await within(dialog).findByText(
      "Encounter changed. Review the refreshed values before retrying your edit.",
    );
    await waitFor(() =>
      expect(
        within(dialog).getByRole("button", { name: "Continue to next turn" }),
      ).toBeEnabled(),
    );
    expect(apiMocks.getEncounter).toHaveBeenCalledTimes(2);
    expect(apiMocks.getEncounterTurnReview).toHaveBeenCalledTimes(2);
    expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledExactlyOnceWith(
      "encounter_ambush",
      { expected_revision: 0n },
    );
    expect(rosterRow("Goblin").querySelector("[data-participant-key]")).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(rosterRow("Kyra").querySelector("[data-participant-key]")).toHaveAttribute(
      "aria-pressed",
      "false",
    );
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Continue to next turn" }),
    );
    await waitFor(() =>
      expect(apiMocks.advanceEncounterTurn).toHaveBeenNthCalledWith(
        2,
        "encounter_ambush",
        { expected_revision: 8n },
      ),
    );
    await waitFor(() =>
      expect(rosterRow("Kyra").querySelector("[data-participant-key]")).toHaveAttribute(
        "aria-pressed",
        "true",
      ),
    );
    expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledTimes(2);
    expect(apiMocks.setEncounterTurn).not.toHaveBeenCalled();
  });

  it.each([
    ["start", "participant_b", "Kyra"],
    ["end", "participant_a", "Goblin"],
  ] as const)(
    "forecasts expiry at the actual %s boundary of %s",
    async (event, participantKey, name) => {
      apiMocks.getEncounterTurnReview.mockResolvedValue({
        encounter_key: "encounter_ambush",
        expected_revision: 0n,
        current_participant_key: "participant_a",
        next_participant_key: "participant_b",
        reminders: [
          {
            kind: "duration_ending",
            participant_key: "participant_a",
            participant_name: "Goblin",
            condition_id: 7n,
            condition_name: "Frightened",
            timing: { participant_key: participantKey, event },
          },
        ],
      });
      render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
        wrapper: queryClientWrapper(),
      });
      fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
      const dialog = await screen.findByRole("dialog", { name: "Review turn" });
      expect(
        within(dialog).getByText(
          `Expires at the ${event} of ${name}’s turn when you continue. Remove now to end this condition immediately.`,
        ),
      ).toBeInTheDocument();
      expect(
        within(dialog).getByRole("heading", { name: "Goblin: Frightened" }),
      ).toBeInTheDocument();
      expect(apiMocks.advanceEncounterTurn).not.toHaveBeenCalled();
    },
  );

  it("renders ended, manual and unavailable timing reminders without inventing expiry or advancing on removal", async () => {
    apiMocks.getEncounterTurnReview.mockResolvedValue({
      encounter_key: "encounter_ambush",
      expected_revision: 0n,
      current_participant_key: "participant_a",
      next_participant_key: "participant_b",
      reminders: [
        {
          kind: "duration_ended",
          participant_key: "participant_a",
          participant_name: "Goblin",
          condition_id: 7n,
          condition_name: "Frightened",
        },
        {
          kind: "duration_reminder",
          participant_key: "participant_a",
          participant_name: "Goblin",
          condition_id: 8n,
          condition_name: "Sickened",
          duration: { amount: 5n, unit: "minutes", timing: null },
        },
        {
          kind: "timing_unavailable",
          participant_key: "participant_a",
          participant_name: "Goblin",
          condition_id: 9n,
          condition_name: "Prone",
          duration: { amount: 2n, unit: "rounds", timing: null },
        },
      ],
    });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
    const dialog = await screen.findByRole("dialog", { name: "Review turn" });
    const ended = within(dialog)
      .getByRole("heading", { name: "Goblin: Frightened" })
      .closest("section")!;
    const manual = within(dialog)
      .getByRole("heading", { name: "Goblin: Sickened" })
      .closest("section")!;
    const unavailable = within(dialog)
      .getByRole("heading", { name: "Goblin: Prone" })
      .closest("section")!;
    expect(
      within(ended).getByText("Duration ended. Confirm removal when ready."),
    ).toBeInTheDocument();
    expect(
      within(manual).getByText("5 minutes: check this duration at the table."),
    ).toBeInTheDocument();
    expect(within(manual).queryByRole("button")).not.toBeInTheDocument();
    expect(within(manual).queryByText(/expires|ended/i)).not.toBeInTheDocument();
    expect(
      within(unavailable).getByText(
        "Round timing is not configured. Edit the condition to choose whose turn and when its rounds count down.",
      ),
    ).toBeInTheDocument();
    expect(within(unavailable).queryByRole("button")).not.toBeInTheDocument();
    expect(
      within(unavailable).queryByText(/start|end of|expires/i),
    ).not.toBeInTheDocument();
    fireEvent.click(within(ended).getByRole("button", { name: "Remove condition" }));
    expect(apiMocks.removeEncounterParticipantCondition).not.toHaveBeenCalled();
    fireEvent.click(
      within(
        await screen.findByRole("dialog", { name: "Remove Frightened from Goblin?" }),
      ).getByRole("button", { name: "Remove condition" }),
    );
    await waitFor(() =>
      expect(
        apiMocks.removeEncounterParticipantCondition,
      ).toHaveBeenCalledExactlyOnceWith("encounter_ambush", "participant_a", 7n),
    );
    expect(
      await within(dialog).findByText("Frightened removed from Goblin."),
    ).toBeInTheDocument();
    expect(apiMocks.advanceEncounterTurn).not.toHaveBeenCalled();
  });

  it("adds conditions with compact fields and details", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await screen.findByText("Frightened");

    fireEvent.click(screen.getByRole("button", { name: "Add Condition" }));
    await selectOption(conditionCombobox("Add condition"), "Sickened");
    expect(await screen.findByLabelText("Condition value")).toHaveValue("1");
    const plus = within(
      screen.getByRole("dialog", { name: "Add condition" }),
    ).getByRole("button", { name: "Increase Value" });
    fireEvent.mouseDown(plus);
    fireEvent.mouseUp(plus);
    expect(screen.getByLabelText("Condition value")).toHaveValue("2");
    await selectOption(conditionCombobox("Duration unit"), "rounds");
    fireEvent.change(await screen.findByLabelText("Duration amount"), {
      target: { value: "3" },
    });
    await screen.findByLabelText("Duration turn");
    await selectOption(conditionCombobox("Duration turn"), "Goblin");
    await selectOption(conditionCombobox("Duration timing"), "End of turn");
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
          duration: {
            amount: 3n,
            unit: "rounds",
            timing: { participant_key: "participant_a", event: "end" },
          },
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

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Edit Frightened" })).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Edit Frightened" }));
    fireEvent.change(lastFieldByAriaLabel("Duration amount"), {
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
          duration: {
            amount: 4n,
            unit: "rounds",
            timing: { participant_key: "participant_a", event: "end" },
          },
          note: "aura",
        }),
      ),
    );

    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Edit Frightened" }),
      ).not.toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Edit Frightened" }));
    fireEvent.click(
      within(await screen.findByRole("dialog", { name: "Edit Frightened" })).getByRole(
        "button",
        { name: "Remove condition" },
      ),
    );
    fireEvent.click(
      within(
        await screen.findByRole("dialog", { name: "Remove Frightened?" }),
      ).getByRole("button", { name: "Remove condition" }),
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
    apiMocks.applyEncounterHpAction.mockReturnValue(pending.promise);
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const hp = await openHpEditor();
    fireEvent.change(hp, { target: { value: "3" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(hp).toBeDisabled());
    expect(screen.getByLabelText("HP summary")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Goblin initiative: 18" }),
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Next turn" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Actions for Goblin" })).toBeDisabled();
    fireEvent.click(
      within(screen.getByRole("dialog", { name: "Edit HP" })).getByRole("button", {
        name: /Save/,
      }),
    );
    fireEvent.drop(rosterRow("Kyra"));
    await act(async () => pending.reject(new Error("Write unavailable")));
    expect(
      await within(screen.getByRole("dialog", { name: "Edit HP" })).findByText(
        "Write unavailable",
      ),
    ).toBeInTheDocument();
    expect(hp).toHaveValue("3");
    expect(screen.getByLabelText("HP summary")).toBeInTheDocument();
    expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledOnce();
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
    apiMocks.applyEncounterHpAction.mockRejectedValue(
      new AtlasApiError(409, "Stale", {
        code: "encounter_participant_conflict",
        message: "Stale",
        retryable: false,
      }),
    );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const hp = await openHpEditor();
    fireEvent.change(hp, { target: { value: "3" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(
      await within(screen.getByRole("dialog", { name: "Edit HP" })).findByText(
        /Encounter changed. Review/,
      ),
    ).toBeInTheDocument();
    expect(hp).toHaveValue("3");
    expect(screen.getByText("9/12 +5")).toBeInTheDocument();
    expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledOnce();
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
    const hp = await openHpEditor();
    fireEvent.change(hp, { target: { value: "3" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(
      await within(screen.getByRole("dialog", { name: "Edit HP" })).findByText(
        "Saved, but the encounter could not refresh",
      ),
    ).toBeInTheDocument();
    expect(hp).toBeDisabled();
    expect(screen.getByLabelText("HP summary")).toBeInTheDocument();
    fireEvent.click(
      within(screen.getByRole("dialog", { name: "Edit HP" })).getByRole("button", {
        name: "Retry encounter refresh",
      }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Edit HP" })).not.toBeInTheDocument(),
    );
    expect(screen.getByLabelText("HP summary")).toHaveTextContent("3/12");
    expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledOnce();
  });

  it("keeps delayed and rejected Save open with its draft", async () => {
    const saving = deferred<{ encounter_key: string }>();
    apiMocks.updateEncounter.mockReturnValue(saving.promise);
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() => expect(screen.getByLabelText("Settings")).not.toBeDisabled());
    fireEvent.click(screen.getByLabelText("Settings"));
    const dialog = await screen.findByRole("dialog", { name: "Settings" });
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
        screen.queryByRole("dialog", { name: "Settings" }),
      ).not.toBeInTheDocument(),
    );
  });

  it("discards canceled encounter metadata when opening a new editing session", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const edit = screen.getByRole("button", { name: "Settings" });
    await waitFor(() => expect(edit).not.toBeDisabled());
    fireEvent.click(edit);
    const dialog = await screen.findByRole("dialog", { name: "Settings" });
    const name = within(dialog).getByLabelText("Name");
    const savedName = encounterIndexFixture().encounters[0].name;
    await waitFor(() => expect(name).toHaveValue(savedName));
    fireEvent.change(name, { target: { value: "Canceled draft" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Settings" }),
      ).not.toBeInTheDocument(),
    );
    fireEvent.click(edit);
    const reopened = await screen.findByRole("dialog", { name: "Settings" });
    await waitFor(() =>
      expect(within(reopened).getByLabelText("Name")).toHaveValue(savedName),
    );
    expect(apiMocks.updateEncounter).not.toHaveBeenCalled();
  });

  it("preserves touched encounter fields across changed server data and resets identity", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    testQueryClients.add(client);
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>
        <SearchProfileProvider>
          <ConfigProvider theme={{ token: { motion: false } }}>
            <App component={false}>{children}</App>
          </ConfigProvider>
        </SearchProfileProvider>
      </QueryClientProvider>
    );
    const { rerender } = render(
      <EncounterEditView route={{ kind: "encounterEdit", slug: "ambush" }} />,
      { wrapper },
    );
    const name = await screen.findByLabelText("Name");
    fireEvent.change(name, { target: { value: "My draft" } });
    client.setQueryData(["record-detail", "old"], {
      artifact_generation: "generation-a",
    });
    await act(async () => retireImmutableReferenceQueries(client));
    expect(client.getQueryData(["record-detail", "old"])).toBeUndefined();
    expect(client.getQueryData(["encounter", "ambush"])).toBeDefined();
    expect(name).toHaveValue("My draft");
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
    expect(screen.getByLabelText("Settings")).toBeDisabled();
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
    apiMocks.applyEncounterHpAction.mockReturnValue(pending.promise);
    const { rerender } = render(
      <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />,
      { wrapper: queryClientWrapper() },
    );
    const hp = await openHpEditor();
    fireEvent.change(hp, { target: { value: "3" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(hp).toBeDisabled());
    setEncounterDetail({
      ...encounterDetailFixture(),
      encounter: { ...encounterIndexFixture().encounters[1], name: "Other" },
    });
    rerender(<EncounterDetailView route={{ kind: "encounter", slug: "other" }} />);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Edit HP" })).not.toBeDisabled(),
    );
    await act(async () => pending.reject(new Error("Old failure")));
    expect(screen.queryByText("Old failure")).not.toBeInTheDocument();
    expect(screen.getByLabelText("HP summary")).toHaveTextContent("10/12");
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

  it("validates optional PC integers visibly and permits clearing both fields before retry", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Add participants" }));
    fireEvent.click(await screen.findByRole("menuitem", { name: "Add PC…" }));
    const dialog = await screen.findByRole("dialog", { name: "Add PC" });
    fireEvent.change(within(dialog).getByLabelText("Name"), {
      target: { value: "Mira" },
    });
    const hp = within(dialog).getByLabelText("Max HP");
    const initiative = within(dialog).getByLabelText("Initiative");
    for (const field of [hp, initiative]) {
      fireEvent.change(field, { target: { value: "1.5" } });
      fireEvent.click(within(dialog).getByRole("button", { name: "OK" }));
      await within(dialog).findByText(/whole number/);
      expect(apiMocks.addEncounterManualParticipant).not.toHaveBeenCalled();
      fireEvent.change(field, { target: { value: "12" } });
      fireEvent.change(field, { target: { value: "" } });
    }
    fireEvent.click(within(dialog).getByRole("button", { name: "OK" }));
    await waitFor(() =>
      expect(apiMocks.addEncounterManualParticipant).toHaveBeenCalledExactlyOnceWith({
        encounter_ref: "encounter_ambush",
        display_name: "Mira",
      }),
    );
  });

  it("clears a committed HP amount after failed refresh and retries only the read", async () => {
    apiMocks.getEncounter
      .mockResolvedValueOnce(encounterDetailFixture())
      .mockRejectedValueOnce(new Error("Artifact unavailable"))
      .mockResolvedValue(
        encounterDetailFixture("participant_a", { current_hp: 5n, revision: 1n }),
      );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    const amount = await screen.findByLabelText("HP change");
    amount.focus();
    fireEvent.change(amount, { target: { value: "-5" } });
    fireEvent.keyDown(amount, { key: "Enter" });
    await screen.findByText("Saved, but the encounter could not refresh");
    expect(amount).toHaveValue("");
    expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledExactlyOnceWith(
      "encounter_ambush",
      expect.objectContaining({ action: { kind: "heal", amount: "-5" } }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Retry encounter refresh" }));
    await waitFor(() => expect(amount).toBeEnabled());
    fireEvent.keyDown(amount, { key: "Enter" });
    expect(apiMocks.applyEncounterHpAction).toHaveBeenCalledOnce();
    expect(apiMocks.getEncounter).toHaveBeenCalledTimes(3);
  });

  it("preserves a failed add-PC draft until a successful receipt", async () => {
    const adding = deferred<{ encounter_key: string }>();
    apiMocks.addEncounterManualParticipant.mockReturnValueOnce(adding.promise);
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Add participants" }),
      ).not.toBeDisabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add participants" }));
    fireEvent.click(await screen.findByRole("menuitem", { name: "Add PC…" }));
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
    fireEvent.click(await screen.findByRole("button", { name: "Edit Frightened" }));
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

  it("finishes scheduled form feedback before disposing the encounter test environment", async () => {
    const clockSetTimeout = globalThis.setTimeout;
    const feedbackCallbacks: ReturnType<typeof vi.fn>[] = [];
    const timerSpy = vi
      .spyOn(globalThis, "setTimeout")
      .mockImplementation((callback, delay, ...args) => {
        if (delay === 10 && typeof callback === "function") {
          const feedback = vi.fn(() => callback(...args));
          feedbackCallbacks.push(feedback);
          return clockSetTimeout(feedback, delay);
        }
        return clockSetTimeout(callback, delay, ...args);
      });
    try {
      render(
        <EditEncounterForm
          encounter={encounterIndexFixture().encounters[0]}
          encounterNote="Encounter note"
          onSave={vi.fn()}
        />,
        { wrapper: queryClientWrapper() },
      );
      expect(screen.getByLabelText("Name")).toHaveValue("Ambush");
      const clients = [...testQueryClients];
      for (const client of clients) client.setQueryData(["disposal-probe"], "retained");
      cleanup();
      for (const client of testQueryClients) client.clear();
      // Ant Form's empty error/warning feedback schedules debounce work even
      // without validation errors. Unmounting does not cancel those callbacks.
      expect(vi.getTimerCount()).toBeGreaterThan(0);
      expect(feedbackCallbacks.length).toBeGreaterThan(0);
      expect(
        feedbackCallbacks.every((callback) => callback.mock.calls.length === 0),
      ).toBe(true);
      await disposeEncounterTest();
      expect(vi.getTimerCount()).toBe(0);
      expect(feedbackCallbacks.some((callback) => callback.mock.calls.length > 0)).toBe(
        true,
      );
      for (const client of clients) expect(client.getQueryCache().getAll()).toEqual([]);
    } finally {
      timerSpy.mockRestore();
    }
  });
});

async function disposeEncounterTest() {
  cleanup();
  for (const client of testQueryClients) client.clear();
  testQueryClients.clear();
  // Execute pending native Form feedback while jsdom/React still exist. The
  // suite owns this fake clock; QueryClient disposal removes its GC work first.
  await act(async () => vi.runOnlyPendingTimersAsync());
  expect(vi.getTimerCount()).toBe(0);
}

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
    .map((element) => element.closest(".encounter-roster__row"))
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
  testQueryClients.add(queryClient);
  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <SearchProfileProvider>
          <ConfigProvider
            theme={{ token: { motion: false } }}
            select={{ classNames: { popup: { listItem: "atlas-select-option" } } }}
          >
            <App component={false}>{children}</App>
          </ConfigProvider>
        </SearchProfileProvider>
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
    supports_duration_instead_of_value: false,
    has_value: hasValue,
    ...(hasValue ? { default_value: 1n } : {}),
  };
}

function encounterDetailFixture(
  currentTurnParticipantKey: string | undefined = "participant_a",
  firstParticipantOverrides: Partial<EncounterParticipantView> = {},
): EncounterDetailViewDto {
  return {
    revision: 0n,
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
            duration: {
              amount: 2n,
              unit: "rounds",
              timing: { participant_key: "participant_a", event: "end" },
            },
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
    hazard_disabled: false,
    cast_unavailable_reason: null,
    hidden: false,
    note_hint: null,
    conditions: [],
    ...overrides,
  };
}

function searchResultsFixture(): SearchResultsPage {
  return {
    ranking: "score" as const,
    search_profile: "bge-base",
    artifact_generation: "generation-a",
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
    embedded: [],
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

async function openHpEditor(target = "Current HP") {
  fireEvent.click(await screen.findByRole("button", { name: "Edit HP" }));
  await screen.findByRole("dialog", { name: "Edit HP" });
  return screen.findByLabelText(target);
}

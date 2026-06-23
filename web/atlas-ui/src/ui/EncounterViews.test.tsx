import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  EncounterDetailView as EncounterDetailViewDto,
  EncounterIndexView as EncounterIndexViewDto,
  EncounterParticipantView,
  RecordDetailView,
  RecordSummaryView,
  ResultWindowPage,
} from "../generated/atlas";
import { EncounterDetailView, EncounterIndexView } from "./EncounterViews";

const apiMocks = vi.hoisted(() => ({
  addEncounterManualParticipant: vi.fn(),
  addEncounterParticipantCondition: vi.fn(),
  addEncounterRecordParticipant: vi.fn(),
  createEncounter: vi.fn(),
  deleteEncounter: vi.fn(),
  getEncounter: vi.fn(),
  getEncounters: vi.fn(),
  getRecordDetail: vi.fn(),
  openResultWindow: vi.fn(),
  removeEncounterParticipant: vi.fn(),
  removeEncounterParticipantCondition: vi.fn(),
  reorderEncounterParticipant: vi.fn(),
  setEncounterTurn: vi.fn(),
  updateEncounter: vi.fn(),
  updateEncounterParticipant: vi.fn(),
  updateEncounterParticipantCondition: vi.fn(),
}));

vi.mock("../api/atlasApi", () => ({
  addEncounterManualParticipant: apiMocks.addEncounterManualParticipant,
  addEncounterParticipantCondition: apiMocks.addEncounterParticipantCondition,
  addEncounterRecordParticipant: apiMocks.addEncounterRecordParticipant,
  createEncounter: apiMocks.createEncounter,
  deleteEncounter: apiMocks.deleteEncounter,
  getEncounter: apiMocks.getEncounter,
  getEncounters: apiMocks.getEncounters,
  getRecordDetail: apiMocks.getRecordDetail,
  openResultWindow: apiMocks.openResultWindow,
  removeEncounterParticipant: apiMocks.removeEncounterParticipant,
  removeEncounterParticipantCondition: apiMocks.removeEncounterParticipantCondition,
  reorderEncounterParticipant: apiMocks.reorderEncounterParticipant,
  setEncounterTurn: apiMocks.setEncounterTurn,
  updateEncounter: apiMocks.updateEncounter,
  updateEncounterParticipant: apiMocks.updateEncounterParticipant,
  updateEncounterParticipantCondition: apiMocks.updateEncounterParticipantCondition,
}));

describe("encounter views", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    history.replaceState(null, "", "/encounters");
    apiMocks.getEncounters.mockResolvedValue(encounterIndexFixture());
    apiMocks.getEncounter.mockResolvedValue(encounterDetailFixture());
    apiMocks.getRecordDetail.mockResolvedValue(recordDetailFixture("actors:goblin"));
    apiMocks.openResultWindow.mockResolvedValue(resultWindowFixture());
    apiMocks.setEncounterTurn.mockResolvedValue(
      encounterDetailFixture("participant_b"),
    );
    apiMocks.updateEncounterParticipant.mockImplementation((_slug: string, request) =>
      Promise.resolve(request),
    );
    apiMocks.addEncounterRecordParticipant.mockResolvedValue(encounterDetailFixture());
    apiMocks.addEncounterParticipantCondition.mockResolvedValue(
      encounterDetailFixture(),
    );
    apiMocks.updateEncounterParticipantCondition.mockResolvedValue(
      encounterDetailFixture(),
    );
    apiMocks.removeEncounterParticipantCondition.mockResolvedValue(
      encounterDetailFixture(),
    );
  });

  it("renders active encounter rows, hides archived rows, and routes to edit", async () => {
    render(<EncounterIndexView route={{ kind: "encounters" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByText("Ambush")).toBeInTheDocument();
    expect(screen.queryByText("Old Fight")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("checkbox", { name: "Archived" }));
    expect(await screen.findByText("Old Fight")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Edit Ambush" }));
    await waitFor(() =>
      expect(window.location.pathname).toBe("/encounters/ambush/edit"),
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
        encounter_ref: "ambush",
        participant_key: "participant_b",
      }),
    );
  });

  it("applies HP formulas and consumes temporary HP before current HP", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const hpInput = await screen.findByLabelText("HP or formula");
    fireEvent.change(hpInput, { target: { value: "50 - 7" } });
    fireEvent.keyDown(hpInput, { key: "Enter" });

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          current_hp: 43n,
        }),
      ),
    );

    fireEvent.change(screen.getByLabelText("Amount"), { target: { value: "8" } });
    fireEvent.click(screen.getByRole("button", { name: "Damage" }));

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          temporary_hp: 0n,
          current_hp: 7n,
        }),
      ),
    );
  });

  it("searches before adding a record-backed participant", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(await screen.findByRole("button", { name: "Add Creature" }));
    expect(apiMocks.openResultWindow).not.toHaveBeenCalled();

    const dialog = await screen.findByRole("dialog", {
      name: "Add creature or hazard",
    });
    fireEvent.change(within(dialog).getByLabelText("Search"), {
      target: { value: "goblin" },
    });
    await waitFor(() => expect(apiMocks.openResultWindow).toHaveBeenCalledTimes(1));

    fireEvent.click(
      await within(dialog).findByRole("button", { name: /Goblin Warrior/i }),
    );
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
        encounter_ref: "ambush",
        record_ref: "actors:goblin",
        quantity: 2,
        initiative: 18n,
      }),
    );
  }, 10_000);

  it("adds, edits, and removes conditions for the current participant", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await screen.findByDisplayValue("Frightened");

    const conditionInputs = screen.getAllByLabelText("Condition");
    fireEvent.change(conditionInputs[conditionInputs.length - 1], {
      target: { value: "Sickened" },
    });
    const valueInputs = screen.getAllByLabelText("Value");
    fireEvent.change(valueInputs[valueInputs.length - 1], {
      target: { value: "2" },
    });
    const roundsInputs = screen.getAllByLabelText("Rounds");
    fireEvent.change(roundsInputs[roundsInputs.length - 1], {
      target: { value: "3" },
    });
    const noteInputs = screen.getAllByLabelText("Note");
    fireEvent.change(noteInputs[noteInputs.length - 1], {
      target: { value: "poison" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add Condition" }));

    await waitFor(() =>
      expect(apiMocks.addEncounterParticipantCondition).toHaveBeenCalledWith(
        "ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          name: "Sickened",
          value: 2n,
          duration_rounds: 3n,
          note: "poison",
        }),
      ),
    );
    expect(
      apiMocks.addEncounterParticipantCondition.mock.calls[0][1],
    ).not.toHaveProperty("condition_key");

    const existingConditionInput = screen.getByDisplayValue("Frightened");
    fireEvent.change(existingConditionInput, { target: { value: "Frightened 2" } });
    fireEvent.blur(existingConditionInput);

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipantCondition).toHaveBeenCalledWith(
        "ambush",
        "participant_a",
        expect.objectContaining({
          condition_id: 7n,
          name: "Frightened 2",
        }),
      ),
    );
    expect(
      apiMocks.updateEncounterParticipantCondition.mock.calls[0][2],
    ).not.toHaveProperty("condition_key");

    fireEvent.click(screen.getByRole("button", { name: "Remove Frightened" }));

    await waitFor(() =>
      expect(apiMocks.removeEncounterParticipantCondition).toHaveBeenCalledWith(
        "ambush",
        "participant_a",
        7n,
      ),
    );
  }, 10_000);
});

function queryClientWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
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

function encounterDetailFixture(
  currentTurnParticipantKey: string | undefined = "participant_a",
): EncounterDetailViewDto {
  return {
    encounter: encounterIndexFixture().encounters[0],
    note: "Encounter note",
    current_turn_participant_key: currentTurnParticipantKey,
    participants: [
      participantFixture({
        participant_key: "participant_a",
        display_name: "Goblin",
        record_key: "actors:goblin",
        participant_kind: "creature",
        side: "enemy",
        initiative: 18n,
        max_hp: 12n,
        current_hp: 10n,
        temporary_hp: 5n,
        record: recordSummaryFixture("actors:goblin", "Goblin Warrior"),
        conditions: [
          {
            condition_id: 7n,
            condition_key: "conditionitems:frightened",
            name: "Frightened",
            value: 1n,
            duration_rounds: 2n,
            created_at: "2026-01-01T00:00:00Z",
            updated_at: "2026-01-01T00:00:00Z",
          },
        ],
      }),
      participantFixture({
        participant_key: "participant_b",
        display_name: "Kyra",
        participant_kind: "pc",
        status: "manual",
        side: "pc",
        initiative: 15n,
        max_hp: 24n,
        current_hp: 24n,
      }),
    ],
  };
}

function participantFixture(
  overrides: Partial<EncounterParticipantView>,
): EncounterParticipantView {
  return {
    participant_key: "participant",
    participant_kind: "creature",
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
    mode: { kind: "text_search", query: "goblin" },
    page: { number: 1, size: 25, count: 1, total: 1n, has_more: false },
    rows: [
      {
        record: recordSummaryFixture("actors:goblin", "Goblin Warrior"),
        match_summary: undefined,
      },
    ],
  };
}

function recordSummaryFixture(recordKey: string, title: string): RecordSummaryView {
  return {
    record_key: recordKey,
    title,
    kind: "creature",
    kind_label: "Creature",
    level_label: "1",
    rarity: undefined,
    traits: [],
    taxonomy: [],
    publication: undefined,
    pack: "Bestiary",
    preview: "A small enemy.",
  };
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  return {
    record_key: recordKey,
    title: "Goblin Warrior",
    kind: "creature",
    presentation: {
      record_key: recordKey,
      kind: "creature",
      title: "Goblin Warrior",
      identity: [],
      badges: [],
      sections: [],
    },
  };
}

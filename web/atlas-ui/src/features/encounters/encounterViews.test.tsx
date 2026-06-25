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
  openResultWindow: vi.fn(),
  removeEncounterParticipant: vi.fn(),
  removeEncounterParticipantCondition: vi.fn(),
  reorderEncounterParticipant: vi.fn(),
  setEncounterTurn: vi.fn(),
  updateEncounter: vi.fn(),
  updateEncounterParticipant: vi.fn(),
  updateEncounterParticipantCondition: vi.fn(),
}));

vi.mock("../../api/atlasApi", () => ({
  addEncounterManualParticipant: apiMocks.addEncounterManualParticipant,
  addEncounterParticipantCondition: apiMocks.addEncounterParticipantCondition,
  addEncounterRecordParticipant: apiMocks.addEncounterRecordParticipant,
  createEncounter: apiMocks.createEncounter,
  deleteEncounter: apiMocks.deleteEncounter,
  getEncounter: apiMocks.getEncounter,
  getEncounterConditionDefinitions: apiMocks.getEncounterConditionDefinitions,
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
    apiMocks.getEncounterConditionDefinitions.mockResolvedValue(
      conditionDefinitionsFixture(),
    );
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) =>
      Promise.resolve(recordDetailFixture(recordKey)),
    );
    apiMocks.openResultWindow.mockResolvedValue(resultWindowFixture());
    apiMocks.setEncounterTurn.mockResolvedValue(
      encounterDetailFixture("participant_b"),
    );
    apiMocks.updateEncounter.mockResolvedValue({
      encounter: {
        ...encounterIndexFixture().encounters[0],
        name: "Renamed Ambush",
      },
    });
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
        encounter_ref: "ambush",
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

    fireEvent.click(await screen.findByText("Next"));
    await waitFor(() =>
      expect(apiMocks.setEncounterTurn).toHaveBeenCalledWith({
        encounter_ref: "ambush",
      }),
    );

    unmount();
    vi.clearAllMocks();
    apiMocks.getEncounter.mockResolvedValue(encounterDetailFixture(undefined));
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(await screen.findByText("Play"));
    await waitFor(() =>
      expect(apiMocks.setEncounterTurn).toHaveBeenCalledWith({
        encounter_ref: "ambush",
      }),
    );
  });

  it("marks the current roster row and omits max HP from roster text", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await screen.findByText("Initiative set");
    const goblinRow = rosterRow("Goblin");
    expect(goblinRow).toHaveClass("encounter-roster__row--current");
    expect(within(goblinRow).getByLabelText("Goblin current HP")).toHaveValue("10");
    expect(within(goblinRow).queryByText("/ 12")).not.toBeInTheDocument();
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
        "ambush",
        expect.objectContaining({
          participant_key: "participant_b",
          current_hp: 20n,
        }),
      ),
    );
  });

  it("keeps encounter lifecycle actions in the edit modal", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(await screen.findByRole("button", { name: "Edit encounter" }));
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

    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith("actors:goblin"),
    );
    const linkedRuleButton = (await screen.findByText("Linked Rule")).closest("button");
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
    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith("actors:goblin"),
    );
    const linkedRuleButtonAfterReselect = (
      await screen.findByText("Linked Rule")
    ).closest("button");
    if (!linkedRuleButtonAfterReselect) {
      throw new Error("Linked Rule button was not rendered after reselection");
    }
    fireEvent.click(linkedRuleButtonAfterReselect);
    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith("rules:linked"),
    );

    fireEvent.click(screen.getByLabelText("Reference preview overlay"));

    await waitFor(() =>
      expect(screen.queryByLabelText("Reference preview")).not.toBeInTheDocument(),
    );
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
    apiMocks.getEncounter.mockResolvedValue(
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
    apiMocks.getEncounter.mockResolvedValue(
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
        "ambush",
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
        "ambush",
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
        "ambush",
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
        "ambush",
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
        "ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          current_hp: 12n,
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
        encounter_ref: "ambush",
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
    expect(screen.getByLabelText("Condition value")).toHaveValue("1");
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
        "ambush",
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
        "ambush",
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
        "ambush",
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
        "ambush",
        "participant_a",
        7n,
      ),
    );
  }, 10_000);
});

async function selectOption(input: HTMLElement, option: string) {
  fireEvent.mouseDown(input);
  const options = await screen.findAllByText(option);
  const visibleOption = options.find((element) =>
    element.classList.contains("ant-select-item-option-content"),
  );
  fireEvent.click(visibleOption ?? options[0]);
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
  const linked = recordKey === "rules:linked";
  const condition = recordKey.startsWith("conditionitems:");
  return {
    record_key: recordKey,
    title: condition
      ? "Frightened Condition"
      : linked
        ? "Linked Rule"
        : "Goblin Warrior",
    kind: condition || linked ? "rule" : "creature",
    presentation: {
      record_key: recordKey,
      kind: condition || linked ? "rule" : "creature",
      title: condition
        ? "Frightened Condition"
        : linked
          ? "Linked Rule"
          : "Goblin Warrior",
      identity: [],
      badges: [],
      sections:
        linked || condition
          ? []
          : [
              {
                kind: "references",
                title: "References",
                blocks: [
                  {
                    kind: "relationships",
                    content: [
                      {
                        kind: "reference",
                        label: "Linked Rule",
                        record_key: "rules:linked",
                      },
                    ],
                  },
                ],
              },
            ],
    },
  };
}

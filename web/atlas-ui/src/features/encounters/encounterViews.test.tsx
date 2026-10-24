import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import {
  spellFixture,
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
  SearchResultsRequest,
  RuntimeCountView,
  TurnResourcesView,
} from "../../generated/atlas";
import { EncounterDetailView } from "./EncounterDetailView";
import { EncounterEditView } from "./EncounterEditView";
import { EncounterIndexView } from "./EncounterIndexView";
import { EditEncounterForm } from "./EncounterModals";
import { atlasRoutePath, currentAtlasRoute } from "../../app/routes";

const apiMocks = vi.hoisted(() => ({
  applyEncounterSpellAction: vi.fn(),
  applyEncounterHpAction: vi.fn(),
  getEncounterPersistentReview: vi.fn(),
  prepareEncounterPersistentDamage: vi.fn(),
  applyEncounterPersistentDamage: vi.fn(),
  recoverEncounterPersistentDamage: vi.fn(),
  getEncounterConditionLifecycleActions: vi.fn(),
  getEncounterHpLifecycle: vi.fn(),
  getEncounterTurnReview: vi.fn(),
  advanceEncounterTurn: vi.fn(),
  handleEncounterOutgoingTurn: vi.fn(),
  addEncounterManualParticipant: vi.fn(),
  addEncounterParticipantCondition: vi.fn(),
  addEncounterRecordParticipant: vi.fn(),
  createEncounter: vi.fn(),
  deleteEncounter: vi.fn(),
  getEncounter: vi.fn(),
  getEncounterConditionDefinitions: vi.fn(),
  previewEncounterCondition: vi.fn(),
  getEncounterConditionConsequences: vi.fn(),
  encounterCastingReminders: vi.fn(),
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
  applyEncounterSpellAction: apiMocks.applyEncounterSpellAction,
  applyEncounterHpAction: apiMocks.applyEncounterHpAction,
  getEncounterPersistentReview: apiMocks.getEncounterPersistentReview,
  prepareEncounterPersistentDamage: apiMocks.prepareEncounterPersistentDamage,
  applyEncounterPersistentDamage: apiMocks.applyEncounterPersistentDamage,
  recoverEncounterPersistentDamage: apiMocks.recoverEncounterPersistentDamage,
  getEncounterConditionLifecycleActions: apiMocks.getEncounterConditionLifecycleActions,
  getEncounterHpLifecycle: apiMocks.getEncounterHpLifecycle,
  getEncounterTurnReview: apiMocks.getEncounterTurnReview,
  advanceEncounterTurn: apiMocks.advanceEncounterTurn,
  handleEncounterOutgoingTurn: apiMocks.handleEncounterOutgoingTurn,
  addEncounterManualParticipant: apiMocks.addEncounterManualParticipant,
  addEncounterParticipantCondition: apiMocks.addEncounterParticipantCondition,
  addEncounterRecordParticipant: apiMocks.addEncounterRecordParticipant,
  createEncounter: apiMocks.createEncounter,
  deleteEncounter: apiMocks.deleteEncounter,
  getEncounter: apiMocks.getEncounter,
  getEncounterConditionDefinitions: apiMocks.getEncounterConditionDefinitions,
  previewEncounterCondition: apiMocks.previewEncounterCondition,
  getEncounterConditionConsequences: apiMocks.getEncounterConditionConsequences,
  encounterCastingReminders: apiMocks.encounterCastingReminders,
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

function setupPersistentReview(damageApplied = false) {
  const participant = serverEncounter.participants[0];
  participant.conditions[0] = {
    ...participant.conditions[0],
    name: "Persistent Damage",
    parameters: {
      kind: "persistent_damage",
      formula: "1d6",
      damage_type: "fire",
      multiplier: "normal",
      recovery: { kind: "standard" },
      episode_id: "fire",
    },
  };
  participant.persistent_damage_groups = [
    {
      episode_id: "fire",
      damage_type: "fire",
      selected_source_id: 7n,
      sources: [
        { condition_id: 7n, formula: "1d6", multiplier: "normal", recovery_dc: 15n },
      ],
    },
  ];
  const damageResult = {
    encounter_key: "encounter_ambush",
    request_id: "saved-damage",
    total: 4n,
    hardness_absorbed: 0n,
    groups: [
      {
        episode_id: "fire",
        damage_type: "fire",
        source_condition_id: 7n,
        roll: null,
        amount: 4n,
        table_resolved: true,
      },
    ],
  };
  apiMocks.prepareEncounterPersistentDamage.mockImplementation(
    async (_ref, request) => ({
      complete: request.groups.length === 1,
      recovery: null,
      damage: {
        ...damageResult,
        total: request.groups[0]?.damage.amount ?? 0n,
        groups: request.groups.length
          ? [
              {
                ...damageResult.groups[0],
                amount: request.groups[0].damage.amount ?? 0n,
              },
            ]
          : [],
      },
    }),
  );
  const state: {
    damageApplied: boolean;
    recoveryResults: import("../../generated/atlas").PersistentRecoveryResultView[];
    afterSave?: () => void;
  } = { damageApplied, recoveryResults: [] };
  apiMocks.getEncounterTurnReview.mockImplementation(async () => ({
    encounter_key: "encounter_ambush",
    expected_revision: serverEncounter.revision,
    current_participant_key: "participant_a",
    next_participant_key: "participant_b",
    reminders: [],
    outgoing: apiMocks.advanceEncounterTurn.mock.calls.length
      ? {
          boundary_key: "original-turn",
          participant_key: "participant_a",
          participant_name: "Goblin",
          round_number: 1n,
          persistent_damage_groups: participant.persistent_damage_groups,
          reminders: state.recoveryResults.length
            ? []
            : [
                {
                  kind: "persistent_damage",
                  participant_key: "participant_a",
                  participant_name: "Goblin",
                  condition_id: 7n,
                  condition_name: "Persistent Damage",
                  note: null,
                },
              ],
        }
      : null,
    next_turn_unavailable_reason: null,
  }));
  apiMocks.getEncounterPersistentReview.mockImplementation(async () => ({
    encounter_revision: serverEncounter.revision,
    damage_groups: state.recoveryResults.some((result) => result.success)
      ? []
      : participant.persistent_damage_groups,
    round_number: 1n,
    damage_applied: state.damageApplied,
    episodes: state.recoveryResults.length
      ? []
      : [
          {
            episode_id: "fire",
            end_turn_recovery_available: true,
            end_turn_roll_available: state.damageApplied,
            end_turn_roll_unavailable_reason: state.damageApplied
              ? null
              : "Record this turn’s damage before rolling recovery.",
            end_turn_recovery_unavailable_reason: null,
          },
        ],
    damage_result: state.damageApplied ? damageResult : null,
    recovery_results: state.recoveryResults,
  }));
  apiMocks.applyEncounterPersistentDamage.mockImplementation(async () => {
    state.damageApplied = true;
    serverEncounter = { ...serverEncounter, revision: serverEncounter.revision + 1n };
    state.afterSave?.();
    return damageResult;
  });
  apiMocks.recoverEncounterPersistentDamage.mockImplementation(async () => {
    const result: import("../../generated/atlas").PersistentRecoveryResultView = {
      encounter_key: "encounter_ambush",
      request_id: "saved-recovery",
      episode_id: "fire",
      damage_type: "fire",
      dc: 15n,
      roll: {
        formula: "1d20",
        total: 17,
        expression: {
          kind: "dice",
          total: 17,
          dice: [{ sides: 20, value: 17, kept: true }],
        },
      },
      success: true,
    };
    state.recoveryResults.push(result);
    serverEncounter = {
      ...serverEncounter,
      revision: serverEncounter.revision + 1n,
      participants: serverEncounter.participants.map((p) =>
        p.participant_key === "participant_a"
          ? { ...p, conditions: [], persistent_damage_groups: [] }
          : p,
      ),
    };
    state.afterSave?.();
    return result;
  });
  return state;
}
async function submitPersistentDamage(review: HTMLElement, amount: string) {
  fireEvent.click(await within(review).findByText("Table results"));
  fireEvent.change(await within(review).findByLabelText("fire damage amount"), {
    target: { value: amount },
  });
  fireEvent.click(
    await within(review).findByRole("button", { name: /^Apply \d+ damage$/ }),
  );
  await within(review).findByText(/Total: .* damage applied/);
}
async function submitPersistentRecovery(review: HTMLElement) {
  const recover = await within(review).findByRole("button", {
    name: "Roll fire recovery",
  });
  await waitFor(() => expect(recover).toBeEnabled());
  fireEvent.click(recover);
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  await waitFor(() =>
    expect(apiMocks.recoverEncounterPersistentDamage).toHaveBeenCalledOnce(),
  );
}

describe("encounter views", () => {
  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.clearAllMocks();
    Object.values(apiMocks).forEach((mock) => mock.mockReset());
    apiMocks.getParties.mockResolvedValue({ parties: [] });
    apiMocks.getEmbeddedReading.mockImplementation(
      async ({ parent, artifact_generation }) => ({
        parent,
        artifact_generation,
        availability: "value",
        total: 0,
        start: 0,
        end: 0,
        entries: [],
        previous: null,
        next: null,
      }),
    );
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ md: true, xl: true });
    history.replaceState(null, "", "/encounters");
    apiMocks.getEncounters.mockResolvedValue(encounterIndexFixture());
    setEncounterDetail(encounterDetailFixture());
    apiMocks.getEncounterConditionConsequences.mockResolvedValue([]);
    apiMocks.getEncounterHpLifecycle.mockResolvedValue([]);
    apiMocks.getEncounterConditionLifecycleActions.mockResolvedValue({ actions: [] });
    apiMocks.getEncounterPersistentReview.mockResolvedValue({
      encounter_revision: serverEncounter.revision,
      damage_groups: [],
      round_number: 1n,
      damage_applied: false,
      episodes: [],
      damage_result: null,
      recovery_results: [],
    });
    apiMocks.previewEncounterCondition.mockResolvedValue({ consequences: [] });
    apiMocks.getEncounterConditionDefinitions.mockResolvedValue(
      conditionDefinitionsFixture(),
    );
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) =>
      Promise.resolve(recordDetailFixture(recordKey)),
    );
    apiMocks.searchPage.mockImplementation(async (request: SearchResultsRequest) => ({
      ...searchResultsFixture(),
      sort:
        request.sort ??
        (request.criteria.kind === "list_records"
          ? { kind: "attributes", rules: [{ field: "name", direction: "asc" }] }
          : { kind: "relevance" }),
    }));
    apiMocks.getEncounterTurnReview.mockImplementation(() =>
      Promise.resolve({
        encounter_key: "encounter_ambush",
        expected_revision: serverEncounter.revision,
        current_participant_key: serverEncounter.current_turn_participant_key ?? null,
        next_participant_key: "participant_b",
        reminders: [],
        outgoing: null,
        next_turn_unavailable_reason: null,
      }),
    );
    const receipt = {
      encounter_key: encounterIndexFixture().encounters[0].encounter_key,
    };
    for (const mutation of [
      apiMocks.applyEncounterSpellAction,
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

  it("keeps the approved inspector sections stable through repeated rerenders", async () => {
    const view = render(
      <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />,
      { wrapper: queryClientWrapper() },
    );
    await screen.findByLabelText("HP summary");
    for (let count = 0; count < 4; count += 1) {
      expect(screen.queryByRole("button", { name: "Relationships" })).toBeNull();
      expect(screen.queryByRole("button", { name: "Action context" })).toBeNull();
      expect(screen.queryByRole("button", { name: "Strike history" })).toBeNull();
      view.rerender(
        <EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />,
      );
    }
  });

  it("casts in the existing spell reference without opening another modal", async () => {
    const initial = encounterDetailFixture();
    const actor = initial.participants[0].presentation!.body;
    if (actor.kind !== "creature") throw new Error("expected creature");
    const spell = {
      ...actor.value.activities.value![0],
      kind: "spell" as const,
      title: "Command",
      casting_ranks: [1],
      base_rank: numberFact(1),
      authored_cast_rank: numberFact(1),
      casting_entry: null,
    };
    spell.navigation = {
      ...spell.navigation,
      source_fingerprint: "current-spell",
      spell_rank: 1,
    };
    actor.value.activities = knownFact([spell]);
    initial.participants[0].spellcasting = {
      resources: [],
      spells: [
        {
          navigation: spell.navigation,
          entry_id: "entry",
          title: "Command",
          expended: null,
          ranks: [{ rank: 1, resource_key: null, unavailable_reason: null }],
        },
      ],
    };
    setEncounterDetail(initial);
    const detail = detailFixture("actors:goblin");
    detail.selected = { ...spell.navigation, source_fingerprint: null };
    detail.presentation.body = { kind: "spell", value: spellFixture() };
    apiMocks.getEncounterReference.mockResolvedValue({
      ...detail,
      artifact_generation: initial.artifact_generation,
    });
    apiMocks.encounterCastingReminders.mockResolvedValue({
      advisories: [],
      spell_title: "Command",
      rank: 1,
      cost: null,
      issues: [],
      confusion: null,
      targeting: null,
      disruption: null,
      expected_encounter_revision: 0n,
      expected_participant_revision: 0n,
    });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("link", { name: "Command" }));
    const reference = (
      await screen.findByLabelText("Reference content")
    ).closest<HTMLElement>('[role="dialog"]')!;
    fireEvent.click(await within(reference).findByRole("button", { name: "Cast" }));
    await waitFor(() =>
      expect(apiMocks.applyEncounterSpellAction).toHaveBeenCalledWith(
        "encounter_ambush",
        expect.objectContaining({
          action: { kind: "cast", selection: spell.navigation },
        }),
      ),
    );
    expect(screen.getByLabelText("Reference content")).toBeInTheDocument();
    expect(screen.queryByRole("dialog", { name: "Goblin: Command" })).toBeNull();
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

  it("loads the participant condition catalog with a bigint encounter revision", async () => {
    setEncounterDetail({ ...encounterDetailFixture(), revision: 9007199254740993n });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByLabelText("HP summary")).toHaveTextContent("10/12");
    await waitFor(() =>
      expect(apiMocks.getEncounterConditionDefinitions).toHaveBeenCalledWith(
        "ambush",
        "participant_a",
      ),
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
        request_id: expect.any(String),
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
        request_id: expect.any(String),
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
    const initial = encounterDetailFixture();
    const body = initial.participants[0].presentation!.body;
    if (body.kind !== "creature") throw new Error("Expected creature fixture");
    initial.participants[0].turn_resources = turnResourcesFixture(
      2n,
      body.value.runtime!.action_budget!.actions,
    );
    setEncounterDetail(initial);
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const runtimeSection = await screen.findByRole("region", { name: "Turn" });
    expect(
      within(runtimeSection).getByRole("button", { name: "Spend action 1" }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(
      within(runtimeSection).getByRole("button", { name: "Spend action 2" }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(
      within(runtimeSection).queryByRole("button", { name: "Spend action 3" }),
    ).toBeNull();
    expect(
      within(runtimeSection).getByRole("button", {
        name: "Actions: details and remaining actions",
      }),
    ).toBeVisible();
    expect(
      within(runtimeSection).getByRole("button", { name: "Spend reaction" }),
    ).toBeVisible();
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
      .getByRole("heading", { name: "Strikes" })
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
    const initial = encounterDetailFixture();
    const body = initial.participants[1].presentation!.body;
    if (body.kind !== "creature") throw new Error("Expected creature fixture");
    initial.participants[1].turn_resources = turnResourcesFixture(
      3n,
      body.value.runtime!.action_budget!.actions,
      false,
    );
    setEncounterDetail(initial);
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click((await screen.findByText("Kyra")).closest("button")!);

    const runtimeSection = await screen.findByRole("region", { name: "Turn" });
    expect(
      within(runtimeSection).getByRole("button", { name: "Spend action 3" }),
    ).toBeDisabled();
    expect(
      within(runtimeSection).getByRole("button", { name: "Spend reaction" }),
    ).toBeEnabled();
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
      summary.compareDocumentPosition(
        screen.getByRole("heading", { name: "Strikes" }),
      ) & Node.DOCUMENT_POSITION_FOLLOWING,
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

  it("preserves maximum override and reset intent through HP controls", async () => {
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
  });

  it("preserves adjustment intent through participant settings", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
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
      await screen.findByRole("link", { name: "View creature reference" }),
    );
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
    expect(await screen.findByLabelText("Reference content")).toBeInTheDocument();

    const kyraRow = (await screen.findByText("Kyra")).closest("button");
    if (!kyraRow) {
      throw new Error("Kyra roster row was not rendered");
    }
    fireEvent.click(kyraRow);
    await waitFor(() =>
      expect(screen.queryByLabelText("Reference content")).not.toBeInTheDocument(),
    );

    fireEvent.click(
      within(rosterRow("Goblin")).getByRole("button", {
        name: /^Goblin(?:, Current turn)?$/,
      }),
    );

    fireEvent.click(
      await screen.findByRole("link", { name: "View creature reference" }),
    );
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

    expect(await screen.findByLabelText("Reference content")).toBeInTheDocument();
  });

  it("dismisses a linked record preview with Escape", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(
      await screen.findByRole("link", { name: "View creature reference" }),
    );
    const linkedRule = (await screen.findByText("Linked Rule")).closest("a[href]");
    if (!linkedRule) throw new Error("Linked Rule native link was not rendered");
    fireEvent.click(linkedRule);
    expect(await screen.findByLabelText("Reference content")).toBeInTheDocument();

    const dialog = screen
      .getByLabelText("Reference content")
      .closest<HTMLElement>('[role="dialog"]')!;
    const heading = within(dialog).getByRole("heading", { level: 2 });
    heading.focus();
    fireEvent.keyDown(heading, { key: "Escape", keyCode: 27 });

    await waitFor(() =>
      expect(screen.queryByLabelText("Reference content")).not.toBeInTheDocument(),
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
    const preview = await screen.findByLabelText("Reference content");
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
    expect(apiMocks.getEmbeddedReading).toHaveBeenCalledTimes(1);
    expect(
      apiMocks.getEmbeddedReading.mock.calls.every(
        ([request]) => request.selection.kind === "actor_content",
      ),
    ).toBe(true);
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
    expect(await screen.findByLabelText("Reference content")).toBeInTheDocument();
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
      left: "64.70588235294119%",
      width: "35.294117647058826%",
    });
    expect(temporarySegment).toHaveStyle({
      left: "35.294117647058826%",
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
        request_id: expect.any(String),
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

  it("advances first and applies outgoing damage and recovery to their original boundary", async () => {
    const state = setupPersistentReview();
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
    const review = await screen.findByRole("dialog", { name: "Turn reminders" });
    await submitPersistentDamage(review, "4");
    expect(apiMocks.applyEncounterPersistentDamage).toHaveBeenCalledWith(
      "encounter_ambush",
      expect.objectContaining({
        request_id: expect.any(String),
        expected_encounter_revision: 0n,
        groups: [
          {
            episode_id: "fire",
            damage: { kind: "table_result", amount: 4n },
          },
        ],
      }),
    );
    await submitPersistentRecovery(review);
    await screen.findByText("Recovered · DC 15");
    expect(screen.getByRole("status", { name: "1d20: 17" })).toBeVisible();
    expect(review).toBeVisible();
    fireEvent.click(within(review).getByRole("button", { name: "Close" }));
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Turn reminders" }),
      ).not.toBeInTheDocument(),
    );
    expect(apiMocks.recoverEncounterPersistentDamage).toHaveBeenCalledWith(
      "encounter_ambush",
      expect.objectContaining({
        mode: "end_turn",
        episode_id: "fire",
        expected_encounter_revision: 1n,
        boundary_key: "original-turn",
      }),
    );
    expect(state.recoveryResults).toHaveLength(1);
    expect(apiMocks.applyEncounterHpAction).not.toHaveBeenCalled();
    expect(apiMocks.removeEncounterParticipantCondition).not.toHaveBeenCalled();
    expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledOnce();
    expect(apiMocks.applyEncounterPersistentDamage.mock.calls[0][1].boundary_key).toBe(
      "original-turn",
    );
  });

  it("keeps the outgoing creature visible through reminder updates and follows the current turn on dismissal", async () => {
    setupPersistentReview();
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
    await screen.findByRole("button", { name: "Next turn" });
    // A different manually inspected participant must not become the reminder context.
    fireEvent.click(document.querySelector('[data-participant-key="participant_b"]')!);
    fireEvent.click(screen.getAllByRole("button", { name: "Next turn" })[0]);
    const review = await screen.findByRole("dialog", { name: "Turn reminders" });
    const selected = () =>
      document
        .querySelector('[data-participant-key="participant_a"]')
        ?.closest(".encounter-roster__row");
    expect(serverEncounter.current_turn_participant_key).toBe("participant_b");
    expect(selected()).toHaveClass("encounter-roster__row--selected");
    await submitPersistentDamage(review, "4");
    expect(selected()).toHaveClass("encounter-roster__row--selected");
    fireEvent.click(within(review).getByRole("button", { name: "Close" }));
    await waitFor(() =>
      expect(
        document
          .querySelector('[data-participant-key="participant_b"]')
          ?.closest(".encounter-roster__row"),
      ).toHaveClass("encounter-roster__row--selected"),
    );
    expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledOnce();
  });

  it("closing persistent reminders keeps saved damage and never advances twice", async () => {
    setupPersistentReview();
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
    const review = await screen.findByRole("dialog", { name: "Turn reminders" });
    await submitPersistentDamage(review, "2");
    fireEvent.click(within(review).getAllByRole("button", { name: "Close" })[0]);
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Turn reminders" }),
      ).not.toBeInTheDocument(),
    );
    expect(
      screen.queryByRole("button", { name: "Outgoing turn reminders" }),
    ).not.toBeInTheDocument();
    expect(apiMocks.applyEncounterPersistentDamage).toHaveBeenCalledOnce();
    expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledOnce();
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
    await clickConditionApply(dialog);
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
    expect(within(dialog).getByRole("button", { name: "Apply" })).toBeDisabled();
    fireEvent.click(reload);
    expect(within(dialog).getByLabelText("Duration amount")).toHaveValue("1");
    fireEvent.change(within(dialog).getByLabelText("Condition note"), {
      target: { value: "Reviewed note" },
    });
    await clickConditionApply(dialog);
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
    await clickConditionApply();
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
      await within(dialog).findByText("Enter a whole number, one or greater.");
      expect(apiMocks.addEncounterParticipantCondition).toHaveBeenCalledOnce();
    }
    fireEvent.change(value, { target: { value: "2" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Add duration" }));
    await selectOption(conditionCombobox("Duration unit"), "minutes");
    const duration = within(dialog).getByLabelText("Duration amount");
    fireEvent.change(duration, { target: { value: "3" } });
    fireEvent.change(duration, { target: { value: "" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Remove duration" }));
    await waitFor(() => expect(duration).not.toBeVisible());
    await clickConditionApply(dialog);
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
    await clickConditionApply(dialog);
    const retry = await within(dialog).findByRole("button", {
      name: "Retry encounter refresh",
    });
    expect(within(dialog).getByLabelText("Condition value")).toHaveValue("1");
    expect(within(dialog).getByRole("button", { name: "Apply" })).toBeDisabled();
    fireEvent.click(retry);
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Add condition" }),
      ).not.toBeInTheDocument(),
    );
    expect(apiMocks.addEncounterParticipantCondition).toHaveBeenCalledOnce();
    expect(apiMocks.getEncounter).toHaveBeenCalledTimes(3);
  });

  it("keeps a saved outgoing damage write through failed canonical refresh without advancing twice", async () => {
    const state = setupPersistentReview();
    state.afterSave = () =>
      apiMocks.getEncounter.mockRejectedValueOnce(new Error("Owner read unavailable"));
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
    const dialog = await screen.findByRole("dialog", { name: "Turn reminders" });
    await submitPersistentDamage(dialog, "3");
    fireEvent.click(
      await within(dialog).findByRole("button", { name: "Retry encounter refresh" }),
    );
    await waitFor(() =>
      expect(
        within(dialog).queryByRole("button", { name: "Retry encounter refresh" }),
      ).not.toBeInTheDocument(),
    );
    expect(apiMocks.applyEncounterPersistentDamage).toHaveBeenCalledOnce();
    expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledOnce();
  });

  it.each(["damage", "recovery"] as const)(
    "retries failed reminder reads after saved %s without replaying writes or advancement",
    async (action) => {
      const state = setupPersistentReview(action === "recovery");
      const originalReview = apiMocks.getEncounterTurnReview.getMockImplementation()!;
      state.afterSave = () =>
        apiMocks.getEncounterTurnReview.mockRejectedValue(
          new Error("Review unavailable"),
        );
      render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
        wrapper: queryClientWrapper(),
      });
      fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
      const dialog = await screen.findByRole("dialog", { name: "Turn reminders" });
      if (action === "damage") await submitPersistentDamage(dialog, "4");
      else await submitPersistentRecovery(dialog);
      const retry = await within(dialog).findByRole("button", {
        name: "Retry turn review",
      });
      apiMocks.getEncounterTurnReview.mockImplementation(originalReview);
      fireEvent.click(retry);
      await waitFor(() =>
        expect(
          within(dialog).queryByText("Review unavailable"),
        ).not.toBeInTheDocument(),
      );
      expect(apiMocks.applyEncounterPersistentDamage).toHaveBeenCalledTimes(
        action === "damage" ? 1 : 0,
      );
      expect(apiMocks.recoverEncounterPersistentDamage).toHaveBeenCalledTimes(
        action === "recovery" ? 1 : 0,
      );
      expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledOnce();
    },
  );

  it("reconciles a stale turn without silently replaying advancement", async () => {
    apiMocks.advanceEncounterTurn.mockImplementationOnce(async () => {
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
    });
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
    await screen.findAllByText(
      "Encounter changed. Review the refreshed values before retrying your edit.",
    );
    expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledOnce();
    expect(
      screen.queryByRole("dialog", { name: "Turn reminders" }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Next turn" }));
    await waitFor(() =>
      expect(apiMocks.advanceEncounterTurn).toHaveBeenNthCalledWith(
        2,
        "encounter_ambush",
        { expected_revision: 8n, request_id: expect.any(String) },
      ),
    );
    expect(apiMocks.setEncounterTurn).not.toHaveBeenCalled();
  });

  it("keeps ended, manual and unavailable timing facts with direct guarded removal after advancing", async () => {
    const reminders = [
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
    ];
    apiMocks.getEncounterTurnReview.mockImplementation(async () => ({
      encounter_key: "encounter_ambush",
      expected_revision: serverEncounter.revision,
      current_participant_key: "participant_b",
      next_participant_key: "participant_a",
      reminders: [],
      next_turn_unavailable_reason: null,
      outgoing: apiMocks.advanceEncounterTurn.mock.calls.length
        ? {
            boundary_key: "outgoing",
            participant_key: "participant_a",
            participant_name: "Goblin",
            round_number: 1n,
            persistent_damage_groups: [],
            reminders,
          }
        : null,
    }));
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Next turn" }));
    const dialog = await screen.findByRole("dialog", { name: "Turn reminders" });
    expect(within(dialog).getByText("Duration ended")).toBeInTheDocument();
    expect(
      within(dialog).getByText("5 minutes: check this duration at the table."),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText(
        "Round timing is not configured. Edit the condition to choose whose turn and when its rounds count down.",
      ),
    ).toBeInTheDocument();
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Remove Frightened from Goblin" }),
    );
    await waitFor(() =>
      expect(apiMocks.removeEncounterParticipantCondition).toHaveBeenCalledOnce(),
    );
    expect(apiMocks.advanceEncounterTurn).toHaveBeenCalledOnce();
    expect(
      screen.queryByRole("dialog", { name: "Remove Frightened from Goblin?" }),
    ).not.toBeInTheDocument();
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
    ).getByRole("button", { name: "Increase Condition value" });
    fireEvent.click(plus);
    expect(screen.getByLabelText("Condition value")).toHaveValue("2");
    fireEvent.click(screen.getByRole("button", { name: "Add duration" }));
    fireEvent.change(await screen.findByLabelText("Duration amount"), {
      target: { value: "3" },
    });
    await selectOption(conditionCombobox("Whose turn"), "Goblin’s turn");
    fireEvent.click(screen.getByRole("button", { name: /^(Add|Hide) details$/ }));
    fireEvent.change(screen.getByLabelText("Condition note"), {
      target: { value: "poison" },
    });
    await clickConditionApply();

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
    apiMocks.updateEncounterParticipantCondition.mockImplementation(
      async (_ref, participantKey, request) => {
        serverEncounter = {
          ...serverEncounter,
          participants: serverEncounter.participants.map((participant) =>
            participant.participant_key === participantKey
              ? {
                  ...participant,
                  conditions: participant.conditions.map((condition) =>
                    condition.condition_id === request.condition_id
                      ? { ...condition, ...request }
                      : condition,
                  ),
                }
              : participant,
          ),
        };
        return { encounter_key: serverEncounter.encounter.encounter_key };
      },
    );
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
    if (
      screen
        .getByRole("button", { name: /^(Add|Hide) details$/ })
        .getAttribute("aria-expanded") === "false"
    )
      fireEvent.click(screen.getByRole("button", { name: /^(Add|Hide) details$/ }));
    fireEvent.change(lastFieldByAriaLabel("Condition note"), {
      target: { value: "aura" },
    });
    await clickConditionApply();

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
    expect(screen.getByText("aura")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Remove Frightened" }));
    expect(screen.queryByRole("dialog", { name: "Remove Frightened?" })).toBeNull();

    await waitFor(() =>
      expect(apiMocks.removeEncounterParticipantCondition).toHaveBeenCalledWith(
        "encounter_ambush",
        "participant_a",
        7n,
        expect.objectContaining({
          request_id: expect.any(String),
          expected_encounter_revision: expect.any(BigInt),
          intent: "remove_effect",
        }),
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
    expect(screen.getByLabelText("HP summary")).toHaveTextContent(
      /9\s*\/\s*12\s*\+\s*5\s*temp/,
    );
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
    expect(
      await screen.findByText("Condition choices unavailable"),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add Condition" })).toBeDisabled();
    const choicesAlert = screen
      .getByText("Condition choices unavailable")
      .closest<HTMLElement>('[role="alert"]')!;
    fireEvent.click(within(choicesAlert).getByRole("button", { name: "Retry" }));
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
    apiMocks.updateEncounterParticipantCondition.mockRejectedValue(
      new Error("Condition unavailable"),
    );
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Edit Frightened" }));
    const note = await screen.findByLabelText("Condition note");
    fireEvent.change(note, { target: { value: "My note" } });
    await clickConditionApply();
    await waitFor(() =>
      expect(screen.getAllByText("Condition unavailable").length).toBeGreaterThan(0),
    );
    expect(note).toHaveValue("My note");
    expect(screen.getByRole("button", { name: "Apply" })).toBeInTheDocument();
    const previewCalls = apiMocks.previewEncounterCondition.mock.calls.length;
    const firstRequest = apiMocks.updateEncounterParticipantCondition.mock.calls[0][2];
    await clickConditionApply();
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipantCondition).toHaveBeenCalledTimes(2),
    );
    expect(apiMocks.previewEncounterCondition).toHaveBeenCalledTimes(previewCalls);
    expect(
      apiMocks.updateEncounterParticipantCondition.mock.calls[1][2].request_id,
    ).toBe(firstRequest.request_id);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
    );
    fireEvent.change(note, { target: { value: "Changed intent" } });
    await clickConditionApply();
    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipantCondition).toHaveBeenCalledTimes(3),
    );
    expect(apiMocks.previewEncounterCondition).toHaveBeenCalledTimes(previewCalls);
    expect(
      apiMocks.updateEncounterParticipantCondition.mock.calls[2][2].request_id,
    ).not.toBe(firstRequest.request_id);
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
    standard_persistent_recovery_dc: 15n,
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
    editor_kind: "ordinary",
    name,
    automation_level: automationLevel,
    applies_to: ["creature"],
    categories: [automationLevel === "automated" ? "stat_modifier" : "runtime_state"],
    supports_duration_instead_of_value: false,
    has_value: hasValue,
    ...(hasValue ? { default_value: 1n } : {}),
  };
}

async function clickConditionApply(dialog?: HTMLElement) {
  await act(async () => {});
  const scope = dialog ? within(dialog) : screen;
  const apply = scope.getByRole("button", { name: "Apply" });
  await waitFor(() => expect(apply).toBeEnabled());
  fireEvent.click(apply);
}

function turnResourcesFixture(
  actions: bigint,
  forecast: RuntimeCountView,
  current = true,
): TurnResourcesView {
  return {
    turn_key: "turn:test",
    round_number: 1n,
    regular_total: actions,
    regular_remaining: actions,
    extra_total: 0n,
    extra_remaining: 0n,
    extra_uses: [],
    reactions_total: 1n,
    reactions_remaining: 1n,
    start_results: [],
    pending_reason: null,
    forecast,
    can_edit_actions: current,
    can_spend_actions: {
      available: current,
      reason: current ? undefined : "This creature's turn has not started.",
    },
    can_spend_reactions: { available: true },
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
            immunity_override: false,
            parameters: { kind: "none" },
            scope: { kind: "everyone" },
            extra_action: { kind: "unspecified" },
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
    knockout_options: [],
    effective_max_hp: {
      state: overrides.max_hp === undefined ? "missing" : "value",
      value: overrides.max_hp === undefined ? null : Number(overrides.max_hp),
      adjustment: null,
    },
    revision: 0n,
    persistent_damage_groups: [],
    track_dying: false,
    mortality: "alive",
    base_death_threshold: 4n,
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
    relationship_clauses: [],
    sort: { kind: "relevance" } as const,
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
        relationship_evidence: [],
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
      senses: [],
      contextual_armor_class: [],
      action_budget: {
        quickened_sources: [],
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

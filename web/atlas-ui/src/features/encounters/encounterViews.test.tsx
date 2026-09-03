import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  EncounterConditionCatalogView,
  EncounterConditionDefinitionView,
  EncounterDetailView as EncounterDetailViewDto,
  EncounterIndexView as EncounterIndexViewDto,
  EncounterParticipantView,
  RecordDetailView,
  ResultWindowPage,
} from "../../generated/atlas";
import {
  creatureSurfaceFixture,
  encounterParticipantFixture,
  encounterRuntimeFixture,
  recordDetailFixture as typedRecordDetailFixture,
  recordSummaryFixture as typedRecordSummaryFixture,
} from "../../test/recordFixtures";
import { EncounterDetailView } from "./EncounterDetailView";
import { EncounterEditView } from "./EncounterEditView";
import { EncounterIndexView } from "./EncounterIndexView";
import { EncounterInspectorPane } from "./EncounterInspectorPane";

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
  mutateEncounterSpellCast: vi.fn(),
  openResultWindow: vi.fn(),
  removeEncounterParticipant: vi.fn(),
  removeEncounterParticipantCondition: vi.fn(),
  reorderEncounterParticipant: vi.fn(),
  resetEncounterParticipant: vi.fn(),
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
  mutateEncounterSpellCast: apiMocks.mutateEncounterSpellCast,
  openResultWindow: apiMocks.openResultWindow,
  removeEncounterParticipant: apiMocks.removeEncounterParticipant,
  removeEncounterParticipantCondition: apiMocks.removeEncounterParticipantCondition,
  reorderEncounterParticipant: apiMocks.reorderEncounterParticipant,
  resetEncounterParticipant: apiMocks.resetEncounterParticipant,
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
    apiMocks.mutateEncounterSpellCast.mockImplementation(
      (_encounterRef, participantKey, request) =>
        Promise.resolve({
          operation: request.operation,
          participant_key: participantKey,
          spell_occurrence_id: request.spell_occurrence_id,
          before: {
            spend_target: request.spend_target,
            available: true,
            state: { state_type: "at_will" },
          },
          after: {
            spend_target: request.spend_target,
            available: true,
            state: { state_type: "at_will" },
          },
          participant: encounterDetailFixture().participants[0],
        }),
    );
    apiMocks.resetEncounterParticipant.mockImplementation(
      (_encounterRef, participantKey) =>
        Promise.resolve({
          participant_key: participantKey,
          reset_domains: [
            "hit_points",
            "defeated",
            "conditions",
            "initiative_turn_state",
            "variant_adjustments",
            "action_budget",
            "spell_resources",
          ],
          preserved_domains: ["display_name", "notes", "visibility", "side"],
          cleared_current_turn: true,
          participant: encounterDetailFixture().participants[0],
        }),
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
    expect(
      (await screen.findAllByRole("heading", { name: "Kyra" })).length,
    ).toBeGreaterThan(0);
    expect(screen.getByText("Not current turn")).toBeVisible();
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

  it("renders runtime action budget and speed projections", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await screen.findByText("Turn Economy");
    const runtimeSection = screen
      .getByText("Turn Economy")
      .closest(".creature-sheet__panel");
    if (!(runtimeSection instanceof HTMLElement)) {
      throw new Error("Runtime section was not rendered");
    }

    expect(within(runtimeSection).getAllByText("Actions").length).toBeGreaterThan(0);
    expect(within(runtimeSection).getByText("2")).toBeInTheDocument();
    expect(within(runtimeSection).getByText(/base 3/)).toBeInTheDocument();
    expect(within(runtimeSection).getAllByText("Reactions").length).toBeGreaterThan(0);
    const movementSection = screen
      .getByText("Movement")
      .closest(".creature-sheet__panel");
    if (!(movementSection instanceof HTMLElement)) {
      throw new Error("Movement section was not rendered");
    }
    expect(within(movementSection).getAllByText("Land Speed").length).toBeGreaterThan(
      0,
    );
    expect(within(movementSection).getByText("15 ft")).toBeInTheDocument();
    expect(within(movementSection).getByText(/base 25 ft/)).toBeInTheDocument();
    const activitiesSection = screen
      .getByText("Actions & Abilities")
      .closest(".creature-sheet__panel");
    if (!(activitiesSection instanceof HTMLElement)) {
      throw new Error("Activities section was not rendered");
    }
    expect(within(activitiesSection).getByText("Claw")).toBeInTheDocument();
    expect(within(activitiesSection).getByText("1d6+2 slashing")).toBeInTheDocument();
  });

  it("explains adjusted runtime facts and preserves canonical context", async () => {
    const surface = recordSurfaceFixture({
      actions: 2,
      actionBase: 3,
      actionAdjustment: -1,
      speed: 15,
      speedBase: 25,
      speedAdjustment: -10,
    });
    if (surface.presentation.presentation_type !== "creature" || !surface.encounter) {
      throw new Error("Expected an encounter creature fixture");
    }
    surface.presentation.body.defenses = {
      ...surface.presentation.body.defenses!,
      armor_class_details: "+1 circumstance bonus against traps",
      hardness: 5,
      resistances: [
        {
          component_id: "resistance-fire",
          authored_order: 0,
          kind: "fire",
          amount: 5,
        },
      ],
    };
    surface.encounter.level = {
      label: "Level",
      base_value: 1,
      adjusted_value: 2,
      modifiers: [
        {
          provenance: {
            source: { source_type: "participant_variant", variant: "elite" },
          },
          label: "Elite level adjustment",
          modifier_type: "adjustment",
          value: 1,
        },
      ],
      provenance: runtimeProvenance,
    };
    surface.encounter.automation_limitations = [
      {
        code: "condition_damage_adjustment_partial",
        target: { target_type: "activity", activity_id: "claw" },
        message: "Damage adjustments for this action require adjudication.",
      },
    ];
    apiMocks.getEncounter.mockResolvedValue(
      encounterDetailFixture("participant_a", {
        note: "Keep the bridge blocked.",
        record_view: surface,
      }),
    );

    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByText("Current turn")).toBeInTheDocument();
    expect(screen.getByLabelText("Participant note")).toHaveValue(
      "Keep the bridge blocked.",
    );
    expect(screen.getByText("+1 circumstance bonus against traps")).toBeVisible();
    expect(screen.getByText(/fire 5/i)).toBeVisible();
    expect(screen.getByText("Common")).toBeVisible();
    expect(
      screen.getByText("Damage adjustments for this action require adjudication."),
    ).toBeVisible();
    expect(screen.getByText("Action or ability")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Level adjustment details" }));
    expect(await screen.findByText("Level details")).toBeInTheDocument();
    expect(screen.getByText("Elite level adjustment +1")).toBeInTheDocument();
    expect(screen.getByText("Elite variant")).toBeInTheDocument();

    fireEvent.mouseDown(document.body);
    fireEvent.click(screen.getByRole("button", { name: "Actions adjustment details" }));
    expect(await screen.findByText("Actions details")).toBeInTheDocument();
    expect(screen.getByText("Reduced actions regained -1")).toBeInTheDocument();
    expect(
      screen.getByText(/Applied to the next action-regain step\./),
    ).toBeInTheDocument();
    expect(screen.getByText("Base: 3")).toBeInTheDocument();
  }, 15_000);

  it("commits participant notes from the visible semantic note section", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    const note = await screen.findByLabelText("Participant note");
    fireEvent.change(note, { target: { value: "Focus fire on the front line." } });
    fireEvent.blur(note);

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          note: "Focus fire on the front line.",
        }),
      ),
    );
  });

  it("renders manual PC runtime state without inferred speed rows", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click((await screen.findByText("Kyra")).closest('[role="button"]')!);

    await screen.findByText("Turn Economy");
    const runtimeSection = screen
      .getByText("Turn Economy")
      .closest(".creature-sheet__panel");
    if (!(runtimeSection instanceof HTMLElement)) {
      throw new Error("Runtime section was not rendered");
    }
    expect(within(runtimeSection).getAllByText("Actions").length).toBeGreaterThan(0);
    expect(within(runtimeSection).getByText("3")).toBeInTheDocument();
    expect(within(runtimeSection).getAllByText("Reactions").length).toBeGreaterThan(0);
    expect(screen.queryByText("Land Speed")).not.toBeInTheDocument();
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
    expect(hpInput).toHaveValue("10");
    fireEvent.change(hpInput, { target: { value: "20" } });
    fireEvent.keyDown(hpInput, { key: "Enter" });

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipant).toHaveBeenCalledWith(
        "ambush",
        expect.objectContaining({
          participant_key: "participant_b",
          current_hp: 12,
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

  it("confirms before invoking the prepared creature reset handler", async () => {
    const detail = encounterDetailFixture();
    const participant = detail.participants[0];
    const onResetParticipant = vi.fn();
    render(
      <EncounterInspectorPane
        conditionDefinitions={conditionDefinitionsFixture().conditions}
        currentTurnParticipantKey={detail.current_turn_participant_key ?? null}
        onAddCondition={vi.fn()}
        onOpenRecordFullPage={vi.fn()}
        onRemoveCondition={vi.fn()}
        onResetParticipant={onResetParticipant}
        onSpellCast={vi.fn()}
        onUpdate={vi.fn()}
        onUpdateCondition={vi.fn()}
        participant={participant}
        participants={detail.participants}
        resetResult={null}
        spellCastResult={null}
      />,
      { wrapper: queryClientWrapper() },
    );

    fireEvent.click(
      await screen.findByRole("button", { name: `Reset ${participant.display_name}` }),
    );
    expect(onResetParticipant).not.toHaveBeenCalled();

    const confirmation = await screen.findByRole("dialog");
    expect(confirmation).toHaveTextContent(`Reset ${participant.display_name}?`);
    expect(confirmation).toHaveTextContent("mechanical encounter state");
    expect(confirmation).toHaveTextContent("creation baseline");
    expect(confirmation).toHaveTextContent(
      "Custom name, notes, and visibility are preserved.",
    );
    expect(confirmation).toHaveTextContent("This cannot be undone.");
    fireEvent.click(
      within(confirmation).getByRole("button", { name: "Reset creature" }),
    );

    await waitFor(() =>
      expect(onResetParticipant).toHaveBeenCalledWith(participant.participant_key),
    );
  });

  it("keeps legacy participants reset-unavailable without inventing a baseline", () => {
    const detail = encounterDetailFixture();
    const participant = {
      ...detail.participants[0],
      reset: {
        available: false,
        unavailable_reason: "missing_creation_baseline" as const,
      },
    };
    render(
      <EncounterInspectorPane
        conditionDefinitions={conditionDefinitionsFixture().conditions}
        currentTurnParticipantKey={detail.current_turn_participant_key ?? null}
        onAddCondition={vi.fn()}
        onOpenRecordFullPage={vi.fn()}
        onRemoveCondition={vi.fn()}
        onResetParticipant={vi.fn()}
        onSpellCast={vi.fn()}
        onUpdate={vi.fn()}
        onUpdateCondition={vi.fn()}
        participant={participant}
        participants={[participant, detail.participants[1]]}
        resetResult={null}
        spellCastResult={null}
      />,
      { wrapper: queryClientWrapper() },
    );

    expect(
      screen.getByRole("button", { name: `Reset ${participant.display_name}` }),
    ).toBeDisabled();
    expect(screen.getByText("Reset unavailable: no creation baseline")).toBeVisible();
  });

  it("invokes typed reset and reports restored and preserved domains", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(await screen.findByRole("button", { name: "Reset Goblin" }));
    const confirmation = await screen.findByRole("dialog");
    fireEvent.click(
      within(confirmation).getByRole("button", { name: "Reset creature" }),
    );

    await waitFor(() =>
      expect(apiMocks.resetEncounterParticipant).toHaveBeenCalledWith(
        "ambush",
        "participant_a",
        { confirmation: "reset_participant" },
      ),
    );
    expect(
      await screen.findByText("Creature reset to its creation baseline"),
    ).toBeVisible();
    expect(
      screen.getByText(/Restored: HP, maximum HP, and temporary HP/),
    ).toHaveTextContent("Preserved: custom name, notes, visibility, side");
    expect(screen.getByText(/Current turn was cleared/)).toBeVisible();
  });

  it("casts an encounter spell through its typed occurrence and target", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(await screen.findByText("Spellcasting"));
    fireEvent.click(await screen.findByText("Innate Spells"));
    fireEvent.click(await screen.findByRole("button", { name: "Cast Linked Rule" }));

    await waitFor(() =>
      expect(apiMocks.mutateEncounterSpellCast).toHaveBeenCalledWith(
        "ambush",
        "participant_a",
        {
          spell_occurrence_id: "linked-spell",
          spend_target: { target_type: "at_will" },
          operation: "cast_one",
        },
      ),
    );
    expect(await screen.findByText("Spell cast")).toBeVisible();
    expect(screen.getAllByText("At will")).not.toHaveLength(0);
  });

  it("opens and dismisses linked record previews inside the encounter record pane", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(apiMocks.getRecordDetail).not.toHaveBeenCalledWith("actors:goblin");
    fireEvent.click(await screen.findByText("Spellcasting"));
    fireEvent.click(await screen.findByText("Innate Spells"));
    const linkedRuleDisclosure = await screen.findByRole("button", {
      name: /Linked Rule.*1st/,
    });
    fireEvent.click(linkedRuleDisclosure);
    const linkedRuleLink = await screen.findByRole("link", {
      name: "Open full spell record",
    });
    if (!linkedRuleLink) {
      throw new Error("Linked Rule link was not rendered");
    }
    fireEvent.click(linkedRuleLink);

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
    expect(apiMocks.getRecordDetail).not.toHaveBeenCalledWith("actors:goblin");
    fireEvent.click(await screen.findByText("Spellcasting"));
    fireEvent.click(await screen.findByText("Innate Spells"));
    const linkedRuleDisclosureAfterReselect = await screen.findByRole("button", {
      name: /Linked Rule.*1st/,
    });
    if (linkedRuleDisclosureAfterReselect.getAttribute("aria-expanded") === "false") {
      fireEvent.click(linkedRuleDisclosureAfterReselect);
    }
    const linkedRuleLinkAfterReselect = await screen.findByRole("link", {
      name: "Open full spell record",
    });
    if (!linkedRuleLinkAfterReselect) {
      throw new Error("Linked Rule link was not rendered after reselection");
    }
    fireEvent.click(linkedRuleLinkAfterReselect);
    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith("rules:linked"),
    );

    fireEvent.mouseDown(document.body);

    await waitFor(() =>
      expect(screen.queryByLabelText("Reference preview")).not.toBeInTheDocument(),
    );
  }, 15_000);

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
        current_hp: 6,
        temporary_hp: 5,
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
        current_hp: 3,
        temporary_hp: 0,
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
          current_hp: 12,
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
          temporary_hp: 6,
          current_hp: 12,
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
          temporary_hp: 0,
          current_hp: 10,
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
          temporary_hp: 0,
          current_hp: 7,
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
          current_hp: 12,
        }),
      ),
    );
  });

  it("consumes temporary HP before current HP in the surfaced participant view", async () => {
    apiMocks.getEncounter.mockResolvedValue(
      encounterDetailFixture("participant_a", {
        record_view: recordSurfaceFixture(),
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
        "ambush",
        expect.objectContaining({
          participant_key: "participant_a",
          temporary_hp: 0,
          current_hp: 7,
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
        initiative: 18,
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
          value: 2,
          duration_rounds: 3,
          note: "poison",
        }),
      ),
    );
    expect(
      apiMocks.addEncounterParticipantCondition.mock.calls[0][1],
    ).not.toHaveProperty("name");
  }, 15_000);

  it("edits and removes conditions for the current participant", async () => {
    render(<EncounterDetailView route={{ kind: "encounter", slug: "ambush" }} />, {
      wrapper: queryClientWrapper(),
    });

    await screen.findByText("Frightened");

    const frightenedControls = screen.getByRole("group", {
      name: "Frightened value controls",
    });
    fireEvent.click(
      within(frightenedControls).getByRole("button", {
        name: "Increase Frightened value",
      }),
    );

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipantCondition).toHaveBeenLastCalledWith(
        "ambush",
        "participant_a",
        expect.objectContaining({
          condition_id: 7,
          name: "Frightened",
          value: 2,
        }),
      ),
    );

    fireEvent.click(
      within(frightenedControls).getByRole("button", {
        name: "Decrease Frightened value",
      }),
    );

    await waitFor(() =>
      expect(apiMocks.updateEncounterParticipantCondition).toHaveBeenLastCalledWith(
        "ambush",
        "participant_a",
        expect.objectContaining({
          condition_id: 7,
          name: "Frightened",
          value: 0,
        }),
      ),
    );

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
          condition_id: 7,
          name: "Frightened",
          value: 2,
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
          condition_id: 7,
          name: "Frightened",
          duration_rounds: 4,
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
        7,
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
        round_number: 1,
        participant_count: 2,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
      {
        encounter_key: "encounter_old",
        slug: "old-fight",
        name: "Old Fight",
        status: "archived",
        round_number: 1,
        participant_count: 1,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
    ],
  };
}

function conditionDefinitionsFixture(): EncounterConditionCatalogView {
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
): EncounterConditionDefinitionView {
  return {
    condition_ref: conditionRef,
    name,
    automation_level: automationLevel,
    applies_to: ["creature"],
    categories: [automationLevel === "automated" ? "stat_modifier" : "runtime_state"],
    has_value: hasValue,
    ...(hasValue ? { default_value: 1 } : {}),
  };
}

type ParticipantRuntimeOverrides = Partial<EncounterParticipantView> & {
  current_hp?: number;
  max_hp?: number;
  temporary_hp?: number;
};

function encounterDetailFixture(
  currentTurnParticipantKey: string | undefined = "participant_a",
  firstParticipantOverrides: ParticipantRuntimeOverrides = {},
): EncounterDetailViewDto {
  const {
    current_hp = 10,
    max_hp = 12,
    temporary_hp = 5,
    record_view,
    ...participantOverrides
  } = firstParticipantOverrides;
  const defaultSurface = recordSurfaceFixture({
    actions: 2,
    actionBase: 3,
    actionAdjustment: -1,
    speed: 15,
    speedBase: 25,
    speedAdjustment: -10,
  });
  const selectedSurface = record_view ?? defaultSurface;
  const selectedRuntime = selectedSurface.encounter ?? encounterRuntimeFixture();
  const goblinSurface = {
    ...selectedSurface,
    encounter: {
      ...selectedRuntime,
      vitals: {
        ...(selectedRuntime.vitals ?? {
          temporary_hp: 0,
        }),
        maximum_hp: numberFact("Maximum HP", max_hp),
        current_hp,
        temporary_hp,
      },
      conditions: [
        {
          condition_id: 7,
          condition_key: "conditionitems:TBSHQspnbcqxsmjL",
          name: "Frightened",
          value: 1,
          duration_rounds: 2,
          created_at: "2026-01-01T00:00:00Z",
          updated_at: "2026-01-01T00:00:00Z",
          provenance: runtimeProvenance,
        },
      ],
    },
  };

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
        initiative: 18,
        record_view: goblinSurface,
        ...participantOverrides,
      }),
      participantFixture({
        participant_key: "participant_b",
        display_name: "Kyra",
        participant_kind: "pc",
        status: "manual",
        side: "pc",
        initiative: 15,
        record_view: recordSurfaceFixture({
          kind: "pc",
          levelLabel: undefined,
          recordKey: "participant_b",
          title: "Kyra",
          traits: [],
          actions: 3,
          reactions: 1,
        }),
      }),
    ],
  };
}

function participantFixture(
  overrides: Partial<EncounterParticipantView>,
): EncounterParticipantView {
  return {
    ...encounterParticipantFixture(),
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
        record: typedRecordSummaryFixture("actors:goblin", "Goblin Warrior"),
        match_summary: undefined,
      },
    ],
  };
}

function recordSurfaceFixture({
  actionAdjustment,
  actionBase,
  actions,
  includeActivities = true,
  kind = "creature",
  levelLabel = "1",
  reactions = 1,
  recordKey = "actors:goblin",
  speed,
  speedAdjustment,
  speedBase,
  title = "Goblin Warrior",
  traits = ["Goblin", "Humanoid"],
}: {
  actionAdjustment?: number;
  actionBase?: number;
  actions?: number;
  includeActivities?: boolean;
  kind?: string;
  kindLabel?: string;
  levelLabel?: string;
  reactions?: number;
  recordKey?: string;
  speed?: number;
  speedAdjustment?: number;
  speedBase?: number;
  title?: string;
  traits?: string[];
} = {}) {
  const runtime = encounterRuntimeFixture({
    actions: actions ?? 3,
    currentHp: 10,
    maximumHp: 12,
    reactions,
    temporaryHp: 5,
  });
  const actionBudget = runtime.action_budget;
  const encounter = {
    ...runtime,
    ...(actionBudget && actions !== undefined
      ? {
          action_budget: {
            ...actionBudget,
            actions: {
              ...actionBudget.actions,
              base_value: actionBase ?? actions,
              adjusted_value: actions,
              segments: [
                {
                  label: "Base",
                  value: actionBase ?? actions,
                  restricted: false,
                },
              ],
              ...(actionAdjustment === undefined
                ? {}
                : {
                    adjustments: [
                      {
                        provenance: runtimeProvenance,
                        label: "Reduced actions regained",
                        value: actionAdjustment,
                        reason: "Applied to the next action-regain step.",
                      },
                    ],
                  }),
            },
          },
        }
      : {}),
    ...(speed === undefined
      ? {}
      : {
          movement: {
            speeds: [
              {
                movement_type: "land",
                label: "Land Speed",
                base_value_feet: speedBase ?? speed,
                adjusted_value_feet: speed,
                ...(speedAdjustment === undefined
                  ? {}
                  : {
                      adjustments: [
                        {
                          provenance: runtimeProvenance,
                          label: "Speed penalty",
                          value: speedAdjustment,
                          reason: "Encumbered reduces speed.",
                        },
                      ],
                    }),
                provenance: runtimeProvenance,
              },
            ],
          },
        }),
    ...(includeActivities
      ? {
          activities: [
            {
              activity_id: "claw",
              label: "Claw",
              kind: "strike" as const,
              usage: "unlimited" as const,
              rolls: [
                {
                  roll_id: "attack",
                  label: "Attack",
                  base_value: 12,
                  adjusted_value: 12,
                  surface: "attack_roll" as const,
                  provenance: runtimeProvenance,
                },
              ],
              damage: [
                {
                  damage_id: "main",
                  formula: "1d6+2",
                  damage_type: "slashing",
                  effect_kind: "damage" as const,
                  provenance: runtimeProvenance,
                },
              ],
              provenance: runtimeProvenance,
            },
          ],
        }
      : { activities: [] }),
    ...(kind === "creature"
      ? {
          spellcasting: [
            {
              entry_id: "innate",
              authored_order: 0,
              label: "Innate Spells",
              spells: [
                {
                  occurrence_id: "linked-spell",
                  authored_order: 0,
                  label: "Linked Rule",
                  target_record_key: "rules:linked",
                  rank: 1,
                  cast: {
                    spend_target: { target_type: "at_will" as const },
                    available: true,
                    state: { state_type: "at_will" as const },
                  },
                  provenance: runtimeProvenance,
                },
              ],
            },
          ],
        }
      : {}),
  };
  return creatureSurfaceFixture({
    encounter,
    level: levelLabel === undefined ? 1 : Number(levelLabel),
    profile: "encounter_participant",
    recordKey,
    title,
    traits,
  });
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  const linked = recordKey === "rules:linked";
  const condition = recordKey.startsWith("conditionitems:");
  const title = condition
    ? "Frightened Condition"
    : linked
      ? "Linked Rule"
      : "Goblin Warrior";
  return typedRecordDetailFixture({
    recordKey,
    title,
    ...(linked || condition
      ? {}
      : { referenceLabel: "Linked Rule", referenceRecordKey: "rules:linked" }),
  });
}

const runtimeProvenance = {
  source: { source_type: "canonical_record" as const },
};

function numberFact(label: string, value: number) {
  return {
    label,
    base_value: value,
    adjusted_value: value,
    provenance: runtimeProvenance,
  };
}

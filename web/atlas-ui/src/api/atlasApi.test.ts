import { expectTypeOf } from "vitest";
import type { AtlasRequestInit } from "./atlasApi";
import {
  applyEncounterHpAction,
  getEncounterTurnReview,
  advanceEncounterTurn,
  getReadiness,
  getRecordDetail,
  getRelatedRecords,
  getTraitDetail,
  getDefinitionDetail,
  getTraitRecords,
  searchPage,
  getRelationshipOccurrences,
  discoverFilterValues,
  discoverFilterEditor,
  discoverFilterCounts,
  createSavedList,
  deleteSavedList,
  removeSavedListItem,
  filterSavedList,
  getSavedList,
  getSavedLists,
  AtlasApiError,
  getEncounter,
  getEncounterConditionLifecycleActions,
  updateEncounterParticipant,
  addEncounterManualParticipant,
  updateEncounter,
  addEncounterRecordParticipant,
  reorderEncounterParticipant,
  removeEncounterParticipant,
  setEncounterTurn,
  addEncounterParticipantCondition,
  updateEncounterParticipantCondition,
  removeEncounterParticipantCondition,
} from "./atlasApi";
import type {
  EncounterMutationView,
  RelationshipOccurrencesPage,
  UpdateEncounterParticipantEdit,
} from "../generated/atlas";
import { detailFixture, summaryFixture } from "../test/fixtures";
import {
  API_CONTRACT,
  API_CONTRACT_HEADER,
  clearCompatibilityFailure,
} from "./compatibility";
const fetchMock = vi.fn();
beforeEach(() => {
  clearCompatibilityFailure();
  vi.stubGlobal("fetch", fetchMock);
  fetchMock.mockReset();
});
afterEach(() => vi.unstubAllGlobals());
function respond(value: unknown, status = 200) {
  fetchMock.mockImplementation(() =>
    Promise.resolve(
      new Response(JSON.stringify(value), {
        status,
        headers: {
          "Content-Type": "application/json",
          [API_CONTRACT_HEADER]: API_CONTRACT,
        },
      }),
    ),
  );
}
it("sends the list-item DELETE body and never replays an incompatible mutation", async () => {
  fetchMock.mockResolvedValue(
    new Response("invalid body", {
      status: 409,
      headers: { [API_CONTRACT_HEADER]: "old-contract" },
    }),
  );
  const request = { record_ref: "actions:seek" };
  await expect(removeSavedListItem("list with/slash", request)).rejects.toMatchObject({
    backendContract: "old-contract",
  });
  await expect(removeSavedListItem("list with/slash", request)).rejects.toMatchObject({
    backendContract: "old-contract",
  });
  expect(fetchMock).toHaveBeenCalledTimes(1);
  const [path, options] = fetchMock.mock.calls[0];
  expect(path).toBe("/api/lists/list%20with%2Fslash/items");
  expect(options.method).toBe("DELETE");
  expect(JSON.parse(options.body)).toEqual(request);
  expect(options.headers.get(API_CONTRACT_HEADER)).toBe(API_CONTRACT);
});

describe("source-backed API transport", () => {
  it("normalizes original boundary sources and grouped recovery identities without requiring one condition ID", async () => {
    respond({
      encounter_key: "ambush",
      expected_revision: 13,
      reminders: [],

      next_turn_unavailable_reason: null,
      incoming: null,
      outgoing: {
        event: "end",
        source_revision: 6,
        boundary_key: "original",
        participant_key: "a",
        participant_name: "Goblin",
        round_number: 2,
        participant_removed: false,
        damage_applied: false,
        recovered_episode_ids: [],
        reminders: [
          {
            kind: "condition_recovery",
            participant_key: "a",
            participant_name: "Goblin",
            condition_ids: [7, 8],
            condition_name: "Frightened",
            condition_value: 1,
          },
        ],
        persistent_damage_groups: [
          {
            episode_id: "fire",
            damage_type: "fire",
            selected_source_id: 9,
            sources: [
              {
                condition_id: 9,
                formula: "1d6",
                multiplier: "normal",
                recovery_dc: 15,
              },
            ],
          },
        ],
      },
    });
    const review = await getEncounterTurnReview("ambush");
    expect(review.outgoing!).toMatchObject({
      event: "end",
      source_revision: 6,
      round_number: 2,
      reminders: [{ condition_ids: [7, 8], condition_value: 1 }],
      persistent_damage_groups: [
        { selected_source_id: 9, sources: [{ condition_id: 9, recovery_dc: 15 }] },
      ],
    });
  });
  it("reads turn reviews without mutation and sends the exact advance revision", async () => {
    respond({
      encounter_key: "ambush",
      request_id: "advance",
      expected_revision: 12,
      current_participant_key: "a",
      next_participant_key: "b",
      reminders: [
        {
          kind: "duration_reminder",
          participant_key: "a",
          participant_name: "Goblin",
          condition_id: 7,
          condition_name: "Frightened",
          duration: { amount: 0, unit: "minutes", timing: null },
        },
      ],
    });
    const review = await getEncounterTurnReview("ambush");
    expect(review.expected_revision).toBe(12);
    expect(review.reminders[0]).toMatchObject({
      condition_id: 7,
      duration: { amount: 0, unit: "minutes", timing: null },
    });
    expect(fetchMock.mock.calls[0][0]).toBe("/api/encounters/ambush/turn-review");
    expect(fetchMock.mock.calls[0][1].method).toBeUndefined();
    respond({ encounter_key: "ambush" });
    await advanceEncounterTurn("ambush", {
      request_id: "advance",
      expected_revision: review.expected_revision,
    });
    expect(fetchMock.mock.calls[1][0]).toBe("/api/encounters/ambush/advance");
    expect(fetchMock.mock.calls[1][1].method).toBe("POST");
    expect(JSON.parse(fetchMock.mock.calls[1][1].body)).toEqual({
      request_id: "advance",
      expected_revision: 12,
    });
  });

  it("normalizes tagged vocabulary totals without fabricating option counts", async () => {
    const controller = new AbortController();
    respond({
      kind: "vocabulary",
      scope: { kind: "artifact" },
      values: {
        field: "traits",
        options: [{ value: "fire" }],
        selected_values: ["absent"],
        total_values: 1,
        basis: "product_artifact",
      },
    });
    const result = await discoverFilterValues(
      {
        kind: "vocabulary",
        scope: { kind: "artifact" },
        field_id: "traits",
        text: null,
        offset: 0,
        limit: 40,
        selected_values: ["absent"],
      },
      controller.signal,
    );
    expect(fetchMock.mock.calls[0][1].signal).toBe(controller.signal);
    expect(result).toMatchObject({
      kind: "vocabulary",
      values: {
        total_values: 1,
        options: [{ value: "fire" }],
        selected_values: ["absent"],
      },
    });
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).not.toHaveProperty("context");
  });
  it("preserves unavailable count outcomes and actual HTTP failures", async () => {
    const controller = new AbortController();
    const request = {
      context: {
        kind: "filtered" as const,
        filter: null,
        text: null,
        mode: "hybrid" as const,
      },
      field_id: "traits",
      clause_id: "traits",
      values: ["fire"],
    };
    respond({ counts: { status: "unavailable", reason: "negative_condition" } });
    expect(await discoverFilterCounts(request, controller.signal)).toEqual({
      counts: { status: "unavailable", reason: "negative_condition" },
    });
    expect(fetchMock.mock.calls[0][1].signal).toBe(controller.signal);
    respond({ code: "query_error", message: "Invalid target" }, 400);
    await expect(discoverFilterCounts(request)).rejects.toThrow("Invalid target");
  });
  it("decodes per-option counts including known zero without field-state statistics", async () => {
    respond({
      search_profile: "bge-base",
      artifact_generation: "generation-a",
      counts: {
        status: "available",
        value: {
          field: "traits",
          options: [{ value: "fire", distinct_roots: 0, selected: false }],
          total_values: 1,
          exhaustive: true,
          count_basis: "exact counterfactual displayed roots",
        },
      },
    });
    const result = await discoverFilterCounts({
      context: { kind: "filtered", filter: null, text: null },
      field_id: "traits",
      clause_id: null,
      values: ["fire"],
    });
    expect(result).toMatchObject({
      search_profile: "bge-base",
      artifact_generation: "generation-a",
      counts: { status: "available", value: { total_values: 1 } },
    });
    if (result.counts.status !== "available") throw new Error("Expected option counts");
    expect(result.counts.value.options).toEqual([
      { value: "fire", distinct_roots: 0, selected: false },
    ]);
  });
  const participantJson = {
    revision: 7,
    participant_key: "ghoul",
    position: 1,
    initiative_order: 2,
    temporary_hp: 0,
    current_hp: 20,
    max_hp: 30,
    initiative: -1,
    conditions: [
      {
        condition_id: 9,
        value: 1,
        duration: {
          amount: 4,
          unit: "rounds",
          timing: { participant_key: "ghoul", event: "end" },
        },
      },
    ],
    display_name: "Ghoul",
    side: "enemy",
    participant_variant: "normal",
    defeated: false,
    hazard_disabled: false,
    hidden: false,
    hp_origin: "derived",
    variant_origin: "default_unadjusted",
    participant_kind: "creature",
    status: "active",
    note_hint: null,
  };
  it("normalizes persistent source identity and recovery DCs together", async () => {
    respond({
      revision: 4,
      encounter: { round_duration_seconds: 6, round_number: 1 },
      participants: [
        {
          ...participantJson,
          conditions: [
            {
              ...participantJson.conditions[0],
              parameters: {
                kind: "persistent_damage",
                formula: "1d6",
                damage_type: "fire",
                multiplier: "normal",
                recovery: { kind: "specified", dc: 12 },
                episode_id: "fire-episode",
              },
            },
          ],
          persistent_damage_groups: [
            {
              episode_id: "fire-episode",
              damage_type: "fire",
              selected_source_id: 9,
              sources: [
                {
                  condition_id: 9,
                  formula: "1d6",
                  multiplier: "normal",
                  recovery_dc: 12,
                },
              ],
            },
          ],
        },
      ],
    });
    const result = await getEncounter("ambush");
    const participant = result.participants[0];
    expect(participant.conditions[0].condition_id).toBe(9);
    expect(participant.conditions[0].parameters).toMatchObject({
      recovery: { kind: "specified", dc: 12 },
    });
    expect(participant.persistent_damage_groups[0]).toMatchObject({
      selected_source_id: 9,
      sources: [{ condition_id: 9, recovery_dc: 12 }],
    });
  });
  it("decodes guidance on saved and inherent condition rows", async () => {
    respond({
      revision: 8,
      encounter: { round_duration_seconds: 6, round_number: 1 },
      participants: [
        {
          ...participantJson,
          participant_kind: "hazard",
          conditions: [
            { condition_id: 9, name: "Prone", guidance: ["Move by Crawling."] },
          ],
          inherent_conditions: {
            entries: [{ name: "Fleeing", guidance: ["Can't Delay or Ready."] }],
            unavailable_reason: null,
          },
        },
      ],
    });
    const {
      participants: [participant],
    } = await getEncounter("test");
    expect(participant.conditions[0]).toMatchObject({
      condition_id: 9,
      guidance: ["Move by Crawling."],
    });
    expect(participant.inherent_conditions?.entries[0].guidance).toEqual([
      "Can't Delay or Ready.",
    ]);
  });

  it("decodes tracker and non-round condition progress from the real router fixture", async () => {
    const fixture = await import("./__fixtures__/wire-conversion.json");
    respond(fixture.encounter);
    const detail = await getEncounter("ambush");
    const manual = detail.participants.find(
      (p) => p.participant_key === "fixture-manual",
    )!;
    const tracker = manual.manual_trackers.find((t) => t.name === "Breath ready")!;
    expect(tracker).toMatchObject({
      name: "Breath ready",
      remaining_rounds: 10,
      duration: { amount: 1, unit: "minutes" },
    });
    expect(tracker.duration).not.toHaveProperty("remaining_rounds");
    expect(manual.conditions[0].duration).toMatchObject({
      amount: 1,
      unit: "minutes",
      remaining_rounds: 10,
    });
  });
  it("decodes tracker completion and removal from the real router fixture", async () => {
    const fixture: Record<string, unknown> =
      await import("./__fixtures__/wire-conversion.json");
    respond(fixture.turn_review);
    const completed = await getEncounterTurnReview("ambush");
    expect(completed.incoming?.reminders[0]).toEqual({
      kind: "tracker_elapsed",
      participant_key: "fixture-manual",
      participant_name: "Kyra",
      tracker_key: "tracker_wire-completion",
      tracker_name: "Countdown",
    });
    expect(typeof completed.expected_revision).toBe("number");
    expect(completed.outgoing).toBeNull();
    expect(completed.incoming?.event).toBe("start");
    expect(typeof completed.incoming?.source_revision).toBe("number");
    respond(fixture.turn_review_removed);
    const removed = await getEncounterTurnReview("ambush");
    expect(removed.incoming?.reminders[0]).toMatchObject({
      kind: "tracker_elapsed",
      result: "Removed",
    });
  });
  it("normalizes actual HTTP encounter integers for display and revision-bound edits", async () => {
    respond({
      revision: 8,
      encounter: { round_duration_seconds: 6, round_number: 1 },
      participants: [participantJson],
    });
    const detail = await getEncounter("test");
    const participant = detail.participants[0];
    expect(detail.encounter.round_number).toBe(1);
    expect(participant).toMatchObject({
      revision: 7,
      position: 1,
      initiative_order: 2,
      initiative: -1,
      current_hp: 20,
      max_hp: 30,
      temporary_hp: 0,
      conditions: [
        {
          condition_id: 9,
          value: 1,
          duration: {
            amount: 4,
            unit: "rounds",
            timing: { participant_key: "ghoul", event: "end" },
          },
        },
      ],
    });
    respond({ encounter_key: "encounter-test" });
    expect(
      await updateEncounterParticipant("test", participant.participant_key, {
        expected_revision: participant.revision,
        initiative: null,
      }),
    ).toEqual({ encounter_key: "encounter-test" });
    expect(JSON.parse(fetchMock.mock.calls[1][1].body)).toMatchObject({
      expected_revision: 7,
      initiative: null,
    });
  });
  it.each([undefined, null, -1, 0.5, "7", "bad", Number.MAX_SAFE_INTEGER + 1])(
    "rejects missing or malformed participant revision %s",
    async (revision) => {
      respond({
        revision: 8,
        encounter: { round_duration_seconds: 6, round_number: 1 },
        participants: [{ ...participantJson, revision }],
      });
      await expect(getEncounter("test")).rejects.toBeInstanceOf(AtlasApiError);
    },
  );
  it("normalizes the largest safe revision without rounding", async () => {
    respond({
      revision: 8,
      encounter: { round_duration_seconds: 6, round_number: 1 },
      participants: [{ ...participantJson, revision: Number.MAX_SAFE_INTEGER }],
    });
    expect((await getEncounter("test")).participants[0].revision).toBe(
      9007199254740991,
    );
  });
  const participantRequest: UpdateEncounterParticipantEdit = {
    expected_revision: 7,
    display_name: "Ghoul",
  };
  const encounterMutations: Array<{
    name: string;
    path: string;
    method: string;
    call: () => Promise<EncounterMutationView>;
  }> = [
    {
      name: "metadata",
      path: "/api/encounters/test",
      method: "PATCH",
      call: () =>
        updateEncounter("test", {
          name: { expected: "Old name", value: "Test" },
        }),
    },
    {
      name: "record addition",
      path: "/api/encounters/test/participants/record",
      method: "POST",
      call: () =>
        addEncounterRecordParticipant("test", {
          record_ref: "actors:ghoul",
          quantity: 2,
        }),
    },
    {
      name: "manual addition",
      path: "/api/encounters/test/participants/manual",
      method: "POST",
      call: () => addEncounterManualParticipant("test", { display_name: "PC" }),
    },
    {
      name: "participant edit",
      path: "/api/encounters/test/participants/ghoul",
      method: "PATCH",
      call: () => updateEncounterParticipant("test", "ghoul", participantRequest),
    },
    {
      name: "reorder",
      path: "/api/encounters/test/participants/reorder",
      method: "POST",
      call: () =>
        reorderEncounterParticipant("test", {
          participant_key: "ghoul",
          target_participant_key: "pc",
          placement: "before",
        }),
    },
    {
      name: "participant removal",
      path: "/api/encounters/test/participants/ghoul",
      method: "DELETE",
      call: () => removeEncounterParticipant("test", "ghoul"),
    },
    {
      name: "turn",
      path: "/api/encounters/test/turn",
      method: "POST",
      call: () => setEncounterTurn("test", { participant_key: "ghoul" }),
    },
    {
      name: "condition addition",
      path: "/api/encounters/test/participants/ghoul/conditions",
      method: "POST",
      call: () =>
        addEncounterParticipantCondition("test", "ghoul", {
          expected_encounter_revision: 8,
          name: "Condition",
          request_id: "condition-test",
          parameters: { kind: "none" },
          event_intent: "apply",
          decisions: { keep_upright: false },
          immunity_override: false,
          scope: { kind: "everyone" },
          extra_action: { kind: "unspecified" },
        }),
    },
    {
      name: "condition edit",
      path: "/api/encounters/test/participants/ghoul/conditions/9",
      method: "PATCH",
      call: () =>
        updateEncounterParticipantCondition("test", "ghoul", 9, {
          expected_encounter_revision: 8,
          name: "Condition",
          request_id: "condition-test",
          parameters: { kind: "none" },
          event_intent: "apply",
          decisions: { keep_upright: false },
          immunity_override: false,
          scope: { kind: "everyone" },
          extra_action: { kind: "unspecified" },
        }),
    },
    {
      name: "condition removal",
      path: "/api/encounters/test/participants/ghoul/conditions/9",
      method: "DELETE",
      call: () =>
        removeEncounterParticipantCondition("test", "ghoul", 9, {
          request_id: "condition-remove",
          expected_encounter_revision: 8,
          intent: "remove_effect",
        }),
    },
  ];
  it.each(encounterMutations)(
    "accepts the $name durable receipt without hydration",
    async ({ path, method, call }) => {
      respond({ encounter_key: "encounter-test" });
      expect(await call()).toEqual({ encounter_key: "encounter-test" });
      expect(fetchMock).toHaveBeenCalledOnce();
      expect(fetchMock).toHaveBeenCalledWith(path, expect.objectContaining({ method }));
    },
  );
  it("sends both clock values as JSON numbers and preserves a nullable metadata clear", async () => {
    respond({ encounter_key: "encounter-test" });
    await updateEncounter("test", {
      round_duration_seconds: { expected: 6, value: 12 },
      note: { expected: "Old note", value: null },
    });
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual({
      round_duration_seconds: { expected: 6, value: 12 },
      note: { expected: "Old note", value: null },
    });
  });

  it("sends HP intent and formula text unchanged with the participant revision", async () => {
    respond({ encounter_key: "encounter-test" });
    await expect(
      applyEncounterHpAction("test", "ghoul", {
        request_id: "hp-action-test",
        expected_revision: 7,
        action: { kind: "damage", amount: " 10 - 3 " },
      }),
    ).resolves.toEqual({ encounter_key: "encounter-test" });
    expect(fetchMock).toHaveBeenCalledOnce();
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/encounters/test/participants/ghoul/hp",
      expect.objectContaining({ method: "POST" }),
    );
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual({
      request_id: "hp-action-test",
      expected_revision: 7,
      action: { kind: "damage", amount: " 10 - 3 " },
    });
  });
  it("preserves typed stale-participant conflicts without retrying", async () => {
    const conflict = {
      code: "encounter_participant_conflict",
      message: "Participant changed",
      retryable: false,
      details: { participant_key: "ghoul", expected_revision: 7, actual_revision: 8 },
    };
    respond(conflict, 409);
    await expect(
      updateEncounterParticipant("test", "ghoul", participantRequest),
    ).rejects.toMatchObject({
      status: 409,
      appError: conflict,
    });
    expect(fetchMock).toHaveBeenCalledOnce();
  });
  it("rejects an unsafe condition ID before constructing the request path", async () => {
    await expect(
      removeEncounterParticipantCondition(
        "test",
        "ghoul",
        Number.MAX_SAFE_INTEGER + 1,
        {
          request_id: "remove",
          expected_encounter_revision: 8,
          intent: "remove_effect",
        },
      ),
    ).rejects.toThrow("safe integer range");
    expect(fetchMock).not.toHaveBeenCalled();
  });
  it("rejects unsafe expected revision before issuing a write", async () => {
    await expect(
      updateEncounterParticipant("test", "ghoul", {
        ...participantRequest,
        expected_revision: 9007199254740992,
      }),
    ).rejects.toThrow("safe numeric range");
    expect(fetchMock).not.toHaveBeenCalled();
  });
  it("preserves unavailable HP and rejects already-rounded integer responses", async () => {
    const unknown = { ...participantJson, current_hp: undefined, max_hp: undefined };
    respond({
      revision: 8,
      encounter: { round_duration_seconds: 6, round_number: 1 },
      participants: [unknown],
    });
    const participant = (await getEncounter("test")).participants[0];
    expect(participant.current_hp).toBeUndefined();
    for (const invalid of [
      Number.MAX_SAFE_INTEGER + 1,
      Number.MIN_SAFE_INTEGER - 1,
      0.5,
    ]) {
      respond({
        revision: 8,
        encounter: { round_duration_seconds: 6, round_number: 1 },
        participants: [{ ...participantJson, current_hp: invalid }],
      });
      await expect(getEncounter("test")).rejects.toThrow("Invalid numeric field");
    }
  });
  it("rejects unsafe integer request values with either sign before serialization", async () => {
    respond({});
    for (const max_hp of [9007199254740992, -9007199254740992]) {
      await expect(
        addEncounterManualParticipant("test", {
          display_name: "Manual",
          max_hp,
        }),
      ).rejects.toThrow("safe numeric range");
    }
    expect(fetchMock).not.toHaveBeenCalled();
  });
  it("rejects unsafe Number literals and integer response counters instead of rounding", async () => {
    for (const value of [Number.MAX_SAFE_INTEGER + 1, Number.MIN_SAFE_INTEGER - 1]) {
      await expect(
        searchPage({
          expected_artifact_generation: null,
          criteria: {
            kind: "list_records",
            filter: { kind: "compare", field: "level", op: "eq", value },
          },
          page: { number: 1, size: 25 },
        }),
      ).rejects.toThrow("safe numeric range");
      respond({
        artifact_generation: "generation-a",
        page: { total: value },
        rows: [],
      });
      await expect(
        searchPage({
          expected_artifact_generation: null,
          criteria: { kind: "list_records", filter: null },
          page: { number: 1, size: 25 },
        }),
      ).rejects.toThrow("Invalid numeric field");
    }
  });
  it.each([NaN, Infinity, -Infinity])(
    "rejects non-finite nested request numbers before JSON serialization (%s)",
    async (value) => {
      await expect(
        searchPage({
          criteria: {
            kind: "list_records",
            filter: { kind: "compare", field: "level", op: "eq", value },
          },
          page: { number: 1, size: 25 },
        }),
      ).rejects.toThrow("safe numeric range");
      expect(fetchMock).not.toHaveBeenCalled();
    },
  );
  it("posts exact embedded/passage selection without per-marker requests", async () => {
    const controller = new AbortController();
    const detail = detailFixture("actors:ghoul", "Ghoul");
    respond(detail);
    const { field: _field, ...address } = detail.selected;
    const request = { ...address, fields: ["system.description.value"] };
    await getRecordDetail(request.record_key, request, controller.signal);
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(fetchMock.mock.calls[0][0]).toContain("/api/records/detail");
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual(request);
    expect(fetchMock.mock.calls[0][1].signal).toBe(controller.signal);
  });
  it("forwards cancellation to root details, related reads and filter catalog requests", async () => {
    const controller = new AbortController();
    const detail = detailFixture("actors:ghoul", "Ghoul");
    respond(detail);
    await getRecordDetail("actors:ghoul", undefined, controller.signal);
    respond({
      selection: detail.selected,
      artifact_generation: detail.artifact_generation,
      seed: detail.record,
      show_versions: false,
      references: { kind: "empty" },
      remaster_relationships: { kind: "empty" },
      variants: { kind: "empty" },
    });
    await getRelatedRecords(
      { selection: detail.selected, artifact_generation: detail.artifact_generation },
      controller.signal,
    );
    respond({ fields: [], scopes: [] });
    await discoverFilterEditor(
      {
        selected_field_ids: [],
      },
      controller.signal,
    );
    expect(fetchMock.mock.calls.map((call) => call[1].signal)).toEqual([
      controller.signal,
      controller.signal,
      controller.signal,
    ]);
    controller.abort();
    expect(fetchMock.mock.calls.every((call) => call[1].signal.aborted)).toBe(true);
  });
  it("preserves finite semantic real numbers and rejects unsafe authored/effective values", async () => {
    const detail = detailFixture("actors:ghoul", "Ghoul");
    const real = {
      ...detail,
      presentation: {
        ...detail.presentation,
        body: {
          kind: "physical_reference",
          value: {
            usage: { state: "value", value: "held" },
            bulk: { state: "value", value: 0.1, adjustment: null },
            price: { state: "value", value: "1 gp" },
            price_per: { state: "value", value: 1, adjustment: null },
          },
        },
      },
    };
    respond(real);
    expect(await getRecordDetail("actors:ghoul")).toEqual(real);
    respond({
      ...detail,
      presentation: {
        ...detail.presentation,
        body: {
          kind: "creature",
          value: {
            armor_class: {
              state: "value",
              value: Number.MAX_SAFE_INTEGER + 1,
              adjustment: null,
            },
          },
        },
      },
    });
    await expect(getRecordDetail("actors:ghoul")).rejects.toThrow("safe numeric range");
    respond({
      ...detail,
      presentation: {
        ...detail.presentation,
        body: {
          kind: "creature",
          value: {
            armor_class: {
              state: "value",
              value: 30,
              adjustment: {
                authored: Number.MAX_SAFE_INTEGER + 1,
                applied: [],
                suppressed: [],
                notes: [],
              },
            },
          },
        },
      },
    });
    await expect(getRecordDetail("actors:ghoul")).rejects.toThrow("safe numeric range");
  });
  it("sends typed predicates and distinguishes retrieval mode", async () => {
    const controller = new AbortController();
    respond({
      search_profile: "embeddinggemma",
      artifact_generation: "generation-a",
      mode: { kind: "text_search", query: "ghoul", mode: "lexical" },
      page: {
        number: 1,
        size: 25,
        count: 0,
        total: 0,
        has_more: false,
        next_page: null,
      },
      rows: [],
      coverage: null,
    });
    const request = {
      search_profile: "embeddinggemma",
      expected_artifact_generation: "generation-a",
      criteria: {
        kind: "text_search" as const,
        query: "ghoul",
        mode: "lexical" as const,
        filter: {
          kind: "compare" as const,
          field: "level",
          op: "gte" as const,
          value: 0,
        },
      },
      page: { number: 1, size: 25 },
    };
    const result = await searchPage(request, controller.signal);
    expect(result.search_profile).toBe("embeddinggemma");
    expect(result.artifact_generation).toBe("generation-a");
    expect(result.page.total).toBe(0);
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual(request);
    expect(fetchMock.mock.calls[0][1].signal).toBe(controller.signal);
    await searchPage({ ...request, page: { number: 2, size: 25 } });
    expect(fetchMock.mock.calls[1][0]).toContain("/api/search/page");
    expect(JSON.parse(fetchMock.mock.calls[1][1].body)).toEqual({
      ...request,
      page: { number: 2, size: 25 },
    });
  });
  it("normalizes only actual integer discovery counts", async () => {
    respond({
      kind: "contextual",
      search_profile: "embeddinggemma",
      artifact_generation: "generation-a",
      outcome: {
        status: "available",
        value: {
          field: "common.traits",
          options: [{ value: "fire", distinct_roots: 4, selected: false }],
          total_values: 1,
          exhaustive: true,
          count_basis: "distinct roots",
        },
      },
    });
    const result = await discoverFilterValues({
      kind: "contextual",
      only_matching: false,
      context: {
        kind: "filtered",
        search_profile: "embeddinggemma",
        expected_artifact_generation: "generation-a",
        filter: null,
        text: null,
      },
      field_id: "common.traits",
      clause_id: null,
      text: null,
      offset: 0,
      limit: 100,
    });
    expect(result.kind).toBe("contextual");
    expect(result).toMatchObject({
      search_profile: "embeddinggemma",
      artifact_generation: "generation-a",
    });
    if (result.kind !== "contextual" || result.outcome.status !== "available")
      throw new Error("Wrong discovery outcome");
    expect(result.outcome.value.options[0].distinct_roots).toBe(4);
    expect(result.outcome.value.total_values).toBe(1);
    respond({
      catalog_version: 4,
      limits: { source_bytes: 16384, nodes: 256, depth: 32, literal_list: 128 },
      groups: [],
    });
    expect(
      await discoverFilterEditor({
        selected_field_ids: [],
      }),
    ).toMatchObject({ catalog_version: 4 });
  });
  it("passes cancellation through a saved-list filter without treating it as a write", async () => {
    const controller = new AbortController();
    const aborted = new DOMException("Canceled", "AbortError");
    fetchMock.mockImplementationOnce(async (_path, init) => {
      expect(init.signal).toBe(controller.signal);
      controller.abort();
      throw aborted;
    });
    await expect(filterSavedList("research", {}, controller.signal)).rejects.toBe(
      aborted,
    );
  });
  it("preserves useful saved-list CRUD requests", async () => {
    respond({ items: [] });
    await createSavedList({ slug: "research", name: "Research", tags: [] });
    await filterSavedList("Research/雪%", {
      filter: { kind: "state_match", field: "level", state: "missing" },
    });
    await deleteSavedList("Research/雪%");
    expect(
      fetchMock.mock.calls.map(([path, init]) => ({
        path,
        method: init.method,
        body: init.body ? JSON.parse(init.body) : null,
      })),
    ).toEqual([
      {
        path: "/api/lists",
        method: "POST",
        body: { slug: "research", name: "Research", tags: [] },
      },
      {
        path: "/api/lists/Research%2F%E9%9B%AA%25/filter",
        method: "POST",
        body: {
          filter: { kind: "state_match", field: "level", state: "missing" },
        },
      },
      {
        path: "/api/lists/Research%2F%E9%9B%AA%25",
        method: "DELETE",
        body: null,
      },
    ]);
  });
  it("keeps safe saved-list JSON counts and positions as numbers with response validation", async () => {
    respond({ lists: [{ item_count: 2 }] });
    expect((await getSavedLists()).lists[0].item_count).toBe(2);
    respond({ list: { item_count: 2 }, items: [{ position: 0 }, { position: 1 }] });
    const detail = await getSavedList("research");
    expect(detail.list.item_count).toBe(2);
    expect(detail.items.map((item) => item.position)).toEqual([0, 1]);
    respond({ items: [{ position: Number.MAX_SAFE_INTEGER + 1 }] });
    await expect(getSavedList("research")).rejects.toThrow("safe numeric range");
    respond([]);
    await expect(getSavedList("research")).rejects.toThrow(
      "Invalid saved-list response",
    );
  });
  it("preserves structured ambiguity/error information", async () => {
    respond(
      {
        code: "record_resolution_ambiguous",
        message: "Multiple records match",
        details: null,
      },
      400,
    );
    await expect(getReadiness()).rejects.toMatchObject({
      status: 400,
      appError: { code: "record_resolution_ambiguous" },
    });
  });
  it("surfaces invalid success JSON and response text", async () => {
    fetchMock.mockResolvedValue(
      new Response("broken", {
        status: 200,
        headers: { [API_CONTRACT_HEADER]: API_CONTRACT },
      }),
    );
    await expect(getReadiness()).rejects.toBeInstanceOf(AtlasApiError);
  });
});

it("normalizes saved HP decisions and sends guarded resolution without another HP event", async () => {
  const api = await import("./atlasApi");
  const context = {
    lethality: null,
    critical: null,
    enemy_source: null,
    responsible_participant_key: null,
    base_death_threshold: 5,
    keep_current_initiative: false,
    attack_or_spell_damage: null,
    natural_rest_healing: false,
    keep_upright: false,
  };
  const pending = {
    consequence_key: "pending",
    participant_key: "actor",
    before_hp: 9,
    after_hp: 0,
    initiative_only: false,
    required_facts: ["lethality"],
    context,
    knockout_options: [{ label: "Knockout", result: "Unconscious", context }],
  };
  respond([pending]);
  const [row] = await api.getEncounterHpLifecycle("test");
  expect(row.knockout_options[0].context.base_death_threshold).toBe(5);
  expect(row.before_hp).toBe(9);
  expect(row.context.base_death_threshold).toBe(5);
  const request = {
    request_id: "hp-resolution",
    expected_encounter_revision: 7,
    consequence_key: "pending",
    resolution: "apply" as const,
    context: { ...row.context, keep_current_initiative: true },
  };
  respond({
    ...pending,
    context: { ...context, keep_current_initiative: true },
    required_facts: [],
  });
  expect(
    (await api.previewEncounterHpLifecycle("test", request)).required_facts,
  ).toEqual([]);
  respond({ encounter_key: "test" });
  await api.resolveEncounterHpLifecycle("test", request);
  expect(fetchMock).toHaveBeenLastCalledWith(
    "/api/encounters/test/hp-lifecycle/resolve",
    expect.objectContaining({
      method: "POST",
      body: JSON.stringify({
        ...request,
        expected_encounter_revision: 7,
        context: { ...context, keep_current_initiative: true },
      }),
    }),
  );
});
it("reads persistent total and one shared Hardness deduction from the server", async () => {
  const api = await import("./atlasApi");
  respond({
    round_number: 2,
    encounter_revision: 7,
    damage_groups: [],
    damage_applied: true,
    episodes: [],
    damage_result: {
      encounter_key: "test",
      request_id: "batch",
      total: 4,
      hardness_absorbed: 5,
      groups: [
        {
          episode_id: "fire",
          damage_type: "fire",
          source_condition_id: 1,
          amount: 9,
          roll: null,
          table_resolved: false,
        },
      ],
    },
    recovery_results: [],
  });
  const review = await api.getEncounterPersistentReview("test", "actor");
  expect(review.round_number).toBe(2);
  expect(review.encounter_revision).toBe(7);
  expect(review.damage_result).toEqual(
    expect.objectContaining({
      total: 4,
      hardness_absorbed: 5,
      groups: [expect.objectContaining({ amount: 9, source_condition_id: 1 })],
    }),
  );
});

it("reads casting reminders without action cost or attempt contracts", async () => {
  const api = await import("./atlasApi");
  const selection = detailFixture("actors:a").selected;
  const advisories = [
    {
      source: "condition:1",
      label: "Stupefied",
      reason: "DC 6 flat check or the spell is disrupted",
    },
  ];
  respond({ advisories });
  expect(
    await api.encounterCastingReminders("ambush", {
      participant_key: "caster",
      selection,
    }),
  ).toEqual({ advisories });
  expect(fetchMock).toHaveBeenLastCalledWith(
    "/api/encounters/ambush/casting-reminders",
    expect.objectContaining({ method: "POST" }),
  );
  expect(
    JSON.parse(fetchMock.mock.calls[fetchMock.mock.calls.length - 1][1].body),
  ).toEqual({
    participant_key: "caster",
    selection,
  });
});

it("normalizes both reference checks while sending only the selected strike and target", async () => {
  const api = await import("./atlasApi");
  const strike = {
    participant_key: "actor",
    reference: {
      artifact_generation: "current",
      selection: {
        address: {
          record_key: "actors:a",
          embedded_path: [],
          source_fingerprint: null,
        },
        fields: [],
        passage: null,
      },
    },
  };
  respond({
    options: { expected_revision: 3, expected_participant_revision: 2 },
    action: {
      issues: [],
      confused: false,
      disruption: { dc: 5, sources: [{ condition_id: 7, name: "Grabbed" }] },
      targeting: {
        prevents_effect: false,
        requires_guessed_location: true,
        check_senses: false,
      },
    },
    targeting_check: { dc: 11, sources: [{ condition_id: 8, name: "Undetected" }] },
  });
  const result = await api.previewEncounterStrikeContext("ambush", {
    strike,
    target_key: "target",
  });
  expect(result.action.disruption).toEqual({
    dc: 5,
    sources: [{ condition_id: 7, name: "Grabbed" }],
  });
  expect(result.targeting_check).toEqual({
    dc: 11,
    sources: [{ condition_id: 8, name: "Undetected" }],
  });
  expect(result.options.expected_revision).toBe(3);
  expect(
    JSON.parse(fetchMock.mock.calls[fetchMock.mock.calls.length - 1][1].body),
  ).toEqual({
    strike,
    target_key: "target",
  });
});

it("rejects trait responses from another identity or artifact generation", async () => {
  respond({ trait_key: "trait:deadly-d12" });
  await expect(
    getTraitDetail({
      trait_key: "trait:deadly-d8",
      expected_artifact_generation: null,
    }),
  ).rejects.toThrow("does not match");
  const request = {
    trait_key: "trait:agile",
    expected_artifact_generation: "current",
    page: { number: 1, size: 20 },
  };
  respond({
    trait_key: "trait:agile",
    artifact_generation: "old",
  });
  await expect(getTraitRecords(request)).rejects.toMatchObject({
    appError: { code: "artifact_generation_changed" },
  });
});

it("posts exact relationship evidence pages and normalizes occurrence totals", async () => {
  const constraint = {
    direction: "incoming" as const,
    selection: {
      record_key: "items:armor",
      embedded_path: [],
      field: null,
      passage: null,
      source_fingerprint: "checked",
    },
    include_contents: true,
  };
  const response = {
    artifact_generation: "generation-a",
    clause: {
      clause_id: "link",
      relationship: constraint,

      record: summaryFixture("items:armor", "Leather Armor"),
      title: "Leather Armor",
    },
    result_record: summaryFixture("actors:goblin", "Goblin"),
    occurrences: [],
    page: { number: 2, size: 3, count: 0, total: 9, has_more: true, next_page: 3 },
  } satisfies RelationshipOccurrencesPage;
  respond(response);
  const controller = new AbortController();
  const request = {
    clause_id: "link",
    filter: {
      kind: "relationship" as const,
      clause_id: "link",
      relationship: constraint,
    },
    result_record_key: "actors:goblin",
    page: { number: 2, size: 3 },
    expected_artifact_generation: "generation-a",
  };
  const result = await getRelationshipOccurrences(request, controller.signal);
  expect(result.page.total).toBe(9);
  expect(fetchMock).toHaveBeenCalledWith(
    "/api/search/relationship-occurrences",
    expect.objectContaining({ method: "POST", signal: controller.signal }),
  );
  expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual(request);
});

it("normalizes recovery source associations for both check and non-check offers", async () => {
  respond({
    encounter_key: "encounter",
    encounter_revision: 4,
    participant_key: "actor",
    actions: [
      {
        kind: "reminder",
        condition_id: 26,
        text: "Recovery guidance",
        condition_ids: [26],
        label: "Treat Wounds",
        available: true,
        unavailable_reason: null,
      },
      {
        kind: "dying_recovery",
        condition_ids: [27],
        label: "Wake",
        available: true,
        unavailable_reason: null,
        check: { dc: 10, modifier: 3 },
      },
    ],
  });
  const result = await getEncounterConditionLifecycleActions("encounter", "actor");
  expect(result.actions[0].condition_ids).toEqual([26]);
  expect(result.actions[1]).toMatchObject({
    condition_ids: [27],
    check: { dc: 10, modifier: 3 },
  });
});

it("normalizes the executable persistent-damage groups returned by turn review", async () => {
  const api = await import("./atlasApi");
  respond({
    round_number: 2,
    encounter_revision: 2,
    damage_applied: false,
    episodes: [],
    damage_result: null,
    recovery_results: [],

    damage_groups: [
      {
        episode_id: "fire",
        damage_type: "fire",
        selected_source_id: 9,
        sources: [
          { condition_id: 9, formula: "2d6", multiplier: "normal", recovery_dc: 15 },
        ],
      },
    ],
  });
  const review = await api.getEncounterPersistentReview("encounter", "actor");
  expect(review.encounter_revision).toBe(2);
  expect(review.damage_groups[0].selected_source_id).toBe(9);
  expect(review.damage_groups[0].sources[0].condition_id).toBe(9);
  expect(review.damage_groups[0].sources[0].recovery_dc).toBe(15);
});

it("normalizes persistent Add/Edit choices without losing source identity", async () => {
  const api = await import("./atlasApi");
  respond({
    encounter_key: "test",
    encounter_revision: 2,
    consequences: [],
    hp_change: {
      current_before: 12,
      current_after: 11,
      maximum_before: null,
      maximum_after: null,
    },
    persistent_precedence: {
      damage_type: "fire",
      sources: [
        { source: { kind: "edited" }, formula: "2d6", multiplier: "normal" },
        {
          source: { kind: "existing", condition_id: 45 },
          formula: "1d12",
          multiplier: "normal",
        },
      ],
    },
  });
  const result = await api.previewEncounterConditionAdd("test", "actor", {
    request_id: "preview",
    expected_encounter_revision: 2,
    event_intent: "apply",
    decisions: { keep_upright: false },
    condition_ref: "conditionitems:lDVqvLKA6eF3Df60",
    scope: { kind: "everyone" },
    extra_action: { kind: "unspecified" },
    immunity_override: false,
    parameters: {
      kind: "persistent_damage",
      formula: "2d6",
      damage_type: "fire",
      multiplier: "normal",
      recovery: { kind: "standard" },
    },
  });
  expect(result.hp_change).toEqual({
    current_before: 12,
    current_after: 11,
    maximum_before: null,
    maximum_after: null,
  });
  expect(result.persistent_precedence?.sources[1].source).toEqual({
    kind: "existing",
    condition_id: 45,
  });
});

it("normalizes reviewed and recorded protection facts", async () => {
  const api = await import("./atlasApi");
  const protection = { immune: true, weakness: 2, resistance: 3 };
  respond({
    encounter_revision: 4,
    round_number: 1,
    damage_groups: [],
    episodes: [
      {
        episode_id: "bleed",
        damage_resolution: { kind: "known", protection },
        end_turn_recovery_available: true,
        end_turn_recovery_unavailable_reason: null,
        end_turn_roll_available: true,
        end_turn_roll_unavailable_reason: null,
      },
    ],
    damage_applied: true,
    damage_result: {
      encounter_key: "encounter",
      request_id: "r",
      total: 0,
      hardness_absorbed: 0,
      groups: [
        {
          episode_id: "bleed",
          damage_type: "bleed",
          source_condition_id: 4,
          amount: 0,
          roll: null,
          table_resolved: false,
          protection,
        },
      ],
    },
    recovery_results: [],
  });
  const review = await api.getEncounterPersistentReview("encounter", "actor");
  expect(review.episodes[0].damage_resolution).toEqual({
    kind: "known",
    protection: { immune: true, weakness: 2, resistance: 3 },
  });
  expect(review.damage_result?.groups[0].protection).toEqual({
    immune: true,
    weakness: 2,
    resistance: 3,
  });
});

it.each([null, undefined])(
  "rejects persistent groups without an active source (%s)",
  async (selected_source_id) => {
    const api = await import("./atlasApi");
    respond({
      round_number: 2,
      encounter_revision: 2,
      damage_applied: false,
      episodes: [],
      damage_result: null,
      recovery_results: [],

      damage_groups: [
        {
          episode_id: "fire",
          damage_type: "fire",
          selected_source_id,
          sources: [
            { condition_id: 9, formula: "2d6", multiplier: "normal", recovery_dc: 15 },
          ],
        },
      ],
    });
    await expect(
      api.getEncounterPersistentReview("encounter", "actor"),
    ).rejects.toBeInstanceOf(AtlasApiError);
  },
);

it("normalizes prepared damage and recovery without changing their recorded identities", async () => {
  const api = await import("./atlasApi");
  const context = {
    encounter_key: "encounter",
    participant_key: "actor",
    boundary_key: "boundary",
    round_number: 2,
    record_key: "npc:test",
    artifact_generation: "generation",
  };
  const roll = {
    formula: "1d6",
    total: 4,
    expression: {
      kind: "dice" as const,
      total: 4,
      dice: [{ sides: 6, value: 4, kept: true }],
    },
  };
  respond({
    damage_draft: null,
    recovery_draft: {
      context,
      source_condition_id: 9,
      dc: 15,
      dc_override: null,
      roll: null,
    },
    complete: false,
    damage: {
      encounter_key: "encounter",
      request_id: "prepared-damage",
      total: 4,
      hardness_absorbed: 0,
      groups: [
        {
          episode_id: "fire",
          damage_type: "fire",
          source_condition_id: 9,
          roll: null,
          amount: 4,
          table_resolved: false,
          protection: { immune: false, weakness: 2, resistance: 3 },
        },
      ],
    },
    recovery: {
      encounter_key: "encounter",
      request_id: "prepared-recovery",
      episode_id: "fire",
      damage_type: "fire",
      dc: 15,
      roll: null,
      success: false,
    },
  });
  const prepared = await api.prepareEncounterPersistentDamage("encounter", {
    request_id: "draft",
    expected_encounter_revision: 7,
    participant_key: "actor",
    boundary_key: "boundary",
    artifact_generation: "generation",
    groups: [
      {
        episode_id: "fire",
        damage: {
          kind: "prepared",
          draft: {
            context: { ...context, round_number: 2 },
            source_condition_id: 9,
            roll,
          },
        },
      },
    ],
    recovery: null,
  });
  expect(prepared.damage.groups[0]).toMatchObject({
    source_condition_id: 9,
    amount: 4,
    protection: { weakness: 2, resistance: 3 },
  });
  expect(prepared.damage_draft).toBeNull();
  expect(prepared.recovery_draft).toMatchObject({
    context: { round_number: 2 },
    source_condition_id: 9,
    dc: 15,
    dc_override: null,
  });
  expect(prepared.recovery).toMatchObject({
    request_id: "prepared-recovery",
    dc: 15,
    success: false,
  });
  expect(fetchMock).toHaveBeenLastCalledWith(
    "/api/encounters/encounter/persistent-preparation",
    expect.objectContaining({
      method: "POST",
      body: JSON.stringify({
        request_id: "draft",
        expected_encounter_revision: 7,
        participant_key: "actor",
        boundary_key: "boundary",
        artifact_generation: "generation",
        groups: [
          {
            episode_id: "fire",
            damage: {
              kind: "prepared",
              draft: { context, source_condition_id: 9, roll },
            },
          },
        ],
        recovery: null,
      }),
    }),
  );
});

it.each([0, 2, null])(
  "normalizes the Stunned preview remainder %s",
  async (remainder) => {
    const api = await import("./atlasApi");
    respond({
      encounter_key: "test",
      encounter_revision: 2,
      consequences: [
        {
          kind: "stunned_actions_lost",
          participant_key: "actor",
          before: 3,
          after: 1,
          stunned_after: remainder,
        },
      ],
    });
    const result = await api.previewEncounterConditionAdd("test", "actor", {
      request_id: "preview-stunned",
      expected_encounter_revision: 2,
      condition_ref: "conditionitems:dfCMdR4wnpbYNTix",
      value: 2,
      event_intent: "apply",
      decisions: { keep_upright: false },
      scope: { kind: "everyone" },
      extra_action: { kind: "unspecified" },
      immunity_override: false,
      parameters: { kind: "none" },
    });
    expect(result.consequences).toEqual([
      {
        kind: "stunned_actions_lost",
        participant_key: "actor",
        before: 3,
        after: 1,
        stunned_after: remainder === null ? null : Number(remainder),
      },
    ]);
  },
);

it("requires deliberate intent for non-GET transport requests", () => {
  expectTypeOf<{ method: "POST"; body: string }>().not.toExtend<AtlasRequestInit>();
  expectTypeOf<{ method: "PATCH" }>().not.toExtend<AtlasRequestInit>();
  expectTypeOf<{ method: "DELETE" }>().not.toExtend<AtlasRequestInit>();
  expectTypeOf<{ method: "POST"; readOnly: true }>().not.toExtend<AtlasRequestInit>();
  expectTypeOf<{ method: string; intent: "read" }>().not.toExtend<AtlasRequestInit>();
  expectTypeOf<{ method: "POST"; intent: "read" }>().toExtend<AtlasRequestInit>();
  expectTypeOf<{ method: "POST"; intent: "action" }>().toExtend<AtlasRequestInit>();
  expectTypeOf<{ signal: AbortSignal }>().toExtend<AtlasRequestInit>();
});

it("reads exact definition detail and preserves absent content and the record destination", async () => {
  const result = {
    artifact_generation: "generation-a",
    definition_key: "sense:darkvision",
    label: "Darkvision",
    help: {
      definition_key: "sense:darkvision",

      destination: {
        kind: "record",
        navigation: {
          record_key: "abilities:darkvision",
          embedded_path: [],
          field: null,
          passage: null,
          source_fingerprint: null,
        },
      },
    },
    content: null,

    notices: [],

    related_filters: [],
    publications: [],
  };
  respond(result);
  expect(
    await getDefinitionDetail({
      definition_key: "sense:darkvision",
      expected_artifact_generation: "generation-a",
    }),
  ).toEqual(result);
  expect(fetchMock.mock.calls[0][0]).toBe("/api/definitions/detail");
  expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual({
    definition_key: "sense:darkvision",
    expected_artifact_generation: "generation-a",
  });
  respond({ ...result, definition_key: "language:common" });
  await expect(
    getDefinitionDetail({
      definition_key: "sense:darkvision",
      expected_artifact_generation: null,
    }),
  ).rejects.toBeInstanceOf(AtlasApiError);
});

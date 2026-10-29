import {
  applyEncounterHpAction,
  getEncounterTurnReview,
  advanceEncounterTurn,
  getReadiness,
  getRecordDetail,
  getRelatedRecords,
  getTraitDetail,
  getTraitRecords,
  searchPage,
  getRelationshipOccurrences,
  discoverFilterValues,
  discoverFilterEditor,
  discoverFilterCounts,
  createSavedList,
  deleteSavedList,
  filterSavedList,
  getSavedList,
  getSavedLists,
  AtlasApiError,
  getEncounter,
  getEncounterConditionLifecycleActions,
  getEncounterReference,
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
  UpdateEncounterParticipantRequest,
} from "../generated/atlas";
import { participantUpdate } from "../features/encounters/participantEdits";
import { detailFixture } from "../test/fixtures";
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
describe("source-backed API transport", () => {
  it("normalizes original boundary sources and grouped recovery identities without requiring one condition ID", async () => {
    respond({
      encounter_key: "ambush",
      expected_revision: 13,
      reminders: [],

      next_turn_unavailable_reason: null,
      outgoing: {
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
      round_number: 2n,
      reminders: [{ condition_ids: [7n, 8n] }],
      persistent_damage_groups: [
        { selected_source_id: 9n, sources: [{ condition_id: 9n, recovery_dc: 15n }] },
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
    expect(review.expected_revision).toBe(12n);
    expect(review.reminders[0]).toMatchObject({
      condition_id: 7n,
      duration: { amount: 0n, unit: "minutes", timing: null },
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
        basis: "product_artifact_before_edition_preference",
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
        total_values: 1n,
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
      context: { kind: "filtered", filter: null, text: null, mode: "hybrid" },
      field_id: "traits",
      clause_id: null,
      values: ["fire"],
    });
    expect(result.counts).toMatchObject({
      status: "available",
      value: { options: [{ value: "fire", distinct_roots: 0n }], total_values: 1n },
    });
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
      encounter: { round_number: 1 },
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
    expect(participant.conditions[0].parameters).toMatchObject({
      recovery: { kind: "specified", dc: 12n },
    });
    expect(participant.persistent_damage_groups[0]).toMatchObject({
      selected_source_id: 9n,
      sources: [
        { condition_id: participant.conditions[0].condition_id, recovery_dc: 12n },
      ],
    });
  });
  it.each(["creature", "hazard"] as const)(
    "preserves exact condition diagnostic identity through %s encounter and reference transport",
    async (kind) => {
      const notes = [
        {
          source: "condition",
          label: "Enfeebled",
          reason: "Manual attack adjustment",
          condition_id: 9,
        },
        { source: "rule", label: "Other effect", reason: "Manual adjustment" },
      ];
      const actor = { runtime: { unapplied_effects: notes } };
      const presentation = {
        body: { kind, value: kind === "creature" ? actor : { actor } },
      };
      respond({
        revision: 8,
        encounter: { round_number: 1 },
        participants: [{ ...participantJson, presentation }],
      });
      const encounter = await getEncounter("test");
      const expected = [
        {
          ...notes[0],
          condition_id: encounter.participants[0].conditions[0].condition_id,
        },
        notes[1],
      ];
      const body = encounter.participants[0].presentation?.body;
      expect(
        body &&
          (body.kind === "creature"
            ? body.value.runtime
            : body.kind === "hazard"
              ? body.value.actor.runtime
              : null
          )?.unapplied_effects,
      ).toEqual(expected);
      respond({ presentation });
      const reference = await getEncounterReference("test", "ghoul", {
        artifact_generation: "generation-test",
        selection: {
          record_key: "actors:ghoul",
          embedded_path: [],
          fields: [],
          passage: null,
          source_fingerprint: null,
        },
      });
      const referenceBody = reference.presentation.body;
      expect(
        (referenceBody.kind === "creature"
          ? referenceBody.value.runtime
          : referenceBody.kind === "hazard"
            ? referenceBody.value.actor.runtime
            : null
        )?.unapplied_effects,
      ).toEqual(expected);
    },
  );
  it("normalizes actual HTTP encounter integers for display and revision-bound edits", async () => {
    respond({
      revision: 8,
      encounter: { round_number: 1 },
      participants: [participantJson],
    });
    const detail = await getEncounter("test");
    const participant = detail.participants[0];
    expect(detail.encounter.round_number).toBe(1n);
    expect(participant).toMatchObject({
      revision: 7n,
      position: 1n,
      initiative_order: 2n,
      initiative: -1n,
      current_hp: 20n,
      max_hp: 30n,
      temporary_hp: 0n,
      conditions: [
        {
          condition_id: 9n,
          value: 1n,
          duration: {
            amount: 4n,
            unit: "rounds",
            timing: { participant_key: "ghoul", event: "end" },
          },
        },
      ],
    });
    respond({ encounter_key: "encounter-test" });
    expect(
      await updateEncounterParticipant(
        "test",
        participantUpdate(participant, { current_hp: 19n }),
      ),
    ).toEqual({ encounter_key: "encounter-test" });
    expect(JSON.parse(fetchMock.mock.calls[1][1].body)).toMatchObject({
      participant_key: "ghoul",
      expected_revision: 7,
      current_hp: 19,
    });
  });
  it.each([undefined, null, -1, 0.5, "7", "bad", Number.MAX_SAFE_INTEGER + 1])(
    "rejects missing or malformed participant revision %s",
    async (revision) => {
      respond({
        revision: 8,
        encounter: { round_number: 1 },
        participants: [{ ...participantJson, revision }],
      });
      await expect(getEncounter("test")).rejects.toBeInstanceOf(AtlasApiError);
    },
  );
  it("normalizes the largest safe revision without rounding", async () => {
    respond({
      revision: 8,
      encounter: { round_number: 1 },
      participants: [{ ...participantJson, revision: Number.MAX_SAFE_INTEGER }],
    });
    expect((await getEncounter("test")).participants[0].revision).toBe(
      9007199254740991n,
    );
  });
  const participantRequest: UpdateEncounterParticipantRequest = {
    expected_revision: 7n,
    participant_key: "ghoul",
    display_name: "Ghoul",
    side: "enemy",
    participant_variant: "normal",
    temporary_hp: 0n,
    defeated: false,
    hazard_disabled: false,
    hidden: false,
    hp_edit: false,
    max_hp_edit: false,
    variant_edit: false,
    use_derived_max: false,
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
        updateEncounter({
          encounter_key: "test",
          slug: "test",
          name: "Test",
          status: "draft",
        }),
    },
    {
      name: "record addition",
      path: "/api/encounters/test/participants/record",
      method: "POST",
      call: () =>
        addEncounterRecordParticipant({
          encounter_ref: "test",
          record_ref: "actors:ghoul",
          quantity: 2,
        }),
    },
    {
      name: "manual addition",
      path: "/api/encounters/test/participants/manual",
      method: "POST",
      call: () =>
        addEncounterManualParticipant({ encounter_ref: "test", display_name: "PC" }),
    },
    {
      name: "participant edit",
      path: "/api/encounters/test/participants/ghoul",
      method: "PATCH",
      call: () => updateEncounterParticipant("test", participantRequest),
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
      call: () => setEncounterTurn({ encounter_ref: "test", participant_key: "ghoul" }),
    },
    {
      name: "condition addition",
      path: "/api/encounters/test/participants/ghoul/conditions",
      method: "POST",
      call: () =>
        addEncounterParticipantCondition("test", {
          participant_key: "ghoul",
          expected_encounter_revision: 8n,
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
        updateEncounterParticipantCondition("test", "ghoul", {
          expected_encounter_revision: 8n,
          condition_id: 9n,
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
        removeEncounterParticipantCondition("test", "ghoul", 9n, {
          request_id: "condition-remove",
          expected_encounter_revision: 8n,
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
  it("sends HP intent and formula text unchanged with the participant revision", async () => {
    respond({ encounter_key: "encounter-test" });
    await expect(
      applyEncounterHpAction("test", {
        request_id: "hp-action-test",
        participant_key: "ghoul",
        expected_revision: 7n,
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
      participant_key: "ghoul",
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
      updateEncounterParticipant("test", participantRequest),
    ).rejects.toMatchObject({
      status: 409,
      appError: conflict,
    });
    expect(fetchMock).toHaveBeenCalledOnce();
  });
  it("rejects unsafe expected revision before issuing a write", async () => {
    await expect(
      updateEncounterParticipant("test", {
        ...participantRequest,
        expected_revision: 9007199254740992n,
      }),
    ).rejects.toThrow("safe integer range");
    expect(fetchMock).not.toHaveBeenCalled();
  });
  it("preserves unavailable HP and rejects already-rounded integer responses", async () => {
    const unknown = { ...participantJson, current_hp: undefined, max_hp: undefined };
    respond({ revision: 8, encounter: { round_number: 1 }, participants: [unknown] });
    const participant = (await getEncounter("test")).participants[0];
    expect(participant.current_hp).toBeUndefined();
    for (const invalid of [
      Number.MAX_SAFE_INTEGER + 1,
      Number.MIN_SAFE_INTEGER - 1,
      0.5,
    ]) {
      respond({
        revision: 8,
        encounter: { round_number: 1 },
        participants: [{ ...participantJson, current_hp: invalid }],
      });
      await expect(getEncounter("test")).rejects.toThrow("Invalid numeric field");
    }
  });
  it("rejects unsafe integer request values with either sign before serialization", async () => {
    respond({});
    for (const max_hp of [9007199254740992n, -9007199254740992n]) {
      await expect(
        addEncounterManualParticipant({
          encounter_ref: "test",
          display_name: "Manual",
          max_hp,
        }),
      ).rejects.toThrow("safe integer range");
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
      references: { kind: "empty" },
      counterparts: { kind: "empty" },
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
    expect(result.page.total).toBe(0n);
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
        mode: "lexical",
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
    expect(result.outcome.value.options[0].distinct_roots).toBe(4n);
    expect(result.outcome.value.total_values).toBe(1n);
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
  it("preserves useful saved-list CRUD requests", async () => {
    respond({ items: [] });
    await createSavedList({ slug: "research", name: "Research", tags: [] });
    await filterSavedList({
      list_ref: "Research",
      filter: { kind: "state_match", field: "level", state: "missing" },
    });
    await deleteSavedList("Research");
    expect(fetchMock.mock.calls.map((c) => c[1].method)).toEqual([
      "POST",
      "POST",
      "DELETE",
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
  respond([
    {
      consequence_key: "pending",
      participant_key: "actor",
      before_hp: 9,
      after_hp: 0,
      initiative_only: false,
      required_facts: ["lethality"],
      context,
    },
  ]);
  const [row] = await api.getEncounterHpLifecycle("test");
  expect(row.before_hp).toBe(9n);
  expect(row.context.base_death_threshold).toBe(5n);
  const request = {
    request_id: "hp-resolution",
    expected_encounter_revision: 7n,
    consequence_key: "pending",
    resolution: "apply" as const,
    context: { ...row.context, keep_current_initiative: true },
  };
  respond({
    ...row,
    before_hp: 9,
    after_hp: 0,
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
    prepared_damage: [],
    prepared_recovery: [],
  });
  const review = await api.getEncounterPersistentReview("test", "actor");
  expect(review.round_number).toBe(2n);
  expect(review.damage_result).toEqual(
    expect.objectContaining({
      total: 4n,
      hardness_absorbed: 5n,
      groups: [expect.objectContaining({ amount: 9n, source_condition_id: 1n })],
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
        record_key: "actors:a",
        fields: [],
        embedded_path: [],
        passage: null,
        source_fingerprint: null,
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
    dc: 5n,
    sources: [{ condition_id: 7n, name: "Grabbed" }],
  });
  expect(result.targeting_check).toEqual({
    dc: 11n,
    sources: [{ condition_id: 8n, name: "Undetected" }],
  });
  expect(result.options.expected_revision).toBe(3n);
  expect(
    JSON.parse(fetchMock.mock.calls[fetchMock.mock.calls.length - 1][1].body),
  ).toEqual({
    strike,
    target_key: "target",
  });
});

it("rejects trait responses from another identity, artifact generation or Similar profile", async () => {
  respond({ trait_key: "trait:deadly-d12" });
  await expect(
    getTraitDetail({
      trait_key: "trait:deadly-d8",
      expected_artifact_generation: null,
    }),
  ).rejects.toThrow("does not match");
  const request = {
    trait_key: "trait:agile",
    kind: "similar" as const,
    expected_artifact_generation: "current",
    search_profile: "bge-base",
    ranking: "score" as const,
    page: { number: 1, size: 20 },
  };
  respond({
    trait_key: "trait:agile",
    kind: "similar",
    artifact_generation: "old",
    search_profile: "bge-base",
  });
  await expect(getTraitRecords(request)).rejects.toMatchObject({
    appError: { code: "artifact_generation_changed" },
  });
  respond({
    trait_key: "trait:agile",
    kind: "similar",
    artifact_generation: "current",
    search_profile: "embeddinggemma",
  });
  await expect(getTraitRecords(request)).rejects.toThrow("selected search model");
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
  respond({
    artifact_generation: "generation-a",
    clause: {
      clause_id: "link",
      relationship: constraint,
      record: {},
      title: "Leather Armor",
    },
    result_record: {},
    occurrences: [],
    page: { number: 2, size: 3, count: 0, total: 9, has_more: true, next_page: 3 },
  });
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
  expect(result.page.total).toBe(9n);
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
  expect(result.actions[0].condition_ids).toEqual([26n]);
  expect(result.actions[1]).toMatchObject({
    condition_ids: [27n],
    check: { dc: 10n, modifier: 3n },
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
    prepared_damage: [],
    prepared_recovery: [],
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
  expect(review.encounter_revision).toBe(2n);
  expect(review.damage_groups[0].selected_source_id).toBe(9n);
  expect(review.damage_groups[0].sources[0].condition_id).toBe(9n);
  expect(review.damage_groups[0].sources[0].recovery_dc).toBe(15n);
});

it("normalizes persistent Add/Edit choices without losing source identity", async () => {
  const api = await import("./atlasApi");
  respond({
    encounter_key: "test",
    encounter_revision: 2,
    consequences: [],
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
  const result = await api.previewEncounterCondition("test", "actor", {
    request_id: "preview",
    expected_encounter_revision: 2n,
    participant_key: "actor",
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
  expect(result.persistent_precedence?.sources[1].source).toEqual({
    kind: "existing",
    condition_id: 45n,
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
    prepared_damage: [],
    prepared_recovery: [],
  });
  const review = await api.getEncounterPersistentReview("encounter", "actor");
  expect(review.episodes[0].damage_resolution).toEqual({
    kind: "known",
    protection: { immune: true, weakness: 2n, resistance: 3n },
  });
  expect(review.damage_result?.groups[0].protection).toEqual({
    immune: true,
    weakness: 2n,
    resistance: 3n,
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
      prepared_damage: [],
      prepared_recovery: [],
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
  respond({
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
    expected_encounter_revision: 7n,
    participant_key: "actor",
    boundary_key: "boundary",
    artifact_generation: "generation",
    groups: [
      {
        episode_id: "fire",
        damage: { kind: "prepared", request_id: "prepared-damage" },
      },
    ],
    recovery: null,
  });
  expect(prepared.damage.groups[0]).toMatchObject({
    source_condition_id: 9n,
    amount: 4n,
    protection: { weakness: 2n, resistance: 3n },
  });
  expect(prepared.recovery).toMatchObject({
    request_id: "prepared-recovery",
    dc: 15n,
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
            damage: { kind: "prepared", request_id: "prepared-damage" },
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
    const result = await api.previewEncounterCondition("test", "actor", {
      request_id: "preview-stunned",
      expected_encounter_revision: 2n,
      participant_key: "actor",
      condition_ref: "conditionitems:dfCMdR4wnpbYNTix",
      value: 2n,
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
        before: 3n,
        after: 1n,
        stunned_after: remainder === null ? null : BigInt(remainder),
      },
    ]);
  },
);

import {
  getReadiness,
  getRecordDetail,
  getRelatedRecords,
  searchPage,
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
import {
  damageChanges,
  healChanges,
  participantUpdate,
} from "../features/encounters/participantEdits";
import { detailFixture } from "../test/fixtures";
const fetchMock = vi.fn();
beforeEach(() => {
  vi.stubGlobal("fetch", fetchMock);
  fetchMock.mockReset();
});
afterEach(() => vi.unstubAllGlobals());
function respond(value: unknown, status = 200) {
  fetchMock.mockImplementation(() =>
    Promise.resolve(
      new Response(JSON.stringify(value), {
        status,
        headers: { "Content-Type": "application/json" },
      }),
    ),
  );
}
describe("source-backed API transport", () => {
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
    };
    respond({ counts: { status: "unavailable", reason: "negative_condition" } });
    expect(await discoverFilterCounts(request, controller.signal)).toEqual({
      counts: { status: "unavailable", reason: "negative_condition" },
    });
    expect(fetchMock.mock.calls[0][1].signal).toBe(controller.signal);
    respond({ code: "query_error", message: "Invalid target" }, 400);
    await expect(discoverFilterCounts(request)).rejects.toThrow("Invalid target");
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
    conditions: [{ condition_id: 9, value: 1, duration_rounds: 4 }],
    display_name: "Ghoul",
    side: "enemy",
    participant_variant: "normal",
    defeated: false,
    hidden: false,
    hp_origin: "derived",
    variant_origin: "default_unadjusted",
    participant_kind: "creature",
    status: "active",
    note_hint: null,
  };
  it("normalizes actual HTTP encounter integers before Damage and Heal", async () => {
    respond({ encounter: { round_number: 1 }, participants: [participantJson] });
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
      conditions: [{ condition_id: 9n, value: 1n, duration_rounds: 4n }],
    });
    expect(damageChanges(participant, 1).current_hp).toBe(19n);
    expect(healChanges(participant, 1).current_hp).toBe(21n);
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
        encounter: { round_number: 1 },
        participants: [{ ...participantJson, revision }],
      });
      await expect(getEncounter("test")).rejects.toBeInstanceOf(AtlasApiError);
    },
  );
  it("normalizes the largest safe revision without rounding", async () => {
    respond({
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
      call: () => setEncounterTurn({ encounter_ref: "test" }),
    },
    {
      name: "condition addition",
      path: "/api/encounters/test/participants/ghoul/conditions",
      method: "POST",
      call: () =>
        addEncounterParticipantCondition("test", {
          participant_key: "ghoul",
          name: "Condition",
        }),
    },
    {
      name: "condition edit",
      path: "/api/encounters/test/participants/ghoul/conditions/9",
      method: "PATCH",
      call: () =>
        updateEncounterParticipantCondition("test", "ghoul", {
          condition_id: 9n,
          name: "Condition",
        }),
    },
    {
      name: "condition removal",
      path: "/api/encounters/test/participants/ghoul/conditions/9",
      method: "DELETE",
      call: () => removeEncounterParticipantCondition("test", "ghoul", 9n),
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
    respond({ encounter: { round_number: 1 }, participants: [unknown] });
    const participant = (await getEncounter("test")).participants[0];
    expect(participant.current_hp).toBeUndefined();
    expect(damageChanges(participant, 1)).toEqual({});
    expect(healChanges(participant, 1)).toEqual({});
    for (const invalid of [
      Number.MAX_SAFE_INTEGER + 1,
      Number.MIN_SAFE_INTEGER - 1,
      0.5,
    ]) {
      respond({
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
            filter: { kind: "compare", field: "actor.level", op: "eq", value },
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
  it("posts exact owned/passage selection without per-marker requests", async () => {
    const controller = new AbortController();
    const detail = detailFixture("actors:ghoul", "Ghoul");
    respond(detail);
    const request = { ...detail.selected, fields: ["system.description.value"] };
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
      expected_artifact_generation: "generation-a",
      criteria: {
        kind: "text_search" as const,
        query: "ghoul",
        mode: "lexical" as const,
        filter: {
          kind: "compare" as const,
          field: "actor.level",
          op: "gte" as const,
          value: 0,
        },
      },
      page: { number: 1, size: 25 },
    };
    const result = await searchPage(request, controller.signal);
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
      context: { kind: "filtered", filter: null, text: null, mode: "lexical" },
      field_id: "common.traits",
      clause_id: null,
      text: null,
      offset: 0,
      limit: 100,
    });
    expect(result.kind).toBe("contextual");
    if (result.kind !== "contextual" || result.outcome.status !== "available")
      throw new Error("Wrong discovery outcome");
    expect(result.outcome.value.options[0].distinct_roots).toBe(4n);
    expect(result.outcome.value.total_values).toBe(1n);
    respond({
      catalog_version: 1,
      limits: { source_bytes: 16384, nodes: 256, depth: 32, literal_list: 128 },
      groups: [],
    });
    expect(
      await discoverFilterEditor({
        selected_field_ids: [],
      }),
    ).toMatchObject({ catalog_version: 1 });
  });
  it("preserves useful saved-list CRUD requests", async () => {
    respond({ items: [] });
    await createSavedList({ slug: "research", name: "Research", tags: [] });
    await filterSavedList({
      list_ref: "Research",
      filter: { kind: "state_match", field: "actor.level", state: "missing" },
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
    fetchMock.mockResolvedValue(new Response("broken", { status: 200 }));
    await expect(getReadiness()).rejects.toBeInstanceOf(AtlasApiError);
  });
});

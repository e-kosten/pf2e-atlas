import {
  getReadiness,
  getRecordDetail,
  openResultWindow,
  readResultWindowPage,
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
} from "./atlasApi";
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
    const result = await discoverFilterValues({
      kind: "vocabulary",
      scope: { kind: "artifact" },
      field_id: "traits",
      text: null,
      offset: 0,
      limit: 40,
      selected_values: ["absent"],
    });
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
    expect(await discoverFilterCounts(request)).toEqual({
      counts: { status: "unavailable", reason: "negative_condition" },
    });
    respond({ code: "query_error", message: "Invalid target" }, 400);
    await expect(discoverFilterCounts(request)).rejects.toThrow("Invalid target");
  });
  const participantJson = {
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
    respond(participantJson);
    expect(
      (
        await updateEncounterParticipant(
          "test",
          participantUpdate(participant, { current_hp: 19n }),
        )
      ).current_hp,
    ).toBe(20n);
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
        openResultWindow({
          mode: {
            kind: "list_records",
            filter: { kind: "compare", field: "actor.level", op: "eq", value },
          },
          page: { number: 1, size: 25 },
        }),
      ).rejects.toThrow("safe numeric range");
      respond({ window_id: value, page: { total: 0 }, rows: [] });
      await expect(
        openResultWindow({
          mode: { kind: "list_records", filter: null },
          page: { number: 1, size: 25 },
        }),
      ).rejects.toThrow("Invalid numeric field");
    }
  });
  it("posts exact owned/passage selection without per-marker requests", async () => {
    const detail = detailFixture("actors:ghoul", "Ghoul");
    respond(detail);
    const request = { ...detail.selected, fields: ["system.description.value"] };
    await getRecordDetail(request.record_key, request);
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(fetchMock.mock.calls[0][0]).toContain("/api/records/detail");
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual(request);
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
    respond({
      window_id: 5,
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
      mode: {
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
    const result = await openResultWindow(request);
    expect(result.window_id).toBe(5n);
    expect(result.page.total).toBe(0n);
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual(request);
    await readResultWindowPage(5n, { page: { number: 2, size: 25 } });
    expect(fetchMock.mock.calls[1][0]).toContain("/5/page");
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
        context: { kind: "filtered", filter: null, text: null, mode: "lexical" },
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

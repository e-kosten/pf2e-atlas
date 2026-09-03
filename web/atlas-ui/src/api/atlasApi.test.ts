import type {
  AppError,
  EncounterSpellCastRequest,
  OpenResultWindowRequest,
} from "../generated/atlas";
import {
  addSavedListItem,
  AtlasApiError,
  createSavedList,
  deleteSavedList,
  discoverFilterEditor,
  discoverFilterValues,
  filterSavedList,
  getReadiness,
  getRecordDetail,
  getSavedList,
  getSavedLists,
  mutateEncounterSpellCast,
  openResultWindow,
  readResultWindowPage,
  removeEncounterParticipantCondition,
  removeSavedListItem,
  resetEncounterParticipant,
  updateEncounterParticipantCondition,
  updateSavedList,
} from "./atlasApi";

describe("atlasApi", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("requests readiness with the expected endpoint", async () => {
    const fetchMock = mockFetch({ status: "ready", message: "Ready" });

    await expect(getReadiness()).resolves.toEqual({
      status: "ready",
      message: "Ready",
    });
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/readiness",
      expect.objectContaining({
        headers: expect.objectContaining({ "Content-Type": "application/json" }),
      }),
    );
  });

  it("posts result-window requests as JSON and normalizes bigint fields", async () => {
    const fetchMock = mockFetch(resultWindowPayload());
    const request: OpenResultWindowRequest = {
      mode: {
        kind: "list_records",
        filter: { clauses: [] },
        sort: { kind: "record_key" },
      },
      page: { number: 1, size: 25 },
      include_diagnostics: false,
    };

    const result = await openResultWindow(request);

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/result-windows",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify(request),
      }),
    );
    expect(result.window_id).toBe(12n);
    expect(result.page.total).toBe(42n);
  });

  it("serializes random sort seeds in result-window requests", async () => {
    const fetchMock = mockFetch(resultWindowPayload());
    const request: OpenResultWindowRequest = {
      mode: {
        kind: "list_records",
        filter: { clauses: [] },
        sort: { kind: "random", seed: 123n },
      },
      page: { number: 1, size: 25 },
      include_diagnostics: false,
    };

    await openResultWindow(request);

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/result-windows",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          ...request,
          mode: {
            ...request.mode,
            sort: { kind: "random", seed: 123 },
          },
        }),
      }),
    );
  });

  it("rejects request bigint fields that exceed the JSON safe integer range", async () => {
    const request: OpenResultWindowRequest = {
      mode: {
        kind: "list_records",
        filter: { clauses: [] },
        sort: { kind: "random", seed: BigInt(Number.MAX_SAFE_INTEGER) + 1n },
      },
      page: { number: 1, size: 25 },
      include_diagnostics: false,
    };

    await expect(openResultWindow(request)).rejects.toMatchObject({
      name: "AtlasApiError",
      message: "Request numeric field exceeds JSON safe integer range",
    });
  });

  it("round-trips encounter numbers without bigint conversion", async () => {
    const fetchMock = mockFetch({
      encounter: { round_number: 2 },
      participants: [
        {
          position: 0,
          initiative: 18,
          initiative_order: 0,
          record_view: {
            metadata: { level: 9 },
            encounter: {
              conditions: [{ condition_id: 12, value: 2, duration_rounds: 3 }],
            },
          },
        },
      ],
    });
    const result = await updateEncounterParticipantCondition(
      "encounter/one",
      "participant/one",
      {
        condition_id: 12,
        name: "Frightened",
        value: 2,
        duration_rounds: 3,
      },
    );

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/encounters/encounter%2Fone/participants/participant%2Fone/conditions/12",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          condition_id: 12,
          name: "Frightened",
          value: 2,
          duration_rounds: 3,
        }),
      }),
    );
    expect(result.encounter.round_number).toBe(2);
    expect(result.participants[0]?.position).toBe(0);
    expect(result.participants[0]?.record_view.metadata.level).toBe(9);
    expect(
      result.participants[0]?.record_view.encounter?.conditions?.[0]?.condition_id,
    ).toBe(12);
  });

  it("rejects unsafe encounter integer requests", async () => {
    await expect(
      updateEncounterParticipantCondition("encounter", "participant", {
        condition_id: Number.MAX_SAFE_INTEGER + 1,
        name: "Unsafe",
      }),
    ).rejects.toMatchObject({
      name: "AtlasApiError",
      message: "Request numeric field exceeds JSON safe integer range",
    });

    await expect(
      removeEncounterParticipantCondition(
        "encounter",
        "participant",
        Number.MAX_SAFE_INTEGER + 1,
      ),
    ).rejects.toMatchObject({
      name: "AtlasApiError",
      message: "Request numeric field exceeds JSON safe integer range",
    });
  });

  it("posts an opaque typed spell-cast target without reconstructing its identity", async () => {
    const request: EncounterSpellCastRequest = {
      spell_occurrence_id: "spell/occurrence",
      spend_target: {
        target_type: "prepared_slot",
        entry_id: "entry/id",
        rank: 4,
        slot_id: "slot4:0",
      },
      operation: "cast_one",
    };
    const resultPayload = {
      operation: "cast_one",
      participant_key: "participant/key",
      spell_occurrence_id: "spell/occurrence",
      before: {
        spend_target: request.spend_target,
        available: true,
        state: {
          state_type: "tracked",
          maximum: 1,
          initial_remaining: 1,
          remaining: 1,
        },
      },
      after: {
        spend_target: request.spend_target,
        available: false,
        state: {
          state_type: "tracked",
          maximum: 1,
          initial_remaining: 1,
          remaining: 0,
        },
        blocked_reason: "exhausted",
      },
      participant: {},
    };
    const fetchMock = mockFetch(resultPayload);

    await expect(
      mutateEncounterSpellCast("encounter/ref", "participant/key", request),
    ).resolves.toEqual(resultPayload);
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/encounters/encounter%2Fref/participants/participant%2Fkey/spell-casts",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify(request),
      }),
    );
  });

  it("rejects unsafe numeric fields in typed spell targets", async () => {
    await expect(
      mutateEncounterSpellCast("encounter", "participant", {
        spell_occurrence_id: "spell",
        spend_target: {
          target_type: "spontaneous_pool",
          entry_id: "entry",
          rank: Number.MAX_SAFE_INTEGER + 1,
        },
        operation: "cast_one",
      }),
    ).rejects.toMatchObject({
      name: "AtlasApiError",
      message: "Request numeric field exceeds JSON safe integer range",
    });
  });

  it("posts the exact typed participant-reset confirmation", async () => {
    const resultPayload = {
      participant_key: "participant/key",
      reset_domains: ["hit_points", "conditions", "spell_resources"],
      preserved_domains: ["display_name", "notes", "visibility", "side"],
      cleared_current_turn: true,
      participant: {},
    };
    const fetchMock = mockFetch(resultPayload);

    await expect(
      resetEncounterParticipant("encounter/ref", "participant/key", {
        confirmation: "reset_participant",
      }),
    ).resolves.toEqual(resultPayload);
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/encounters/encounter%2Fref/participants/participant%2Fkey/reset",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ confirmation: "reset_participant" }),
      }),
    );
  });

  it("posts filter-editor requests and normalizes record counts", async () => {
    const request = {
      context: { kind: "filtered" as const, filter: { clauses: [] } },
    };
    const fetchMock = mockFetch({
      matching_record_count: 42,
      groups: [],
    });

    const result = await discoverFilterEditor(request);

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/filters/editor",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify(request),
      }),
    );
    expect(result.matching_record_count).toBe(42n);
  });

  it("posts filter-value discovery requests and normalizes option counts", async () => {
    const request = {
      context: { kind: "filtered" as const, filter: { clauses: [] } },
      field_id: "traits",
    };
    const fetchMock = mockFetch({
      field_id: "traits",
      matching_record_count: 7,
      options: [
        {
          value: "fire",
          label: "fire",
          count: 3,
          selected: false,
          disabled: false,
          status: "available",
        },
      ],
    });

    const result = await discoverFilterValues(request);

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/filters/values",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify(request),
      }),
    );
    expect(result.matching_record_count).toBe(7n);
    expect(result.options[0]?.count).toBe(3n);
  });

  it("formats bigint window IDs when reading later pages", async () => {
    const fetchMock = mockFetch(resultWindowPayload({ window_id: 99 }));

    const result = await readResultWindowPage(99n, {
      page: { number: 2, size: 25 },
    });

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/result-windows/99/page",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ page: { number: 2, size: 25 } }),
      }),
    );
    expect(result.window_id).toBe(99n);
  });

  it("url-encodes record keys", async () => {
    const fetchMock = mockFetch({
      record_key: "spell:dirge/of doom",
      title: "Dirge",
      kind: "spell",
      presentation: {
        record_key: "spell:dirge/of doom",
        kind: "spell",
        title: "Dirge",
        identity: [],
        badges: [],
        sections: [],
      },
    });

    await getRecordDetail("spell:dirge/of doom");

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/records/spell%3Adirge%2Fof%20doom",
      expect.any(Object),
    );
  });

  it("requests saved lists with the expected endpoint", async () => {
    const fetchMock = mockFetch({ lists: [savedListSummary()] });

    const result = await getSavedLists();

    expect(fetchMock).toHaveBeenCalledWith("/api/lists", expect.any(Object));
    expect(result.lists[0]?.slug).toBe("research");
  });

  it("url-encodes saved-list refs and normalizes item positions", async () => {
    const fetchMock = mockFetch({
      list: savedListSummary({ slug: "campaign/research" }),
      items: [
        {
          record_key: "actions:testAction1",
          position: 2,
          status: "active",
          snapshot: { title: "Test Action 1", kind: "rule" },
          record: {
            record_key: "actions:testAction1",
            title: "Test Action 1",
            kind: "rule",
            kind_label: "Rule",
          },
        },
      ],
    });

    const result = await getSavedList("list_campaign/research");

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/lists/list_campaign%2Fresearch",
      expect.any(Object),
    );
    expect(result.items[0]?.position).toBe(2n);
  });

  it("posts saved-list filters through the encoded list route", async () => {
    const request = {
      list_ref: "list_campaign/research",
      query: "Test Action 2",
      filter: {
        clauses: [
          {
            id: "kind-include_any",
            field: "kind",
            operator: "include_any" as const,
            values: ["action"],
          },
        ],
      },
    };
    const fetchMock = mockFetch({
      list: savedListSummary({ slug: "campaign/research" }),
      items: [],
    });

    await filterSavedList(request);

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/lists/list_campaign%2Fresearch/filter",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify(request),
      }),
    );
  });

  it("writes saved-list mutations through encoded list routes", async () => {
    const fetchMock = mockFetch({
      list: savedListSummary({ slug: "campaign/research" }),
    });

    await createSavedList({
      slug: "campaign/research",
      name: "Campaign Research",
      description: "Session prep",
      tags: ["arc-one"],
    });

    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/lists",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          slug: "campaign/research",
          name: "Campaign Research",
          description: "Session prep",
          tags: ["arc-one"],
        }),
      }),
    );

    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        list: savedListSummary({
          list_key: "list_campaign/research",
          slug: "renamed-research",
          name: "Renamed Research",
          description: "Updated prep",
          tags: ["arc-two"],
        }),
      }),
    );
    await updateSavedList({
      list_key: "list_campaign/research",
      slug: "renamed-research",
      name: "Renamed Research",
      description: "Updated prep",
      tags: ["arc-two"],
    });
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/lists/list_campaign%2Fresearch",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({
          list_key: "list_campaign/research",
          slug: "renamed-research",
          name: "Renamed Research",
          description: "Updated prep",
          tags: ["arc-two"],
        }),
      }),
    );

    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        list_key: "list_campaign/research",
        slug: "campaign/research",
        deleted: true,
      }),
    );
    await deleteSavedList("list_campaign/research");
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/lists/list_campaign%2Fresearch",
      expect.objectContaining({ method: "DELETE" }),
    );

    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        list_key: "list_campaign/research",
        slug: "campaign/research",
        record_key: "spell:dirge/of doom",
        outcome: "added",
      }),
    );
    await addSavedListItem({
      list_ref: "list_campaign/research",
      record_ref: "spell:dirge/of doom",
    });
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/lists/list_campaign%2Fresearch/items",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          list_ref: "list_campaign/research",
          record_ref: "spell:dirge/of doom",
        }),
      }),
    );

    fetchMock.mockResolvedValueOnce(
      jsonResponse({
        list_key: "list_campaign/research",
        slug: "campaign/research",
        record_key: "spell:dirge/of doom",
        outcome: "removed",
      }),
    );
    await removeSavedListItem({
      list_ref: "list_campaign/research",
      record_ref: "spell:dirge/of doom",
    });
    expect(fetchMock).toHaveBeenLastCalledWith(
      "/api/lists/list_campaign%2Fresearch/items",
      expect.objectContaining({
        method: "DELETE",
        body: JSON.stringify({
          list_ref: "list_campaign/research",
          record_ref: "spell:dirge/of doom",
        }),
      }),
    );
  });

  it("throws AtlasApiError with app-error details for app error responses", async () => {
    const appError: AppError = {
      code: "window_expired",
      message: "Expired",
    };
    mockFetch(appError, { ok: false, status: 410, statusText: "Gone" });

    await expect(getReadiness()).rejects.toMatchObject({
      name: "AtlasApiError",
      status: 410,
      message: "Expired",
      appError,
    });
  });

  it("throws AtlasApiError for non-json error responses", async () => {
    mockFetchText("proxy failure", {
      ok: false,
      status: 502,
      statusText: "Bad Gateway",
    });

    await expect(getReadiness()).rejects.toMatchObject({
      name: "AtlasApiError",
      status: 502,
      message: "proxy failure",
      appError: undefined,
    });
  });

  it("throws AtlasApiError for malformed successful JSON", async () => {
    mockFetchText("{", { ok: true, status: 200, statusText: "OK" });

    await expect(getReadiness()).rejects.toBeInstanceOf(AtlasApiError);
  });
});

function mockFetch(
  payload: unknown,
  options: { ok?: boolean; status?: number; statusText?: string } = {},
) {
  return mockFetchText(JSON.stringify(payload), options);
}

function mockFetchText(
  text: string,
  options: { ok?: boolean; status?: number; statusText?: string } = {},
) {
  const fetchMock = vi.fn().mockResolvedValue({
    ok: options.ok ?? true,
    status: options.status ?? 200,
    statusText: options.statusText ?? "OK",
    text: vi.fn().mockResolvedValue(text),
  });
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

function jsonResponse(payload: unknown) {
  return {
    ok: true,
    status: 200,
    statusText: "OK",
    text: vi.fn().mockResolvedValue(JSON.stringify(payload)),
  };
}

function resultWindowPayload(overrides: Record<string, unknown> = {}) {
  return {
    window_id: 12,
    mode: { kind: "list_records" },
    page: {
      number: 1,
      size: 25,
      count: 1,
      total: 42,
      has_more: false,
    },
    rows: [],
    ...overrides,
  };
}

function savedListSummary(overrides: Record<string, unknown> = {}) {
  return {
    list_key: "list_research",
    slug: "research",
    name: "Research",
    description: "Campaign prep",
    tags: ["arc-one"],
    item_count: 1,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

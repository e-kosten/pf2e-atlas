import type { AppError, OpenResultWindowRequest } from "../generated/atlas";
import {
  AtlasApiError,
  discoverFilterFields,
  discoverFilterValues,
  getReadiness,
  getRecordDetail,
  openResultWindow,
  readResultWindowPage,
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

  it("posts filter-field discovery requests and normalizes record counts", async () => {
    const request = {
      context: { kind: "filtered" as const, filter: { clauses: [] } },
    };
    const fetchMock = mockFetch({
      matching_record_count: 42,
      fields: [],
    });

    const result = await discoverFilterFields(request);

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/filters/fields",
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

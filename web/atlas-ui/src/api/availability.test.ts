import {
  AtlasApiError,
  createSavedList,
  getReadiness,
  getRecordDetail,
  getSimilarityParts,
  searchPage,
} from "./atlasApi";
import { errorExplanation } from "../shared/feedback/errorMessages";
import {
  API_CONTRACT,
  API_CONTRACT_HEADER,
  clearCompatibilityFailure,
} from "./compatibility";
import {
  AtlasConnectionError,
  clearAvailabilityFailure,
  getAvailabilityFailure,
  reportAvailabilityFailure,
} from "./availability";

const fetchMock = vi.fn();
beforeEach(() => {
  clearCompatibilityFailure();
  clearAvailabilityFailure();
  fetchMock.mockReset();
  vi.stubGlobal("fetch", fetchMock);
});
afterEach(() => {
  vi.unstubAllGlobals();
  clearAvailabilityFailure();
});

it("keeps native transport diagnostics while distinguishing read-only POSTs from writes", async () => {
  const cause = new TypeError("fetch failed: private proxy detail");
  fetchMock.mockRejectedValue(cause);
  await expect(
    getRecordDetail("actors:ghoul", {
      record_key: "actors:ghoul",
      embedded_path: [],
      fields: [],
      passage: null,
      source_fingerprint: null,
    }),
  ).rejects.toMatchObject({ write: false, cause });
  expect(fetchMock.mock.calls[0][1]).not.toHaveProperty("intent");
  await expect(
    createSavedList({ name: "Research", slug: "research", tags: [] }),
  ).rejects.toMatchObject({ write: true, cause });
});

it("treats an unstructured gateway outage as shared while keeping structured busy errors local", async () => {
  fetchMock.mockResolvedValueOnce(
    new Response("<html>upstream unavailable</html>", { status: 502 }),
  );
  await expect(getReadiness()).rejects.toBeInstanceOf(AtlasConnectionError);
  fetchMock.mockResolvedValueOnce(
    new Response(
      JSON.stringify({ code: "service_busy", message: "SQL queue internals" }),
      { status: 503, headers: { [API_CONTRACT_HEADER]: API_CONTRACT } },
    ),
  );
  await expect(getReadiness()).rejects.toBeInstanceOf(AtlasApiError);
});

it("does not turn an abandoned read into a connection failure", async () => {
  const controller = new AbortController();
  controller.abort();
  const cause = new DOMException("aborted", "AbortError");
  fetchMock.mockRejectedValue(cause);
  await expect(
    getRecordDetail("actors:ghoul", undefined, controller.signal),
  ).rejects.toBe(cause);
  expect(getAvailabilityFailure()).toBeNull();
});

it("retains an unconfirmed write outcome when subsequent background reads fail", () => {
  const write = new AtlasConnectionError(true, "write interrupted");
  reportAvailabilityFailure(write);
  reportAvailabilityFailure(new AtlasConnectionError(false, "read interrupted"));
  expect(getAvailabilityFailure()).toBe(write);
});

it.each([false, true])(
  "preserves the %s write classification when the response stream fails",
  async (write) => {
    const cause = new TypeError("response interrupted");
    const response = new Response(null, {
      headers: { [API_CONTRACT_HEADER]: API_CONTRACT },
    });
    vi.spyOn(response, "text").mockRejectedValue(cause);
    fetchMock.mockResolvedValue(response);
    const request = write
      ? createSavedList({ name: "Research", slug: "research", tags: [] })
      : getReadiness();
    await expect(request).rejects.toMatchObject({
      name: "AtlasConnectionError",
      write,
      cause,
    });
    expect(fetchMock).toHaveBeenCalledOnce();
  },
);

const similarityRequest = {
  seed: { kind: "record" as const, record_key: "actors:ghoul" },
  expected_artifact_generation: null,
};

it.each(["network", "stream", "gateway"] as const)(
  "Similarity inventory %s failure stays a read without uncertain-change feedback",
  async (failure) => {
    const cause = new TypeError("private transport diagnostic");
    if (failure === "network") fetchMock.mockRejectedValue(cause);
    else if (failure === "stream") {
      const response = new Response(null, {
        headers: { [API_CONTRACT_HEADER]: API_CONTRACT },
      });
      vi.spyOn(response, "text").mockRejectedValue(cause);
      fetchMock.mockResolvedValue(response);
    } else
      fetchMock.mockResolvedValue(
        new Response("upstream unavailable", { status: 504 }),
      );
    const error = await getSimilarityParts(similarityRequest).catch(
      (error: unknown) => error,
    );
    expect(error).toBeInstanceOf(AtlasConnectionError);
    expect(error).toMatchObject({ write: false });
    if (failure !== "gateway") expect(error).toHaveProperty("cause", cause);
    expect(errorExplanation(error)).toBeUndefined();
    reportAvailabilityFailure(error);
    expect(getAvailabilityFailure()).toBe(error);
    const write = new AtlasConnectionError(true, "uncertain save");
    reportAvailabilityFailure(write);
    reportAvailabilityFailure(error);
    expect(getAvailabilityFailure()).toBe(write);
    expect(fetchMock).toHaveBeenCalledOnce();
  },
);

it("keeps search POST outages read-only through the real adapter", async () => {
  const cause = new TypeError("connection lost");
  fetchMock.mockRejectedValue(cause);
  await expect(
    searchPage({
      criteria: { kind: "list_records", filter: null },
      page: { number: 1, size: 20 },
      expected_artifact_generation: null,
    }),
  ).rejects.toMatchObject({ write: false, cause });
});

it.each(["network", "stream"] as const)(
  "preserves abandoned Similarity %s failures",
  async (failure) => {
    const controller = new AbortController();
    const cause = new DOMException("abandoned", "AbortError");
    if (failure === "network") fetchMock.mockRejectedValue(cause);
    else {
      const response = new Response(null, {
        headers: { [API_CONTRACT_HEADER]: API_CONTRACT },
      });
      vi.spyOn(response, "text").mockImplementation(async () => {
        controller.abort();
        throw cause;
      });
      fetchMock.mockResolvedValue(response);
    }
    await expect(getSimilarityParts(similarityRequest, controller.signal)).rejects.toBe(
      cause,
    );
    expect(getAvailabilityFailure()).toBeNull();
  },
);

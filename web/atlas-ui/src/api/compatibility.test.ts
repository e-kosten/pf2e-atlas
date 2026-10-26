import { getReadiness, verifyBackendCompatibility } from "./atlasApi";
import {
  API_CONTRACT,
  API_CONTRACT_HEADER,
  AtlasCompatibilityError,
} from "./compatibility";

afterEach(() => vi.unstubAllGlobals());

it("advertises the frontend contract and admits a matching backend", async () => {
  const fetchMock = vi
    .fn()
    .mockResolvedValue(
      new Response("{}", { headers: { [API_CONTRACT_HEADER]: API_CONTRACT } }),
    );
  vi.stubGlobal("fetch", fetchMock);
  await expect(verifyBackendCompatibility()).resolves.toBe(true);
  expect(fetchMock.mock.calls[0][1].headers.get(API_CONTRACT_HEADER)).toBe(
    API_CONTRACT,
  );
  expect(fetchMock.mock.calls[0][1].cache).toBe("no-store");
});

it.each([
  ["old server", 404, null],
  ["different contract", 409, "atlas-web-v1-other"],
  ["old successful API", 200, null],
])("rejects %s before interpreting its body", async (_name, status, contract) => {
  vi.stubGlobal(
    "fetch",
    vi.fn().mockResolvedValue(
      new Response("not JSON", {
        status,
        headers: contract ? { [API_CONTRACT_HEADER]: contract } : {},
      }),
    ),
  );
  await expect(verifyBackendCompatibility()).rejects.toBeInstanceOf(
    AtlasCompatibilityError,
  );
});

it("rejects a backend replaced after startup before normalizing readiness", async () => {
  vi.stubGlobal(
    "fetch",
    vi
      .fn()
      .mockResolvedValue(
        new Response("not JSON", { headers: { [API_CONTRACT_HEADER]: "other" } }),
      ),
  );
  await expect(getReadiness()).rejects.toBeInstanceOf(AtlasCompatibilityError);
});

it("preserves a proxy outage as a connection failure", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn().mockResolvedValue(new Response("Backend unavailable", { status: 502 })),
  );
  await expect(verifyBackendCompatibility()).rejects.not.toBeInstanceOf(
    AtlasCompatibilityError,
  );
});

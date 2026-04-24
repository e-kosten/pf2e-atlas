import { afterEach, describe, expect, it } from "vitest";

import { loadTestService } from "../helpers/pf2e-fixture.js";
import { browseRequest, packFilter, scopeFilter, searchRequest } from "../helpers/search-request-fixture.js";
import { cleanupCreatedRoots, createFixture } from "../helpers/pf2e-service-fixture.js";

describe("Pf2eDataService", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await cleanupCreatedRoots(createdRoots);
  });

  it("loads packs and records from the PF2E filesystem layout", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const stats = service.getStats();
    expect(service.listPacks()).toHaveLength(16);
    expect(stats.packCount).toBe(16);
    expect(stats.recordCount).toBeGreaterThan(0);
    expect(service.getPack("Actions")?.name).toBe("actions");
  });

  it("supports a representative lookup and search flow", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(service.lookup("Raise Shield").match?.name).toBe("Raise a Shield");
    expect(service.listRecords(browseRequest({ filter: packFilter("actions") })).records).toHaveLength(4);
    expect(
      (
        await service.search(
          searchRequest({
            search: { query: "ghost ship" },
            filter: scopeFilter("creature"),
          }),
        )
      ).records.length,
    ).toBeGreaterThan(0);
  });
});

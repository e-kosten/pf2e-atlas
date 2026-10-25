import { beforeEach, expect, it } from "vitest";
import { createMemoryHistory } from "@tanstack/react-router";
import { atlasRouter } from "./router";
import { destinationHref, navigateDestination, ownerReturnHref } from "./navigation";
import { recordDestination, searchDestination, listDestination } from "./destinations";
import { currentSearchLocation } from "./searchLocation";
import { selectedNavigation } from "./urlState";
import type { RecordNavigationView } from "../generated/atlas";

beforeEach(async () => {
  atlasRouter.update({ history: createMemoryHistory({ initialEntries: ["/search"] }) });
  await atlasRouter.load();
});

it("uses generated routes for reserved and Unicode identities without double encoding", async () => {
  const recordKey = "actors:key/with space?query#fragment%雪:tail";
  const selection: RecordNavigationView = {
    record_key: recordKey,
    embedded_path: [{ collection: "items", identity: { SnapshotLocal: { index: 0 } } }],
    field: "system.description.value",
    passage: { kind: "identity" },
    source_fingerprint: "snapshot:full/hash",
    spell_rank: 0,
  };
  const destination = recordDestination({
    recordKey,
    selection,
    section: "atlas-section-雪",
  });
  const href = destinationHref(destination);
  expect(href.split("?")[0]).toBe(
    "/records/actors:key%2Fwith%20space%3Fquery%23fragment%25%E9%9B%AA:tail",
  );
  await navigateDestination(destination);
  expect(
    atlasRouter.state.matches.some(
      (match) => "recordKey" in match.params && match.params.recordKey === recordKey,
    ),
  ).toBe(true);
  expect(selectedNavigation(atlasRouter.state.location.search)).toEqual(selection);
  expect(atlasRouter.state.location.hash).toBe("atlas-section-雪");
});

it("retains repeated sorting and exact selected reading across Search navigation", async () => {
  await atlasRouter.navigate(
    searchDestination({
      selectedRecordKey: "equipment:armor",
      query: "?q=Leather&sort=level_desc&sort=name_asc&filter=3.opaque",
    }),
  );
  expect(currentSearchLocation()).toMatchObject({
    selectedRecordKey: "equipment:armor",
  });
  expect(atlasRouter.state.location.search.sort).toEqual(["level_desc", "name_asc"]);
  expect(atlasRouter.state.location.search.filter).toBe("3.opaque");
  expect(
    destinationHref(
      listDestination({
        slug: "a/b 雪",
        selectedRecordKey: "equipment:armor",
        query: "?sort=name_desc&sort=level_asc",
      }),
    ),
  ).toBe(
    "/lists/a%2Fb%20%E9%9B%AA/records/equipment:armor?sort=name_desc&sort=level_asc",
  );
});

it("retains the owner workspace when opening an embedded full page", async () => {
  await atlasRouter.navigate(
    searchDestination({
      selectedRecordKey: "actors:one",
      query: "?q=one&sort=name_asc",
    }),
  );
  const selection: RecordNavigationView = {
    record_key: "actors:one",
    embedded_path: [{ collection: "items", identity: { SnapshotLocal: { index: 1 } } }],
    field: null,
    passage: null,
    source_fingerprint: "snapshot:one",
  };
  await navigateDestination(recordDestination({ recordKey: "actors:one", selection }));
  const back = ownerReturnHref("actors:one", selection);
  expect(back).toContain("/search/records/actors:one?q=one&sort=name_asc&landing=");
  expect(
    JSON.parse(new URL(back, "http://localhost").searchParams.get("landing")!),
  ).toEqual(selection);
});

import { encodeFilterUrl } from "../shared/filters/filterUrl";
import { beforeEach, expect, it } from "vitest";
import { createMemoryHistory } from "@tanstack/react-router";
import { atlasRouter } from "./router";
import {
  destinationHref,
  navigateDestination,
  navigatePath,
  ownerReturnHref,
} from "./navigation";
import { recordDestination, searchDestination, listDestination } from "./destinations";
import { currentSearchLocation } from "./searchLocation";
import { selectedNavigation, parseSearchParams } from "./urlState";
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
  expect(selectedNavigation(atlasRouter.state.location.search, recordKey)).toEqual(
    selection,
  );
  expect(atlasRouter.state.location.hash).toBe("atlas-section-雪");
});

it("retains repeated sorting and exact selected reading across Search navigation", async () => {
  const filter = encodeFilterUrl({
    kind: "in",
    clause_id: "kind",
    field: "kind",
    values: ["equipment"],
  });
  await atlasRouter.navigate(
    searchDestination({
      selectedRecordKey: "equipment:armor",
      query: `?q=Leather&sort=level_desc&sort=name_asc&filter=${filter}`,
    }),
  );
  expect(currentSearchLocation()).toMatchObject({
    selectedRecordKey: "equipment:armor",
  });
  expect(atlasRouter.state.location.search.sort).toEqual(["level_desc", "name_asc"]);
  expect(atlasRouter.state.location.search.filter).toBe(filter);
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
  expect(back).toContain("/search/records/actors:one?q=one&sort=name_asc&focus=");
  expect(
    selectedNavigation(
      parseSearchParams(new URL(back, "http://localhost").search),
      "actors:one",
      "focus",
    ),
  ).toEqual(selection);
});

it.each(["search", "list", "record"])(
  "returns a directly opened embedded %s URL to the whole parent in that workspace",
  async (kind) => {
    const parent = "actors:direct/雪";
    const selection: RecordNavigationView = {
      record_key: parent,
      embedded_path: [{ collection: "/items", identity: { Stable: "breath" } }],
      field: null,
      passage: null,
      source_fingerprint: null,
      spell_rank: 8,
    };
    const query = "?q=dragon&filter=3.opaque&sort=level_desc&sort=name_asc";
    const input = { selection, view: "explore" as const, tab: "similar" as const };
    const destination =
      kind === "search"
        ? searchDestination({ ...input, selectedRecordKey: parent, query })
        : kind === "list"
          ? listDestination({
              ...input,
              slug: "campaign/雪",
              selectedRecordKey: parent,
              query,
            })
          : recordDestination({ ...input, recordKey: parent });
    const directHref = destinationHref(destination);
    atlasRouter.update({
      history: createMemoryHistory({ initialEntries: [directHref] }),
    });
    await atlasRouter.load();
    expect(atlasRouter.latestLocation.state.atlasOwnerOrigin).toBeUndefined();

    const returnHref = ownerReturnHref(parent, selection);
    const returned = new URL(returnHref, window.location.origin);
    expect(returned.pathname).toBe(
      new URL(directHref, window.location.origin).pathname,
    );
    expect(returned.searchParams.has("at")).toBe(false);
    expect(returned.searchParams.has("rank")).toBe(false);
    expect(returned.searchParams.get("record_view")).toBe("explore");
    expect(returned.searchParams.get("record_tab")).toBe("similar");
    if (kind !== "record") {
      expect(returned.searchParams.get("q")).toBe("dragon");
      expect(returned.searchParams.get("filter")).toBe("3.opaque");
      expect(returned.searchParams.getAll("sort")).toEqual(["level_desc", "name_asc"]);
    }
    expect(
      selectedNavigation(parseSearchParams(returned.search), parent, "focus"),
    ).toEqual(selection);
    await navigatePath(returnHref);
    expect(
      selectedNavigation(atlasRouter.latestLocation.search, parent),
    ).toBeUndefined();
    expect(
      selectedNavigation(atlasRouter.latestLocation.search, parent, "focus"),
    ).toEqual(selection);
  },
);

it("prefers an inherited whole-parent workspace and preserves only its explicit rank", async () => {
  const parent = "spells:parent";
  const root: RecordNavigationView = {
    record_key: parent,
    embedded_path: [],
    field: null,
    passage: null,
    source_fingerprint: null,
    spell_rank: 5,
  };
  await atlasRouter.navigate(
    searchDestination({
      selectedRecordKey: parent,
      selection: root,
      query: "?q=parent",
    }),
  );
  const embedded = {
    ...root,
    embedded_path: [{ collection: "/items", identity: { Stable: "child" } }],
    spell_rank: 8,
  };
  await navigateDestination(
    recordDestination({ recordKey: parent, selection: embedded }),
  );
  const returned = new URL(ownerReturnHref(parent, embedded), window.location.origin);
  expect(returned.pathname).toBe("/search/records/spells:parent");
  expect(returned.searchParams.get("q")).toBe("parent");
  expect(returned.searchParams.get("rank")).toBe("5");
  expect(returned.searchParams.has("at")).toBe(false);
  expect(
    selectedNavigation(parseSearchParams(returned.search), parent, "focus")?.spell_rank,
  ).toBe(8);
});

it.each(["search field", "list passage"])(
  "normalizes an inherited %s origin to its whole parent without losing workspace or root rank",
  async (kind) => {
    const parent = "spells:parent";
    const rootRank = kind === "search field" ? 5 : 0;
    const root: RecordNavigationView = {
      record_key: parent,
      embedded_path: [],
      field: "/system/description/value",
      passage:
        kind === "list passage"
          ? {
              kind: "plain_section",
              selection_version: "content/v10",
              section_ordinal: 2,
              label: "Parent passage",
              chunk_bytes: { start: 10, end: 30 },
              source_text_sha256: "checked-source-text",
            }
          : null,
      source_fingerprint: null,
      spell_rank: rootRank,
    };
    const query = "?q=parent&filter=3.opaque&sort=level_desc&sort=name_asc";
    const destination =
      kind === "search field"
        ? searchDestination({ selectedRecordKey: parent, selection: root, query })
        : listDestination({
            slug: "research",
            selectedRecordKey: parent,
            selection: root,
            query,
          });
    await atlasRouter.navigate(destination);
    const originPath = atlasRouter.latestLocation.pathname;
    const embedded: RecordNavigationView = {
      ...root,
      embedded_path: [{ collection: "/items", identity: { Stable: "child" } }],
      field: null,
      passage: null,
      spell_rank: 8,
    };
    await navigateDestination(
      recordDestination({ recordKey: parent, selection: embedded }),
    );

    const returned = new URL(ownerReturnHref(parent, embedded), window.location.origin);
    expect(returned.pathname).toBe(originPath);
    expect(returned.searchParams.get("q")).toBe("parent");
    expect(returned.searchParams.get("filter")).toBe("3.opaque");
    expect(returned.searchParams.getAll("sort")).toEqual(["level_desc", "name_asc"]);
    expect(returned.searchParams.has("at")).toBe(false);
    expect(selectedNavigation(parseSearchParams(returned.search), parent)).toEqual({
      ...root,
      field: null,
      passage: null,
    });
    expect(
      selectedNavigation(parseSearchParams(returned.search), parent, "focus"),
    ).toEqual(embedded);
  },
);

it.each(["embedded", "sibling"])(
  "does not borrow a whole-parent return context from an inherited %s destination",
  async (kind) => {
    const parent = "actors:parent";
    const selection: RecordNavigationView = {
      record_key: parent,
      embedded_path: [{ collection: "/items", identity: { Stable: "child" } }],
      field: null,
      passage: null,
      source_fingerprint: null,
    };
    const originHref = destinationHref(
      listDestination({
        slug: "other-context",
        selectedRecordKey: kind === "sibling" ? "actors:sibling" : parent,
        selection: kind === "embedded" ? selection : undefined,
      }),
    );
    await atlasRouter.navigate({
      ...recordDestination({ recordKey: parent, selection }),
      state: { atlasOwnerOrigin: { recordKey: parent, href: originHref } },
    });
    const returned = new URL(
      ownerReturnHref(parent, selection),
      window.location.origin,
    );
    expect(returned.pathname).toBe("/records/actors:parent");
    expect(returned.searchParams.has("at")).toBe(false);
    expect(
      selectedNavigation(parseSearchParams(returned.search), parent, "focus"),
    ).toEqual(selection);
  },
);

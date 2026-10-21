import {
  searchStateQueryString,
  DEFAULT_SEARCH_STATE,
} from "../shared/filters/searchState";
import type React from "react";
import {
  atlasRoutePath,
  currentAtlasRoute,
  encounterEditPath,
  encounterPath,
  listEditPath,
  listPath,
  listsPath,
  navigateToAtlasRoute,
  searchPath,
  parseAtlasRoute,
  ownerReturnPath,
  navigateToAtlasPath,
  shouldHandleAtlasRouteClick,
} from "./routes";

describe("atlas routes", () => {
  beforeEach(() => {
    history.replaceState(null, "", "/");
  });

  it("parses search, lists, standalone record, and reader routes", () => {
    expect(currentAtlasRoute()).toEqual({ kind: "search", selectedRecordKey: null });

    history.replaceState(null, "", "/search?q=heal");
    expect(currentAtlasRoute()).toEqual({ kind: "search", selectedRecordKey: null });

    history.replaceState(null, "", "/search/records/spell%3Aheal?q=heal");
    expect(currentAtlasRoute()).toEqual({
      kind: "search",
      selectedRecordKey: "spell:heal",
    });

    history.replaceState(null, "", "/records/spell%3Aheal");
    expect(currentAtlasRoute()).toEqual({ kind: "record", recordKey: "spell:heal" });

    history.replaceState(null, "", "/presentation-mocks");
    expect(currentAtlasRoute()).toEqual({ kind: "search", selectedRecordKey: null });

    history.replaceState(null, "", "/lists");
    expect(currentAtlasRoute()).toEqual({ kind: "lists" });

    history.replaceState(null, "", "/encounters");
    expect(currentAtlasRoute()).toEqual({ kind: "encounters" });

    history.replaceState(null, "", "/encounters/balradon-ambush");
    expect(currentAtlasRoute()).toEqual({
      kind: "encounter",
      slug: "balradon-ambush",
    });

    history.replaceState(null, "", "/encounters/balradon-ambush/edit");
    expect(currentAtlasRoute()).toEqual({
      kind: "encounterEdit",
      slug: "balradon-ambush",
    });

    history.replaceState(null, "", "/lists/session-prep");
    expect(currentAtlasRoute()).toEqual({
      kind: "list",
      slug: "session-prep",
      selectedRecordKey: null,
    });

    history.replaceState(null, "", "/lists/session-prep/edit");
    expect(currentAtlasRoute()).toEqual({
      kind: "listEdit",
      slug: "session-prep",
    });

    history.replaceState(null, "", "/lists/session-prep/records/spell%3Aheal");
    expect(currentAtlasRoute()).toEqual({
      kind: "list",
      slug: "session-prep",
      selectedRecordKey: "spell:heal",
    });

    history.replaceState(null, "", "/reader/spell%3Aheal?preview=condition%3Awounded");
    expect(currentAtlasRoute()).toEqual({
      kind: "reader",
      recordKey: "spell:heal",
      previewRecordKey: "condition:wounded",
    });
  });

  it("builds and navigates route URLs", () => {
    const listener = vi.fn();
    window.addEventListener("atlas-route-change", listener);

    navigateToAtlasRoute({
      kind: "reader",
      recordKey: "spell:heal",
      previewRecordKey: "condition:wounded",
    });

    expect(window.location.pathname).toBe("/reader/spell:heal");
    expect(window.location.search).toBe("?preview=condition%3Awounded");
    expect(listener).toHaveBeenCalledTimes(1);
    expect(atlasRoutePath({ kind: "record", recordKey: "spell:heal" })).toBe(
      "/records/spell:heal",
    );
    expect(atlasRoutePath({ kind: "search", selectedRecordKey: "spell:heal" })).toBe(
      "/search/records/spell:heal",
    );
    expect(
      atlasRoutePath({
        kind: "encounterEdit",
        slug: "balradon-ambush",
      }),
    ).toBe("/encounters/balradon-ambush/edit");
    expect(
      atlasRoutePath({
        kind: "listEdit",
        slug: "session-prep",
      }),
    ).toBe("/lists/session-prep/edit");
    expect(
      atlasRoutePath({
        kind: "list",
        slug: "session-prep",
        selectedRecordKey: "spell:heal",
      }),
    ).toBe("/lists/session-prep/records/spell:heal");

    window.removeEventListener("atlas-route-change", listener);
  });

  it("builds search workspace paths", () => {
    expect(searchPath()).toBe("/search");
    expect(searchPath(null)).toBe("/search");
    expect(searchPath("spell:heal")).toBe("/search/records/spell:heal");
  });

  it("builds list workspace paths", () => {
    expect(encounterPath("balradon-ambush")).toBe("/encounters/balradon-ambush");
    expect(encounterEditPath("balradon-ambush")).toBe(
      "/encounters/balradon-ambush/edit",
    );
    expect(listsPath()).toBe("/lists");
    expect(listPath("session-prep")).toBe("/lists/session-prep");
    expect(listPath("session-prep", "spell:heal")).toBe(
      "/lists/session-prep/records/spell:heal",
    );
    expect(listEditPath("session-prep")).toBe("/lists/session-prep/edit");
  });
  it("keeps literal colons while escaping other path characters and preserving selected identity", () => {
    const recordKey = "actors:key/with space?query#fragment%value:tail";
    const selection = {
      record_key: recordKey,
      embedded_path: [
        { collection: "items", identity: { SnapshotLocal: { index: 2 } } },
      ],
      field: "system.description.value",
      passage: { kind: "identity" as const },
      source_fingerprint: "snapshot:full/hash",
    };
    const path = atlasRoutePath({ kind: "record", recordKey, selection });
    expect(path.split("?")[0]).toBe(
      "/records/actors:key%2Fwith%20space%3Fquery%23fragment%25value:tail",
    );
    const url = new URL(path, "http://localhost");
    expect(parseAtlasRoute(url.pathname, url.search)).toEqual({
      kind: "record",
      recordKey,
      selection,
    });
    const reader = new URL(
      atlasRoutePath({
        kind: "reader",
        recordKey,
        selection,
        previewRecordKey: recordKey,
        previewSelection: selection,
      }),
      "http://localhost",
    );
    expect(parseAtlasRoute(reader.pathname, reader.search)).toEqual({
      kind: "reader",
      recordKey,
      selection,
      previewRecordKey: recordKey,
      previewSelection: selection,
    });
  });
});

it.each(["search", "list", "reader"])(
  "returns an embedded selection to its known %s context with distinct exact landing and native href",
  (kind) => {
    const parent = "actors:checked";
    const selection = {
      record_key: parent,
      embedded_path: [
        { collection: "/items", identity: { SnapshotLocal: { index: 400 } } },
      ],
      field: null,
      passage: null,
      source_fingerprint: "checked-snapshot",
    };
    const query = searchStateQueryString({
      ...DEFAULT_SEARCH_STATE,
      similarSeed: "spells:seed",
      mode: "similar_records",
      filter: { kind: "in", field: "kind", values: ["creature"] },
    });
    const original =
      kind === "search"
        ? "/search/records/actors:checked" + query
        : kind === "list"
          ? "/lists/session/records/actors:checked" + query
          : "/reader/actors:checked" + query;
    history.replaceState(null, "", original);
    navigateToAtlasRoute({ kind: "record", recordKey: parent, selection });
    const selectedUrl = window.location.pathname + window.location.search;
    const href = ownerReturnPath(parent, selection);
    const url = new URL(href, window.location.origin);
    expect(url.pathname).toBe(original.split("?")[0]);
    expect(url.searchParams.get("similar")).toBe("spells:seed");
    expect(url.searchParams.get("filter")).toBe(
      new URLSearchParams(query).get("filter"),
    );
    expect(JSON.parse(url.searchParams.get("landing")!)).toEqual(selection);
    expect(url.searchParams.has("selection")).toBe(false);
    expect(
      shouldHandleAtlasRouteClick({
        button: 0,
        defaultPrevented: false,
        ctrlKey: true,
      } as React.MouseEvent<HTMLElement>),
    ).toBe(false);
    expect(window.location.pathname + window.location.search).toBe(selectedUrl);
    navigateToAtlasPath(href);
    expect(currentAtlasRoute()).toMatchObject({ kind, landing: selection });
  },
);

it("round trips exact trait variant identities independently from record keys", () => {
  const route = { kind: "trait" as const, traitKey: "trait:deadly-d8" };
  expect(atlasRoutePath(route)).toBe("/traits/trait:deadly-d8");
  expect(parseAtlasRoute(atlasRoutePath(route))).toEqual(route);
});

it("preserves saved-list and trait sort URLs in native navigation", () => {
  const list = {
    kind: "list" as const,
    slug: "research",
    selectedRecordKey: "items:armor",
    query: "?q=armor&sort=unit_price_desc",
  };
  expect(parseAtlasRoute("/lists/research/records/items:armor", list.query)).toEqual(
    list,
  );
  expect(atlasRoutePath(list)).toBe("/lists/research/records/items:armor" + list.query);
  const trait = {
    kind: "trait" as const,
    traitKey: "trait:agile",
    query: "?tab=similar&sort=level_desc",
  };
  expect(parseAtlasRoute("/traits/trait:agile", trait.query)).toEqual(trait);
  expect(atlasRoutePath(trait)).toBe("/traits/trait:agile" + trait.query);
});

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
      owners: [{ collection: "items", identity: { SnapshotLocal: { index: 2 } } }],
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
  "returns an owned selection to its known %s context with distinct exact landing and native href",
  (kind) => {
    const parent = "actors:checked";
    const selection = {
      record_key: parent,
      owners: [{ collection: "/items", identity: { SnapshotLocal: { index: 400 } } }],
      field: null,
      passage: null,
      source_fingerprint: "checked-snapshot",
    };
    const query =
      "?similar=spells%3Aseed&filter=%7B%22kind%22%3A%22in%22%2C%22clause_id%22%3A%22kind%22%2C%22field%22%3A%22record.kind%22%2C%22values%22%3A%5B%22Creature%22%5D%7D&q=";
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
      '{"kind":"in","clause_id":"kind","field":"record.kind","values":["Creature"]}',
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

import {
  atlasRoutePath,
  currentAtlasRoute,
  listEditPath,
  listPath,
  listsPath,
  navigateToAtlasRoute,
  searchPath,
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

    history.replaceState(null, "", "/lists");
    expect(currentAtlasRoute()).toEqual({ kind: "lists" });

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

    expect(window.location.pathname).toBe("/reader/spell%3Aheal");
    expect(window.location.search).toBe("?preview=condition%3Awounded");
    expect(listener).toHaveBeenCalledTimes(1);
    expect(atlasRoutePath({ kind: "record", recordKey: "spell:heal" })).toBe(
      "/records/spell%3Aheal",
    );
    expect(atlasRoutePath({ kind: "search", selectedRecordKey: "spell:heal" })).toBe(
      "/search/records/spell%3Aheal",
    );
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
    ).toBe("/lists/session-prep/records/spell%3Aheal");

    window.removeEventListener("atlas-route-change", listener);
  });

  it("builds search workspace paths", () => {
    expect(searchPath()).toBe("/search");
    expect(searchPath(null)).toBe("/search");
    expect(searchPath("spell:heal")).toBe("/search/records/spell%3Aheal");
  });

  it("builds list workspace paths", () => {
    expect(listsPath()).toBe("/lists");
    expect(listPath("session-prep")).toBe("/lists/session-prep");
    expect(listPath("session-prep", "spell:heal")).toBe(
      "/lists/session-prep/records/spell%3Aheal",
    );
    expect(listEditPath("session-prep")).toBe("/lists/session-prep/edit");
  });
});

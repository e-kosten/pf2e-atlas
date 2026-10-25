import { atlasRouter } from "./router";
import { createMemoryHistory } from "@tanstack/react-router";
import {
  destinationHref,
  navigateDestination,
  navigatePath,
  ownerReturnHref,
  isPrimaryActivation,
} from "./navigation";
import {
  recordDestination,
  searchDestination,
  listDestination,
  traitDestination,
  listsDestination,
  encountersDestination,
  encounterDestination,
  encounterEditDestination,
  listEditDestination,
} from "./destinations";
import { selectedNavigation } from "./urlState";
import type { RecordNavigationView } from "../generated/atlas";

beforeEach(async () => {
  atlasRouter.update({ history: createMemoryHistory({ initialEntries: ["/search"] }) });
  await atlasRouter.load();
});

it.each([
  ["/", "/", "search"],
  ["/presentation-mocks", "/$", "search"],
  ["/search?q=heal", "/search/", "search"],
  ["/search/records/spell%3Aheal?q=heal", "/search/records/$recordKey", "search"],
  ["/records/spell%3Aheal", "/records/$recordKey", "reference"],
  ["/traits/trait:deadly-d8", "/traits/$traitKey", "reference"],
  ["/lists", "/lists/", "lists"],
  ["/lists/session-prep", "/lists/$slug/", "lists"],
  ["/lists/session-prep/edit", "/lists/$slug/edit", "lists"],
  [
    "/lists/session-prep/records/spell%3Aheal",
    "/lists/$slug/records/$recordKey",
    "lists",
  ],
  ["/encounters", "/encounters/", "encounters"],
  ["/encounters/ambush", "/encounters/$slug/", "encounters"],
  ["/encounters/ambush/edit", "/encounters/$slug/edit", "encounters"],
])(
  "matches native URL %s with the generated route tree",
  async (href, routeId, section) => {
    await atlasRouter.navigate({ href });
    expect(
      atlasRouter.state.matches[atlasRouter.state.matches.length - 1]?.routeId,
    ).toBe(routeId);
    expect(
      [...atlasRouter.state.matches].reverse().find((match) => match.staticData.section)
        ?.staticData.section,
    ).toBe(section);
    if (href.includes("spell%3Aheal"))
      expect(
        atlasRouter.state.matches[atlasRouter.state.matches.length - 1]?.params,
      ).toMatchObject({ recordKey: "spell:heal" });
  },
);

it("builds native destinations for every workspace", () => {
  expect(destinationHref(searchDestination())).toBe("/search");
  expect(destinationHref(searchDestination({ selectedRecordKey: "spell:heal" }))).toBe(
    "/search/records/spell:heal",
  );
  expect(destinationHref(listsDestination())).toBe("/lists");
  expect(
    destinationHref(listDestination({ slug: "session-prep", selectedRecordKey: null })),
  ).toBe("/lists/session-prep");
  expect(
    destinationHref(
      listDestination({ slug: "session-prep", selectedRecordKey: "spell:heal" }),
    ),
  ).toBe("/lists/session-prep/records/spell:heal");
  expect(destinationHref(listEditDestination({ slug: "session-prep" }))).toBe(
    "/lists/session-prep/edit",
  );
  expect(destinationHref(encountersDestination())).toBe("/encounters");
  expect(destinationHref(encounterDestination({ slug: "ambush" }))).toBe(
    "/encounters/ambush",
  );
  expect(destinationHref(encounterEditDestination({ slug: "ambush" }))).toBe(
    "/encounters/ambush/edit",
  );
  expect(destinationHref(traitDestination({ traitKey: "trait:deadly-d8" }))).toBe(
    "/traits/trait:deadly-d8",
  );
});

it.each(["search", "list", "record"])(
  "returns an embedded selection to its known %s workspace and exact landing",
  async (kind) => {
    const parent = "actors:checked";
    const selection: RecordNavigationView = {
      record_key: parent,
      embedded_path: [
        { collection: "/items", identity: { SnapshotLocal: { index: 400 } } },
      ],
      field: null,
      passage: null,
      source_fingerprint: "checked-snapshot",
    };
    const path =
      kind === "search"
        ? "/search/records/actors:checked"
        : kind === "list"
          ? "/lists/session/records/actors:checked"
          : "/records/actors:checked";
    await atlasRouter.navigate({
      href: path + "?similar=spells:seed&filter=3.opaque&sort=level_desc&sort=name_asc",
    });
    await navigateDestination(recordDestination({ recordKey: parent, selection }));
    const href = ownerReturnHref(parent, selection);
    const url = new URL(href, window.location.origin);
    expect(url.pathname).toBe(path);
    expect(url.searchParams.get("similar")).toBe("spells:seed");
    expect(url.searchParams.get("filter")).toBe("3.opaque");
    expect(url.searchParams.getAll("sort")).toEqual(["level_desc", "name_asc"]);
    expect(JSON.parse(url.searchParams.get("landing")!)).toEqual(selection);
    expect(url.searchParams.has("selection")).toBe(false);
    await navigatePath(href);
    expect(selectedNavigation(atlasRouter.state.location.search, "landing")).toEqual(
      selection,
    );
  },
);

it("keeps document sections separate from exact selections, including malformed escapes", async () => {
  await navigatePath("/records/journals:classes#atlas-section-123-4");
  expect(atlasRouter.state.location.hash).toBe("atlas-section-123-4");
  expect(selectedNavigation(atlasRouter.state.location.search)).toBeUndefined();
  await navigatePath("/records/journals:classes#%");
  expect(atlasRouter.state.location.hash).toBe("%");
});

it("retains sort priority on trait native links", () => {
  expect(
    destinationHref(
      traitDestination({
        traitKey: "trait:agile",
        query: "?tab=similar&sort=level_desc&sort=name_asc",
      }),
    ),
  ).toBe("/traits/trait:agile?tab=similar&sort=level_desc&sort=name_asc");
});

it.each([
  { metaKey: true },
  { ctrlKey: true },
  { altKey: true },
  { shiftKey: true },
  { button: 1 },
  { defaultPrevented: true },
])("preserves native modified activation %j", (modifiers) => {
  expect(
    isPrimaryActivation({
      button: 0,
      defaultPrevented: false,
      metaKey: false,
      ctrlKey: false,
      altKey: false,
      shiftKey: false,
      ...modifiers,
    }),
  ).toBe(false);
});

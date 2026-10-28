import { createMemoryHistory } from "@tanstack/react-router";
import { encodeFilterUrl } from "../shared/filters/filterUrl";
import { createAtlasRouter } from "./router";
import {
  canonicalRouteSearch,
  routeSearch,
  searchPage,
  listIndexSearch,
  encounterIndexSearch,
} from "./routeSearch";
import { parseSearchParams, stringifySearchParams } from "./urlState";
import {
  recordDestination,
  searchDestination,
  listDestination,
  listsDestination,
  encountersDestination,
} from "./destinations";

const filter = encodeFilterUrl({
  kind: "in",
  clause_id: "example",
  field: "kind",
  values: ["spell"],
});

it("normalizes committed values and defaults without dropping explicit ordered sort intent", () => {
  const result = canonicalRouteSearch(
    "search",
    parseSearchParams(
      `?unused=yes&q=++fire++&q=++fire++&filter=${filter}&limit=025&ranking=score&sort=level_desc&sort=name_asc&page=005`,
    ),
  );
  expect(stringifySearchParams(result)).toBe(
    `?ranking=score&q=fire&filter=${filter}&sort=level_desc&sort=name_asc&page=5`,
  );
  expect(canonicalRouteSearch("search", result)).toEqual(result);
  expect(searchDestination({ query: "?q=fire&page=5", page: 1 }).search).toEqual({
    q: "fire",
  });
  expect(searchDestination({ query: "?q=fire", page: 5 }).search).toEqual({
    q: "fire",
    page: "5",
  });
});

it.each(["search", "search_record"] as const)(
  "retains explicit comparison settings and repeated section selections on %s",
  (kind) => {
    const settings = {
      search_profile: "bge-base",
      ranking: "score",
      similar: "creatures:lich",
      search_content: "core",
      similarity_method: "passages",
      candidate_parts: "corresponding",
      similarity_sections: "custom",
      similarity_decay: "0.25",
      similarity_generation: "generation-a",
      similarity_part: ["identity:lich", "description:lich"],
    };
    const result = canonicalRouteSearch(kind, settings);
    expect(result).toEqual(settings);
    expect(canonicalRouteSearch(kind, result)).toEqual(result);
    expect(
      searchDestination({ query: stringifySearchParams(settings) }).search,
    ).toEqual(settings);
  },
);

it("retains explicit All and score settings over different browser preferences", () => {
  expect(
    canonicalRouteSearch("search", {
      ranking: ["score", "score"],
      search_content: ["all", "all"],
      similar: "creatures:lich",
      similarity_sections: ["all", "all"],
    }),
  ).toEqual({
    ranking: "score",
    search_content: "all",
    similar: "creatures:lich",
    similarity_sections: "all",
  });
});

it("accepts only lexical list criteria and applicable ordering", () => {
  const search = listDestination({
    slug: "research",
    selectedRecordKey: null,
    query: `?q=+fire+&similar=spells:seed&trait_refs=trait:fire&filter=${filter}&limit=100&ranking=rrf&search_profile=other&page=5&sort=saved_order`,
  }).search;
  expect(search).toEqual({ q: "fire", filter, sort: "saved_order" });
});

it("preserves only explicit trait profiles with singleton validation and canonical trimming", () => {
  expect(
    canonicalRouteSearch("trait", {
      search_profile: [" embeddinggemma ", " embeddinggemma "],
      tab: "similar",
    }),
  ).toEqual({ search_profile: "embeddinggemma", tab: "similar" });
  expect(canonicalRouteSearch("trait", { search_profile: "  " })).toEqual({});
  expect(canonicalRouteSearch("trait", {})).toEqual({});
  expect(() =>
    canonicalRouteSearch("trait", { search_profile: ["bge-base", "embeddinggemma"] }),
  ).toThrow("conflicting search_profile");
  for (const kind of ["record", "list", "list_record"] as const)
    expect(canonicalRouteSearch(kind, { search_profile: "embeddinggemma" })).toEqual(
      {},
    );
});

it("uses existing tag normalization and ordinary index parameters", () => {
  const search = listsDestination({
    query: " castle ",
    include: [" Prep ", "prep", "Alpha"],
    exclude: [" old "],
  }).search;
  expect(search).toEqual({ q: "castle", tag: ["Alpha", "Prep"], exclude_tag: ["old"] });
  expect(listIndexSearch(search)).toEqual({
    query: "castle",
    include: ["Alpha", "Prep"],
    exclude: ["old"],
  });
  expect(encountersDestination({ view: "active" }).search).toEqual({});
  expect(encounterIndexSearch({ view: "archived" })).toEqual({ view: "archived" });
  expect(() => canonicalRouteSearch("encounters", { view: "other" })).toThrow(
    "view parameter",
  );
});

it("omits irrelevant record parameters while preserving exact rank and view", () => {
  expect(
    canonicalRouteSearch("record", {
      rank: ["000", "000"],
      record_view: "read",
      record_tab: "incoming",
      q: "ignored",
      page: "5",
      filter,
    }),
  ).toEqual({ rank: "0" });
  expect(
    canonicalRouteSearch("trait", {
      tab: "members",
      q: "ignored",
      sort: ["level_desc", "name_asc"],
      seed: "invalid",
    }),
  ).toEqual({ sort: ["level_desc", "name_asc"], seed: "invalid" });
  expect(canonicalRouteSearch("empty", { q: "ignored", rank: "bad" })).toEqual({});
});

it.each(["0", "-1", "1.5", "oops", "4294967296"])(
  "normalizes benign invalid page %s to page one",
  (page) => {
    expect(canonicalRouteSearch("search", { page })).toEqual({});
    expect(searchPage({ page })).toBe(1);
  },
);

it("retains the full accepted page-size range", () => {
  expect(canonicalRouteSearch("search", { limit: "5" })).toEqual({ limit: "5" });
  expect(canonicalRouteSearch("search", { limit: "100" })).toEqual({ limit: "100" });
  expect(canonicalRouteSearch("search", { limit: "101" })).toEqual({});
});

it.each(["search", "list", "trait"] as const)(
  "canonicalizes a valid random seed on %s and retains invalid seed errors",
  (kind) => {
    expect(canonicalRouteSearch(kind, { sort: "random", seed: "00042" })).toEqual({
      sort: "random",
      seed: "42",
    });
    expect(canonicalRouteSearch(kind, { sort: "random", seed: "bad" })).toEqual({
      sort: "random",
      seed: "bad",
    });
  },
);

it.each([
  "q",
  "page",
  "limit",
  "similar",
  "seed",
  "search_profile",
  "ranking",
  "search_content",
  "similarity_method",
  "candidate_parts",
  "similarity_sections",
  "similarity_decay",
  "similarity_generation",
])("rejects conflicting singleton %s values", (key) => {
  expect(() => canonicalRouteSearch("search", { [key]: ["1", "2"] })).toThrow(
    "conflicting",
  );
});

it("leaves failed filter transport untouched for shared empty-search recovery before reading validation", () => {
  const raw = {
    q: " retained until reset ",
    filter: "3.broken",
    rank: "bad",
    selection: "obsolete",
    unused: "keep until reset",
  };
  expect(canonicalRouteSearch("search_record", raw)).toBe(raw);
  const location = {
    ...createAtlasRouter(
      createMemoryHistory({ initialEntries: ["/search/records/spells:one"] }),
    ).latestLocation,
    search: raw,
  };
  expect(() => routeSearch("search_record").beforeLoad({ location })).not.toThrow();
  expect(canonicalRouteSearch("search", { filter: [filter, "3.broken"] })).toEqual({
    filter: [filter, "3.broken"],
  });
  expect(() => canonicalRouteSearch("search_record", { filter, rank: "bad" })).toThrow(
    "invalid spell rank",
  );
});

it("canonicalizes incoming URLs in place while preserving fragments and navigation state", async () => {
  const history = createMemoryHistory({
    initialEntries: [
      "/records/spells:one?rank=008&record_view=read&unused=yes#literal%2523",
    ],
  });
  const router = createAtlasRouter(history);
  await router.load();
  expect(router.latestLocation.href).toBe("/records/spells:one?rank=8#literal%2523");
  expect(history.length).toBe(1);
  await router.navigate({
    to: "/search",
    search: { q: " fire ", page: "005", unused: "yes" },
    state: { atlasSearchReset: "entry", atlasFilterResetNotice: true },
  });
  expect(router.latestLocation.href).toBe("/search?q=fire&page=5");
  expect(router.latestLocation.state).toMatchObject({
    atlasSearchReset: "entry",
    atlasFilterResetNotice: true,
  });
});

it("normalizes the root to Search without changing supported criteria or native fragments", async () => {
  const history = createMemoryHistory({
    initialEntries: ["/?q=+fire+&page=2#section"],
  });
  const router = createAtlasRouter(history);
  await router.load();
  expect(router.latestLocation.href).toBe("/search?q=fire&page=2#section");
  expect(history.length).toBe(1);
  expect(recordDestination({ recordKey: "spells:one" }).search).toEqual({});
});

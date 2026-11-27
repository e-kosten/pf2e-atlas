import { readUrl } from "../readingUrl";
import type { Page, Request } from "@playwright/test";
import { expect, test } from "./fixtures";
import {
  activate,
  creatureFilter,
  noPageOverflow,
  showSearchFilters,
  sentinel,
} from "./reading";

test("a records-only workspace supports Search, trait browse and the list picker", async ({
  page,
}, info) => {
  const readiness = await (await page.request.get("/api/readiness")).json();
  expect(readiness.status).toBe("ready");
  expect(
    readiness.search_profiles.every(
      (profile: { stored_vectors: boolean }) => !profile.stored_vectors,
    ),
  ).toBe(true);

  await page.goto(`/search?filter=${creatureFilter}`);
  const keyword = page.getByText("Keyword search", { exact: true });
  await expect(keyword).toBeVisible();
  const next = page.getByRole("button", { name: "Next page", exact: true });
  await activate(next, info);
  await expect(page.getByRole("textbox", { name: "Current page" })).toHaveValue("2");
  await showSearchFilters(page, info);
  const kind = page.getByRole("button", { name: "Edit Kind filter", exact: true });
  const counts = page.waitForResponse((candidate) =>
    candidate.url().endsWith("/api/filters/counts"),
  );
  await activate(kind, info);
  const countResponse = await counts;
  expect(countResponse.request().postDataJSON().context.search_profile).toBeUndefined();
  const countPage = await countResponse.json();
  expect(countPage.counts.status).toBe("available");
  expect(
    countPage.counts.value.options.find(
      (option: { value: string }) => option.value === "creature",
    ).distinct_roots,
  ).toBeGreaterThan(0);
  await page.keyboard.press("Escape");

  const query = page.getByRole("combobox", { name: "Search records" });
  await query.fill("Atlas Sentinel");
  const search = page.waitForResponse(
    (response) =>
      response.url().endsWith("/api/search/page") &&
      response.request().postDataJSON()?.criteria?.kind === "text_search",
  );
  await query.press("Enter");
  const response = await search;
  const request = response.request().postDataJSON();
  expect(request.search_profile).toBeUndefined();
  expect(request.criteria.mode).toBe("lexical");
  expect(response.ok()).toBe(true);

  const vocabulary = page.waitForResponse(
    (candidate) =>
      candidate.url().endsWith("/api/filters/values") &&
      candidate.request().postDataJSON()?.field_id === "traits",
  );
  await activate(page.getByRole("button", { name: "Edit Traits filter" }), info);
  expect((await vocabulary).request().postDataJSON().kind).toBe("vocabulary");
  await expect(
    page.getByRole("button", { name: /^Traits: Light, neutral;/ }),
  ).toBeVisible();
  await page.keyboard.press("Escape");

  await activate(page.locator(`[data-record-key="${sentinel}"]`), info);
  await expect(
    page.getByRole("heading", { name: "Atlas Sentinel", exact: true }),
  ).toBeVisible();
  await noPageOverflow(page);

  await page.goto("/traits/trait:light");
  await activate(
    page.getByRole("link", { name: "Search records with this trait" }),
    info,
  );
  await showSearchFilters(page, info);
  await expect(page.getByRole("button", { name: "Edit Traits filter" })).toBeVisible();
  await expect(page.locator("[data-record-key]").first()).toBeVisible();

  const slug = `keyword-${info.project.name}`;
  const created = await page.request.post("/api/lists", {
    data: { name: `Keyword test list ${info.project.name}`, slug, tags: [] },
  });
  expect(created.ok()).toBe(true);
  await page.goto(`/lists/${slug}`);
  await activate(page.getByRole("button", { name: "Add records", exact: true }), info);
  const picker = page.getByRole("dialog", { name: "Add records", exact: true });
  const pickerQuery = picker.getByRole("searchbox", { name: "Search records" });
  await pickerQuery.fill("Atlas Sentinel");
  await pickerQuery.press("Enter");
  await activate(picker.locator(`[data-record-key="${sentinel}"]`), info);
  await activate(picker.getByRole("button", { name: "Add", exact: true }), info);
  await expect(
    picker.getByRole("status").filter({ hasText: "Added to list." }),
  ).toBeVisible();
  await activate(picker.getByRole("button", { name: "Cancel", exact: true }), info);
  await expect(picker).toBeHidden();
  await expect(page.getByText("Atlas Sentinel", { exact: true }).first()).toBeVisible();
});

test("inline record links work without stored vectors", async ({ page }, info) => {
  const slug = `keyword-inline-${info.project.name}`;
  const created = await page.request.post("/api/lists", {
    data: { slug, name: "Keyword inline lookup" },
  });
  expect(created.ok()).toBe(true);
  const { list } = await created.json();
  try {
    await page.goto(`/lists/${slug}/edit`);
    const editor = page.getByRole("textbox", { name: "Description" });
    await editor.pressSequentially("[[Atlas Sentin");
    const suggestions = page.getByRole("listbox", { name: "Record link suggestions" });
    await activate(
      suggestions.getByRole("option", { name: /^Atlas Sentinel\s*Creature/ }).first(),
      info,
    );
    const inserted = `[Atlas Sentinel](record:${sentinel})`;
    await expect(editor).toContainText(inserted);
    await editor.pressSequentially(" and [[Lantern Ward");
    await activate(suggestions.getByRole("option", { name: "Open full search" }), info);
    const picker = page.getByRole("dialog", { name: "Insert record link" });
    await expect(picker.getByRole("searchbox", { name: "Search records" })).toHaveValue(
      "Lantern Ward",
    );
    await activate(
      picker.locator('[data-record-key="browser-spells:spell00000000001"]'),
      info,
    );
    await activate(picker.getByRole("button", { name: "Insert link" }), info);
    await expect(editor).toBeFocused();
    await expect(editor).toContainText(
      `${inserted} and [Lantern Ward](record:browser-spells:spell00000000001)`,
    );
  } finally {
    await page.request.delete(`/api/lists/${list.list_key}`);
  }
});

async function holdResponse(
  page: Page,
  path: string,
  matches: (request: Request) => boolean = () => true,
) {
  const arrived = Promise.withResolvers<void>();
  const gate = Promise.withResolvers<void>();
  await page.route(path, async (route) => {
    if (!matches(route.request())) return route.fallback();
    arrived.resolve();
    await gate.promise;
    await route.continue();
  });
  return { arrived: arrived.promise, release: gate.resolve };
}

test("keeps submitted text and newer drafts through readiness and results", async ({
  page,
}, info) => {
  await page.goto("/search");
  await expect(page.getByText("Keyword search", { exact: true })).toBeVisible();
  const query = page.getByRole("combobox", { name: "Search records" });
  await query.fill("Atlas Sentinel");
  const readiness = await holdResponse(page, "**/api/readiness");
  let submittedRequests = 0;
  const results = await holdResponse(page, "**/api/search/page", (request) => {
    const criteria = request.postDataJSON().criteria;
    if (
      criteria.kind !== "text_search" ||
      criteria.query !== "Atlas Sentinel" ||
      criteria.lexical
    )
      return false;
    submittedRequests += 1;
    return true;
  });

  try {
    await query.press("Enter");
    await readiness.arrived;
    expect(await query.inputValue()).toBe("Atlas Sentinel");
    await expect(page).toHaveURL(/[?&]q=Atlas(?:\+|%20)Sentinel/);
    const exact = page.getByRole("region", { name: "Exact matches" });
    await activate(exact.getByRole("link", { name: /^Atlas Sentinel/ }), info);
    await expect(
      page.getByRole("heading", { name: "Atlas Sentinel", exact: true }),
    ).toBeVisible();
    expect(submittedRequests).toBe(0);
    const back = page.getByRole("button", { name: "Results", exact: true });
    if (await back.isVisible()) await activate(back, info);
    await query.fill("new draft during readiness");
  } finally {
    readiness.release();
  }

  try {
    await results.arrived;
    expect(new URL(page.url()).searchParams.get("q")).toBe("Atlas Sentinel");
    await expect(query).toHaveValue("new draft during readiness");
    await query.fill("new draft during results");
  } finally {
    results.release();
  }
  await expect(page.locator(`[data-record-key="${sentinel}"]`)).toBeVisible();
  await expect(query).toHaveValue("new draft during results");
  expect(submittedRequests).toBe(1);
  expect(new URL(page.url()).pathname).toBe(`/search/records/${sentinel}`);

  const nextSearch = page.waitForRequest(
    (request) =>
      request.url().endsWith("/api/search/page") &&
      request.postDataJSON().criteria.query === "new draft during results" &&
      !request.postDataJSON().criteria.lexical,
  );
  await page.getByRole("button", { name: "search", exact: true }).click();
  await nextSearch;
  await expect(query).toHaveValue("new draft during results");
});

test("cancels a pager edit and clears a submission while readiness is pending", async ({
  page,
}) => {
  await page.goto(`/search?filter=${creatureFilter}`);
  await expect(page.getByText("Keyword search", { exact: true })).toBeVisible();
  const current = page.getByRole("textbox", { name: "Current page" });
  await expect(current).toHaveValue("1");
  const originalUrl = page.url();
  const pageRequests: number[] = [];
  page.on("request", (request) => {
    if (request.url().endsWith("/api/search/page"))
      pageRequests.push(request.postDataJSON().page.number);
  });
  await current.fill("2");
  await expect(current).toBeFocused();
  await current.press("Escape");
  await expect(current).toHaveValue("1");
  await expect(current).not.toBeFocused();
  expect(page.url()).toBe(originalUrl);
  expect(pageRequests).not.toContain(2);

  const query = page.getByRole("combobox", { name: "Search records" });
  await query.fill("Atlas Sentinel");
  const readiness = await holdResponse(page, "**/api/readiness");
  const ready = page.waitForResponse((response) =>
    response.url().endsWith("/api/readiness"),
  );
  try {
    await query.press("Enter");
    await readiness.arrived;
    await expect(page).toHaveURL(/[?&]q=Atlas(?:\+|%20)Sentinel/);
    await expect(current).toBeHidden();
    await query.fill("");
    await expect.poll(() => new URL(page.url()).searchParams.get("q")).toBeNull();
  } finally {
    readiness.release();
  }
  expect((await ready).ok()).toBe(true);
  await current.fill("2");
  await current.press("Enter");
  await expect(page).toHaveURL(/[?&]page=2(?:&|$)/);
  await expect(current).toHaveValue("2");
  await expect(query).toHaveValue("");
  expect(new URL(page.url()).searchParams.get("q")).toBeNull();
  expect(pageRequests.filter((number) => number === 2)).toHaveLength(1);
});

test("opens a name suggestion without submitting the draft", async ({ page }, info) => {
  await page.goto("/search");
  await expect(page.getByText("Keyword search", { exact: true })).toBeVisible();
  const query = page.getByRole("combobox", { name: "Search records" });
  await query.fill("atlas sentin");
  const option = page.getByRole("option", { name: /^Atlas Sentinel/ });
  await expect(option).toBeVisible();
  await query.press("Escape");
  await expect(option).toBeHidden();
  await expect(query).toHaveValue("atlas sentin");
  await query.fill("Atlas Sentin");
  await expect(option).toBeVisible();
  if (info.project.use.hasTouch) {
    await activate(option, info);
  } else {
    await query.press("ArrowDown");
    await expect(option).toHaveAttribute("aria-selected", "true");
    await query.press("Enter");
  }
  await expect(
    page.getByRole("heading", { name: "Atlas Sentinel", exact: true }),
  ).toBeVisible();
  expect(new URL(page.url()).searchParams.get("q")).toBeNull();
  const back = page.getByRole("button", { name: "Results", exact: true });
  if (await back.isVisible()) await activate(back, info);
  await expect(query).toHaveValue("Atlas Sentin");
});

test("keeps newer search text in the Add records picker", async ({ page }, info) => {
  const slug = `draft-${info.project.name}`;
  const created = await page.request.post("/api/lists", {
    data: { name: `Picker draft test ${info.project.name}`, slug, tags: [] },
  });
  expect(created.ok()).toBe(true);
  await page.goto(`/lists/${slug}`);
  const add = page.getByRole("button", { name: "Add records", exact: true });
  await activate(add, info);
  const picker = page.getByRole("dialog", { name: "Add records", exact: true });
  const query = picker.getByRole("searchbox", { name: "Search records" });
  await expect(picker.locator("[data-record-key]").first()).toBeVisible();
  await query.fill("Atlas Sentinel");
  const readiness = await holdResponse(page, "**/api/readiness");
  const matches = (request: Request) =>
    request.url().endsWith("/api/search/page") &&
    request.postDataJSON().criteria.query === "Atlas Sentinel";
  const requests: Request[] = [];
  page.on("request", (request) => {
    if (matches(request)) requests.push(request);
  });
  const searched = page.waitForResponse((response) => matches(response.request()));
  try {
    await query.press("Enter");
    await readiness.arrived;
    expect(await query.inputValue()).toBe("Atlas Sentinel");
    expect(requests).toHaveLength(0);
    await query.fill("unfinished picker draft");
  } finally {
    readiness.release();
  }
  const response = await searched;
  expect(response.ok()).toBe(true);
  const { page: resultPage } = await response.json();
  await expect(
    picker.getByText(`${resultPage.total} records`, { exact: true }),
  ).toBeVisible();
  await expect(picker.locator(`[data-record-key="${sentinel}"]`)).toBeVisible();
  await expect(query).toHaveValue("unfinished picker draft");

  await activate(picker.getByRole("button", { name: "Cancel", exact: true }), info);
  await expect(picker).toBeHidden();
  await activate(add, info);
  await expect(query).toHaveValue("");
});

test("an exact journal page opens from its result with explanations hidden", async ({
  page,
  context,
}, info) => {
  await page.addInitScript(() =>
    localStorage.setItem(
      "atlas.search.preferences.v1",
      JSON.stringify({ version: 1, matchDisplay: "hidden" }),
    ),
  );
  await page.goto("/search?q=%22Class%20Features%22");
  const link = page.getByRole("link", {
    name: "Remaster Changes › Class Features",
    exact: true,
  });
  await expect(link).toBeVisible();
  const resultsUrl = page.url();
  const target = readUrl(new URL((await link.getAttribute("href"))!, page.url()));
  expect(target).toMatchObject({
    record_key: "browser-journals:editionjournal01",
    embedded_path: [{ collection: "/pages", identity: { Stable: "editionpage00001" } }],
    field: null,
    passage: null,
  });
  if (!info.project.use.hasTouch) {
    const opened = context.waitForEvent("page");
    await link.click({ modifiers: ["ControlOrMeta"] });
    const native = await opened;
    await expect(
      native.getByRole("heading", { name: "Class Features", exact: true }).first(),
    ).toBeVisible();
    expect(readUrl(new URL(native.url()))).toEqual(target);
    await native.close();
    await link.focus();
    await link.press("Enter");
  } else {
    await activate(link, info);
  }
  await expect(
    page.getByRole("heading", { name: "Class Features", exact: true }).first(),
  ).toBeVisible();
  await expect(page).toHaveURL(
    (url) => url.pathname.startsWith("/search/records/") && url.searchParams.has("at"),
  );
  expect(readUrl(new URL(page.url()))).toEqual(target);
  await page.goBack();
  await expect(page).toHaveURL(resultsUrl);
  await expect(link).toBeVisible();
  await activate(
    page.locator('[data-record-key="browser-journals:editionjournal01"]').first(),
    info,
  );
  await expect(page).toHaveURL(
    (url) =>
      url.pathname.endsWith("/browser-journals:editionjournal01") &&
      !url.searchParams.has("at"),
  );
});

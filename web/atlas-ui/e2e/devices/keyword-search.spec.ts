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

  const query = page.getByRole("searchbox", { name: "Search records" });
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
    await page.request.delete(`/api/lists/${slug}`);
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
}) => {
  await page.goto("/search");
  await expect(page.getByText("Keyword search", { exact: true })).toBeVisible();
  const query = page.getByRole("searchbox", { name: "Search records" });
  await query.fill("Atlas Sentinel");
  const readiness = await holdResponse(page, "**/api/readiness");
  let sent = false;
  const results = await holdResponse(page, "**/api/search/page", (request) => {
    const criteria = request.postDataJSON().criteria;
    if (criteria.kind !== "text_search" || criteria.query !== "Atlas Sentinel")
      return false;
    sent = true;
    return true;
  });

  try {
    await query.press("Enter");
    await readiness.arrived;
    expect(await query.inputValue()).toBe("Atlas Sentinel");
    expect(sent).toBe(false);
    expect(new URL(page.url()).searchParams.get("q")).toBeNull();
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

  const nextSearch = page.waitForRequest(
    (request) =>
      request.url().endsWith("/api/search/page") &&
      request.postDataJSON().criteria.query === "new draft during results",
  );
  await page.getByRole("button", { name: "search", exact: true }).click();
  await nextSearch;
  await expect(query).toHaveValue("new draft during results");
});

test("retains a canceled draft on paging and cancels a pager edit on Escape", async ({
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

  const query = page.getByRole("searchbox", { name: "Search records" });
  await query.fill("Atlas Sentinel");
  const readiness = await holdResponse(page, "**/api/readiness");
  const ready = page.waitForResponse((response) =>
    response.url().endsWith("/api/readiness"),
  );
  const nextPage = page.waitForResponse(
    (response) =>
      response.url().endsWith("/api/search/page") &&
      response.request().postDataJSON().page.number === 2,
  );
  try {
    await query.press("Enter");
    await readiness.arrived;
    await current.fill("2");
    await current.press("Enter");
    await expect(page).toHaveURL(/[?&]page=2(?:&|$)/);
    expect(new URL(page.url()).searchParams.get("q")).toBeNull();
  } finally {
    readiness.release();
  }
  expect((await ready).ok()).toBe(true);
  expect((await nextPage).ok()).toBe(true);
  await expect(current).toHaveValue("2");
  await expect(query).toHaveValue("Atlas Sentinel");
  expect(new URL(page.url()).searchParams.get("q")).toBeNull();
  expect(pageRequests.filter((number) => number === 2)).toHaveLength(1);
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

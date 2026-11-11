import { expect, test } from "./fixtures";
import { activate, creatureFilter, noPageOverflow, sentinel } from "./reading";

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
  await expect(page.getByRole("button", { name: "Edit Traits filter" })).toBeVisible();
  await expect(page.locator("[data-record-key]").first()).toBeVisible();

  const slug = `keyword-${info.project.name}`;
  const created = await page.request.post("/api/lists", {
    data: { name: "Keyword test list", slug, tags: [] },
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

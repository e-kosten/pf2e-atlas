import { expect, test, type APIRequestContext } from "@playwright/test";
import type {
  TraitDetailView,
  TraitRecordsPage,
  RecordNavigationView,
} from "../src/generated/atlas";

// Every POST in this file is an ordinary read API. The caller supplies the real
// server and isolated full-corpus workspace; no source or saved state is changed.
async function definition(request: APIRequestContext, traitKey: string) {
  const response = await request.post("/api/traits/detail", {
    data: { trait_key: traitKey, expected_artifact_generation: null },
  });
  expect(response.ok(), await response.text()).toBeTruthy();
  return (await response.json()) as TraitDetailView;
}
async function memberships(request: APIRequestContext, detail: TraitDetailView) {
  const response = await request.post("/api/traits/records", {
    data: {
      trait_key: detail.trait_key,
      kind: "members",
      page: { number: 1, size: 20 },
      expected_artifact_generation: detail.artifact_generation,
      search_profile: null,
      ranking: "score",
    },
  });
  expect(response.ok(), await response.text()).toBeTruthy();
  return (await response.json()) as TraitRecordsPage;
}
function recordPath(navigation: RecordNavigationView) {
  return `/records/${encodeURIComponent(navigation.record_key).replace(/%3A/g, ":")}?selection=${encodeURIComponent(JSON.stringify(navigation))}`;
}

test("real trait chip opens its definition and Back returns to the selected source", async ({
  page,
  request,
}) => {
  const detail = await definition(request, "trait:deadly-d8");
  const record = "equipment-srd:tH5GirEy7YB3ZgCk";
  await page.goto(`/records/${record}`);
  const chip = page.locator('a[href="/traits/trait:deadly-d8"]').first();
  await expect(chip).toBeVisible();
  await chip.click();
  await expect(page).toHaveURL(/\/traits\/trait:deadly-d8$/);
  await expect(page.getByRole("button", { name: "Back", exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Refresh", exact: true })).toHaveCount(
    0,
  );
  await expect(page.locator(".record-reference__kind")).toHaveText("Trait");
  await expect(
    page.getByRole("heading", { name: detail.label, exact: true }),
  ).toBeVisible();
  await expect(page.locator(".prepared-content")).toContainText("critical hit");
  await page.goBack();
  await expect(page).toHaveURL(`/records/${record}`);
  await expect(
    page.getByRole("heading", { name: "Rapier", exact: true }),
  ).toBeVisible();
  await expect(page.locator('a[href="/traits/trait:deadly-d8"]').first()).toBeVisible();
});

test("missing definitions and exact parameter variants stay distinct", async ({
  page,
  request,
}) => {
  const amp = await definition(request, "trait:amp");
  expect(amp.content).toBeNull();
  await page.goto("/traits/trait:amp");
  await expect(
    page
      .getByRole("alert")
      .filter({ hasText: "No definition is available in this reference collection." }),
  ).toBeVisible();
  const unknown = await definition(request, "trait:atlas-unknown-trait");
  expect(unknown.content).toBeNull();
  await page.goto("/traits/trait:atlas-unknown-trait");
  await expect(
    page.getByRole("heading", { name: "atlas-unknown-trait" }),
  ).toBeVisible();
  const d8 = await definition(request, "trait:deadly-d8");
  const d12 = await definition(request, "trait:deadly-d12");
  expect(d8.trait_key).not.toBe(d12.trait_key);
  expect(d8.content).not.toBeNull();
  expect(d12.content).not.toBeNull();
  for (const detail of [d8, d12]) {
    await page.goto(`/traits/${detail.trait_key}`);
    await expect(
      page.getByRole("heading", { name: detail.label, exact: true }),
    ).toBeVisible();
    await expect(page.locator(".prepared-content")).toContainText("critical hit");
  }
});

test("definition references retain authored labels and checked record links", async ({
  page,
  request,
}, testInfo) => {
  const detail = await definition(request, "trait:throneglass");
  expect(
    detail.content?.references.filter((reference) => reference.target).length,
  ).toBe(2);
  await page.goto("/traits/trait:throneglass");
  const content = page.locator(".prepared-content");
  for (const label of ["Stupefied 1", "Stupefied 2"]) {
    await expect(
      content.getByRole("link", { name: label, exact: true }),
    ).toHaveAttribute("href", /\/records\/conditionitems:/);
  }
  await page.getByRole("tab", { name: "References", exact: true }).click();
  const references = page.getByRole("region", {
    name: "Trait references",
    exact: true,
  });
  await expect(references.getByRole("link").first()).toBeVisible();
  await expect(
    page.getByRole("tab", { name: "References", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
  const screenshot = testInfo.outputPath("trait-throneglass.png");
  await page.screenshot({ path: screenshot, fullPage: true, animations: "disabled" });
  await testInfo.attach("Throneglass trait and references", {
    path: screenshot,
    contentType: "image/png",
  });
  await content.getByRole("link", { name: "Stupefied 1", exact: true }).click();
  await expect(page).toHaveURL(/\/records\/conditionitems:/);
  await expect(
    page.getByRole("heading", { name: "Stupefied", exact: true }),
  ).toBeVisible();
});

test("trait membership opens the exact embedded source", async ({
  page,
  request,
}, testInfo) => {
  const detail = await definition(request, "trait:agile");
  const members = await memberships(request, detail);
  const member = members.rows.find(
    (row) => row.navigation.embedded_path.length > 0 && row.source_label != null,
  );
  expect(
    member,
    "Full corpus agile membership must include an embedded source",
  ).toBeDefined();
  await page.goto("/traits/trait:agile");
  const region = page.getByRole("region", {
    name: "Records with this trait",
    exact: true,
  });
  const destination = recordPath(member!.navigation);
  const link = region.locator(`a[href=${JSON.stringify(destination)}]`).first();
  await expect(link).toBeVisible();
  await expect(link).toHaveText(`${member!.source_label} · ${member!.record.title}`);
  await link.scrollIntoViewIfNeeded();
  const screenshot = testInfo.outputPath("trait-named-embedded-member.png");
  await page.screenshot({ path: screenshot, animations: "disabled" });
  await testInfo.attach("Named embedded trait source", {
    path: screenshot,
    contentType: "image/png",
  });
  await link.click();
  await expect
    .poll(() => JSON.parse(new URL(page.url()).searchParams.get("selection") ?? "null"))
    .toEqual(member!.navigation);
  await expect(page.locator('a[href="/traits/trait:agile"]').first()).toBeVisible();
});

for (const profile of ["embeddinggemma", "bge-base"]) {
  test(`stored trait Similar uses ${profile} and reports bounded coverage`, async ({
    page,
    request,
  }) => {
    const readiness = await (await request.get("/api/readiness")).json();
    expect(
      readiness.search_profiles.some(
        (entry: { id: string; stored_vectors: boolean }) =>
          entry.id === profile && entry.stored_vectors,
      ),
      `Preview must contain stored ${profile} vectors`,
    ).toBeTruthy();
    await page.goto(`/traits/trait:agile?search_profile=${profile}`);
    await expect(
      page.getByRole("heading", { name: "Agile", exact: true }),
    ).toBeVisible();
    const response = page.waitForResponse(
      (response) =>
        response.url().endsWith("/api/traits/records") &&
        response.request().postDataJSON().kind === "similar",
    );
    await page.getByRole("tab", { name: "Similar", exact: true }).click();
    const result = await response;
    expect(result.ok(), await result.text()).toBeTruthy();
    const data = (await result.json()) as TraitRecordsPage;
    expect(data.search_profile).toBe(profile);
    expect(data.coverage).not.toBeNull();
    expect(data.rows.length).toBeGreaterThan(0);
    const region = page.getByRole("region", { name: "Similar records", exact: true });
    await expect(region.getByRole("link").first()).toBeVisible();
    if (!data.coverage!.exhaustive)
      await expect(region.getByText("Top matches", { exact: true })).toBeVisible();
    await expect(region.getByRole("link").first()).toHaveAttribute(
      "href",
      /^\/records\//,
    );
  });
}

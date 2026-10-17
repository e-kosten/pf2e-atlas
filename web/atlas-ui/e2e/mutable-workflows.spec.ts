import { expect, test } from "@playwright/test";

test("committed edits keep refresh feedback compact and Retry only reloads", async ({
  page,
}) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Refresh recovery ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const detailPath = `/api/encounters/${encounter.encounter_key}`;
  const added = await page.request.post(`${detailPath}/participants/manual`, {
    data: {
      encounter_ref: encounter.encounter_key,
      display_name: "Recovery Hero",
      max_hp: 30,
      current_hp: 30,
    },
  });
  expect(added.ok()).toBeTruthy();

  try {
    await page.goto(`/encounters/${encounter.slug}`);
    await expect(page.getByRole("textbox", { name: "HP", exact: true })).toHaveValue(
      "30",
    );
    const workspace = page.locator(".encounter-detail-view .workspace-grid");
    const baseline = await workspace.boundingBox();
    expect(baseline).not.toBeNull();
    let writes = 0;
    page.on("request", (request) => {
      if (
        request.method() === "PATCH" &&
        new URL(request.url()).pathname.startsWith(`${detailPath}/participants/`)
      ) {
        writes += 1;
      }
    });
    const abortDetail = (url: URL) =>
      url.pathname === detailPath ||
      url.pathname === `/api/encounters/${encounter.slug}`;
    await page.route(abortDetail, (route) => route.abort("failed"));
    await page.getByRole("textbox", { name: "HP", exact: true }).fill("25");
    await page.getByRole("button", { name: "Set", exact: true }).first().click();
    const feedback = page.getByRole("alert").filter({
      hasText: "Saved, but the encounter could not refresh",
    });
    await expect(feedback).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Add PC", exact: true }),
    ).toBeDisabled();
    await expect
      .poll(async () => (await feedback.boundingBox())?.height)
      .toBeLessThan(160);
    await expect
      .poll(async () => (await workspace.boundingBox())?.height)
      .toBeGreaterThan(baseline!.height * 0.7);
    expect(writes).toBe(1);

    await page.unroute(abortDetail);
    await page.getByRole("button", { name: "Retry encounter refresh" }).click();
    await expect(feedback).toHaveCount(0);
    await expect(
      page.getByRole("textbox", { name: "Recovery Hero current HP" }),
    ).toHaveValue("25");
    await expect(
      page.getByRole("button", { name: "Add PC", exact: true }),
    ).toBeEnabled();
    expect(writes).toBe(1);
  } finally {
    await page.request.delete(detailPath);
  }
});

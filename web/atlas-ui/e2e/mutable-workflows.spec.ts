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
      display_name: "Recovery Hero",
      max_hp: 30,
      current_hp: 30,
    },
  });
  expect(added.ok()).toBeTruthy();

  try {
    await page.goto(`/encounters/${encounter.slug}`);
    const summary = page.getByLabel("HP summary", { exact: true });
    await expect(summary).toHaveText("30/30");
    const workspace = page.locator(".encounter-detail-view .workspace-grid");
    const baseline = await workspace.boundingBox();
    expect(baseline).not.toBeNull();
    let writes = 0;
    page.on("request", (request) => {
      if (
        request.method() === "POST" &&
        new URL(request.url()).pathname.startsWith(`${detailPath}/participants/`) &&
        new URL(request.url()).pathname.endsWith("/hp")
      ) {
        writes += 1;
      }
    });
    const abortDetail = (url: URL) =>
      url.pathname === detailPath ||
      url.pathname === `/api/encounters/${encounter.slug}`;
    await page.route(abortDetail, (route) => route.abort("failed"));
    await page.getByRole("button", { name: "Edit Current HP", exact: true }).click();
    const input = page.getByRole("textbox", { name: "Current HP", exact: true });
    await input.fill("25");
    await input.press("Enter");
    const feedback = page.getByRole("status").filter({
      hasText: "Saved, but the encounter could not refresh",
    });
    await expect(feedback).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Add participants", exact: true }),
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
    await expect(summary).toHaveText("25/30");
    await expect(
      page.getByRole("button", { name: "Edit Current HP", exact: true }),
    ).toHaveText("25");
    await expect(
      page.getByRole("button", { name: "Add participants", exact: true }),
    ).toBeEnabled();
    expect(writes).toBe(1);
  } finally {
    await page.request.delete(detailPath);
  }
});

test("a character can be deleted from the list after confirmation", async ({
  page,
  request,
}) => {
  const name = `Character deletion ${Date.now()}`;
  const created = await request.post("/api/characters", {
    data: { name, request_id: crypto.randomUUID() },
  });
  expect(created.ok()).toBe(true);
  const { character_key } = await created.json();
  const path = `/api/characters/${character_key}`;
  try {
    await page.goto("/characters");
    const row = page
      .getByRole("row")
      .filter({ has: page.getByRole("link", { name, exact: true }) });
    const remove = row.getByRole("button", { name: `Delete ${name}`, exact: true });
    const dialog = page.getByRole("dialog", { name: `Delete ${name}?` });
    await remove.press("Enter");
    const cancel = dialog.getByRole("button", { name: "Cancel" });
    await expect(cancel).toBeFocused();
    await cancel.press("Enter");
    await expect(remove).toBeFocused();
    await expect(page).toHaveURL(/\/characters\/?$/);

    await page.setViewportSize({ width: 390, height: 740 });
    await remove.click();
    const beforeEdit = await request.get(path);
    const { character_revision } = await beforeEdit.json();
    const edited = await request.patch(path, {
      data: { expected_revision: character_revision, name, level: 8 },
    });
    expect(edited.ok()).toBe(true);
    const confirm = dialog.getByRole("button", { name: "Delete character" });
    await confirm.click();
    await expect(dialog.getByRole("alert")).toContainText("This character changed.");
    await cancel.click();
    await expect(row).toContainText("Level 8");
    await remove.click();
    await confirm.click();
    await expect(dialog).toBeHidden();
    await expect(row).toHaveCount(0);
    await expect(page).toHaveURL(/\/characters\/?$/);
    const index = await request.get("/api/characters");
    expect((await index.json()).characters).not.toContainEqual(
      expect.objectContaining({ character_key }),
    );
  } finally {
    const current = await request.get(path);
    const character = current.ok() && (await current.json());
    if (character) {
      await request.delete(path, {
        data: { expected_revision: character.character_revision },
      });
    }
  }
});

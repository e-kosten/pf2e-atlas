import { encodeFilterUrl } from "../../src/shared/filters/filterUrl";
import { expect, test } from "./fixtures";
import { creatureFilter, landedSearch, noPageOverflow, sentinel } from "./reading";

test("four primary destinations remain readable and usable at narrow widths", async ({
  page,
}) => {
  await page.goto("/encounters");
  const views = page.getByLabel("Atlas views").getByRole("link");
  await expect(views).toHaveText(["Search", "Lists", "Encounters", "Characters"]);
  for (const width of [390, 320]) {
    await page.setViewportSize({ width, height: 740 });
    for (const enlarged of [false, true]) {
      const style = enlarged
        ? await page.addStyleTag({
            content: '[aria-label="Atlas views"] a { font-size: 24px !important; }',
          })
        : null;
      await noPageOverflow(page);
      // Hit-test both ends of each rendered label, including when the menu reflows.
      // A clipped link can still have an in-viewport bounding box.
      for (const view of await views.all()) {
        await expect(view).toBeInViewport();
        expect(
          await view.evaluate((link) => {
            const label = document.createRange();
            label.selectNodeContents(link);
            return Array.from(label.getClientRects()).every((rect) =>
              [rect.left + 1, rect.right - 1].every((x) =>
                link.contains(document.elementFromPoint(x, rect.top + rect.height / 2)),
              ),
            );
          }),
        ).toBe(true);
      }
      await style?.evaluate((node) => node.remove());
    }
  }
  const characters = page.getByRole("link", { name: "Characters", exact: true });
  await expect(characters).toHaveAttribute("href", "/characters");
  await characters.press("Enter");
  await expect(page).toHaveURL(/\/characters\/?$/);
  const allCharacters = page.getByRole("link", { name: "All characters", exact: true });
  await allCharacters.hover();
  await expect(allCharacters).toHaveAttribute("aria-current", "page");
  await expect(allCharacters).toHaveCSS("text-decoration-line", "none");
  await expect(allCharacters).not.toHaveCSS(
    "border-block-end-color",
    "rgba(0, 0, 0, 0)",
  );
  const parties = page.getByRole("link", { name: "Parties", exact: true });
  await parties.hover();
  await expect(parties).toHaveCSS("text-decoration-line", "underline");
  const [popup] = await Promise.all([
    page.context().waitForEvent("page"),
    parties.click({ button: "middle" }),
  ]);
  try {
    await expect(popup).toHaveURL(/\/parties\/?$/);
    await expect(
      popup
        .getByRole("navigation", { name: "Character pages" })
        .locator('a[aria-current="page"]'),
    ).toHaveText("Parties");
    await expect(page).toHaveURL(/\/characters\/?$/);
  } finally {
    await popup.close();
  }
});

test("an open reference and unfinished query survive crossing the reader breakpoint", async ({
  page,
}) => {
  await page.goto(landedSearch);
  const input = page.getByRole("combobox", { name: "Search records" });
  const profileNotice = page
    .getByRole("alert")
    .filter({ hasText: "Text search is unavailable" });
  await input.fill("unfinished search intent");
  await expect(profileNotice).toBeVisible();
  await expect(page.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
  await page.setViewportSize({ width: 390, height: 664 });
  await expect(page.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
  await expect(profileNotice).toBeHidden();
  await page.getByRole("button", { name: "Results", exact: true }).click();
  await expect(profileNotice).toBeVisible();
  await expect(input).toHaveValue("unfinished search intent");
  await expect(page.locator(`[data-record-key="${sentinel}"]`)).toBeFocused();
  await page.setViewportSize({ width: 1100, height: 700 });
  await expect(input).toHaveValue("unfinished search intent");
  await expect(page.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
  await noPageOverflow(page);
});

test("320 CSS pixel reflow keeps keyboard filters, results and the reader operable", async ({
  page,
}) => {
  // Effective-layout-width coverage for zoom/reflow. This is not physical-device
  // or browser text-enlargement evidence; those require the documented visual pass.
  await page.setViewportSize({ width: 320, height: 640 });
  await page.goto(`/search?filter=${creatureFilter}`);
  const filters = page.getByRole("button", { name: "Filters (1)", exact: true });
  const kind = page.getByRole("button", { name: "Edit Kind filter", exact: true });
  await page.keyboard.press("Tab");
  // Walk native tab order rather than proving only programmatic focusability.
  for (
    let step = 0;
    step < 20 && !(await filters.evaluate((node) => node === document.activeElement));
    step++
  ) {
    await page.keyboard.press("Tab");
  }
  await expect(filters).toBeFocused();
  await expect(kind).toBeHidden();
  await page.keyboard.press("Enter");
  await expect(filters).toHaveAttribute("aria-expanded", "true");
  await page.keyboard.press("Tab");
  await expect(kind).toBeFocused();
  await page.keyboard.press("Enter");
  const drawer = page.getByRole("dialog", { name: "Kind", exact: true });
  await expect(drawer).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(kind).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(filters).toBeFocused();
  const row = page.locator(`[data-record-key="${sentinel}"]`);
  await row.press("Enter");
  await expect(page).toHaveURL(/\/search\/records\//);
  await expect(
    page.getByRole("heading", { name: "Atlas Sentinel", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Results", exact: true }).click();
  await expect(row).toBeFocused();
  await noPageOverflow(page);
});

test("missing query assets leave a repairable text search and usable filter browsing", async ({
  page,
}) => {
  await page.goto(`/search?filter=${creatureFilter}`);
  await expect(page.locator("[data-record-key]").first()).toBeVisible();
  await page.getByRole("combobox", { name: "Search records" }).fill("lantern sentinel");
  await page.getByRole("combobox", { name: "Search records" }).press("Enter");
  await expect(
    page.getByRole("alert").filter({ hasText: "Text search is unavailable" }),
  ).toBeVisible();
  await expect(page.getByRole("combobox", { name: "Search records" })).toHaveValue(
    "lantern sentinel",
  );
  await page.getByRole("button", { name: "Clear filters", exact: true }).click();
  await expect(page.getByRole("combobox", { name: "Search records" })).toHaveValue(
    "lantern sentinel",
  );
  await page.getByRole("combobox", { name: "Search records" }).fill("");
  await page.getByRole("combobox", { name: "Search records" }).press("Enter");
  await page.getByRole("button", { name: "Edit Kind filter", exact: true }).click();
  await page.getByRole("button", { name: /^Kind: Creature, neutral;/ }).click();
  await page.keyboard.press("Escape");
  await expect(page.locator("[data-record-key]").first()).toBeVisible();
});
test("short landscape filter popovers keep their footer reachable by pointer", async ({
  page,
}) => {
  await page.setViewportSize({ width: 844, height: 390 });
  await page.goto("/search");
  await page.getByRole("button", { name: "Filters", exact: true }).click();
  await page.getByRole("button", { name: "Edit Traits filter", exact: true }).click();
  const menu = page.getByRole("dialog", { name: "Traits choices", exact: true });
  const footer = menu.getByRole("checkbox", { name: "Show all values" });
  // The picker focuses its search field after the opening motion finishes.
  await expect(menu.getByLabel("Search traits choices", { exact: true })).toBeFocused();
  await expect(footer).toBeVisible();
  await expect
    .poll(async () => {
      const box = await menu.boundingBox();
      return !!box && box.y >= 0 && box.y + box.height <= 390;
    })
    .toBe(true);
  // Click at the current coordinates, without Playwright scrolling it into view.
  const box = await footer.boundingBox();
  expect(box).not.toBeNull();
  await page.mouse.click(box!.x + box!.width / 2, box!.y + box!.height / 2);
  await expect(footer).toBeChecked();
});

test("replacing the reference library restarts only the open vocabulary page", async ({
  page,
}) => {
  let changed = false;
  let continued = false;
  let recovered = false;
  await page.route("**/api/filters/values", async (route) => {
    const request = route.request().postDataJSON();
    if (request.kind !== "vocabulary" || request.field_id !== "traits")
      return route.continue();
    const original = await route.fetch({
      postData: { ...request, expected_artifact_generation: null },
    });
    const result = await original.json();
    if (changed && request.offset > 0) {
      continued = request.expected_artifact_generation === result.artifact_generation;
      await route.fulfill({
        response: original,
        status: 409,
        json: {
          code: "artifact_generation_changed",
          message: "The reference library changed.",
          recoverable_action: "refresh_results",
        },
      });
      return;
    }
    recovered ||=
      changed &&
      request.offset === 0 &&
      !request.expected_artifact_generation &&
      request.text === "test" &&
      request.selected_values.includes("fire");
    await route.fulfill({
      response: original,
      json: {
        ...result,
        artifact_generation: changed
          ? "replacement-generation"
          : result.artifact_generation,
        values: {
          ...result.values,
          total_values: 60,
          options: [
            { value: "fire", label: "Fire" },
            {
              value: "test",
              label: changed ? "Test after update" : "Test before update",
            },
          ],
        },
      },
    });
  });
  const filter = encodeFilterUrl({
    kind: "set_match",
    field: "traits",
    op: "includes_any",
    clause_id: "traits",
    values: ["fire"],
  });
  await Promise.all([
    page.waitForResponse((response) => response.url().endsWith("/api/search/page")),
    page.goto(`/search?filter=${encodeURIComponent(filter)}`),
  ]);
  await page.getByRole("button", { name: "Edit Traits filter", exact: true }).click();
  const menu = page.getByRole("dialog", { name: "Traits choices" });
  await menu.getByRole("checkbox", { name: "Show all values" }).check();
  const input = menu.getByLabel("Search traits choices", { exact: true });
  await Promise.all([
    page.waitForResponse(
      (r) =>
        r.url().endsWith("/api/filters/values") &&
        r.request().postDataJSON().text === "test",
    ),
    input.fill("test"),
  ]);
  changed = true;
  await menu.getByRole("button", { name: "More values" }).click();
  await expect(
    menu.getByRole("button", { name: /^Traits: Test after update, neutral/ }),
  ).toBeVisible();
  await expect(
    menu.getByRole("button", { name: /^Traits: Fire, included/ }),
  ).toBeVisible();
  await expect(menu.getByRole("button", { name: "Previous values" })).toBeDisabled();
  await expect(input).toHaveValue("test");
  expect(continued).toBe(true);
  expect(recovered).toBe(true);
  await test.info().attach("Recovered vocabulary page", {
    body: await menu.screenshot(),
    contentType: "image/png",
  });
});

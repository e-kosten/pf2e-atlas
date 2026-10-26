import { expect, test } from "@playwright/test";
import { encodeFilterUrl } from "../src/shared/filters/filterUrl";

test("Sort rules and seeded Random remain usable at 390px", async ({
  page,
  context,
}) => {
  await page.setViewportSize({ width: 390, height: 1000 });
  await page.goto(
    "/search?filter=" +
      encodeURIComponent(
        encodeFilterUrl({ kind: "in", field: "kind", values: ["creature"] }),
      ),
  );
  await expect(page.locator("[data-record-key]").first()).toBeVisible();
  await page.getByRole("button", { name: /^Sort:/ }).click();
  const dialog = page.getByRole("dialog", { name: "Sort results" });
  const choose = async (name: string, option: string) => {
    await dialog.getByRole("combobox", { name, exact: true }).click();
    await page
      .getByText(option, { exact: true })
      .filter({ visible: true })
      .last()
      .click();
  };
  await choose("Sort field 1", "Level / Rank");
  await choose("Sort direction 1", "High to low");
  await dialog.getByRole("button", { name: "Add another sort", exact: true }).click();
  await choose("Sort field 2", "Unit price");
  await dialog.getByRole("button", { name: "Move sort rule 2 up" }).click();
  await expect(page).toHaveURL(
    (url) => url.searchParams.getAll("sort").join(",") === "unit_price_asc,level_desc",
  );
  const bounds = await dialog.boundingBox();
  expect(bounds).not.toBeNull();
  expect(bounds!.x).toBeGreaterThanOrEqual(0);
  expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(390);
  expect(bounds!.y).toBeGreaterThanOrEqual(0);
  expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(1000);
  await choose("Sort mode", "Random");
  const seed = dialog.getByRole("textbox", { name: "Random seed" });
  await expect(seed).toHaveValue(/^\d+$/);
  const original = new URL(page.url()).searchParams.get("seed");
  await seed.fill(original === "0" ? "1" : "0");
  await seed.press("Escape");
  await expect(dialog).not.toBeVisible();
  await expect(
    page.getByRole("button", { name: "Sort: Random", exact: true }),
  ).toBeFocused();
  expect(new URL(page.url()).searchParams.get("seed")).toBe(original);
  await page.getByRole("button", { name: "Sort: Random", exact: true }).click();
  await expect(seed).toHaveValue(original!);
  await seed.fill("0");
  expect(new URL(page.url()).searchParams.get("seed")).toBe(original);
  await seed.press("Enter");
  await expect(page).toHaveURL((url) => url.searchParams.get("seed") === "0");
  await context.grantPermissions(["clipboard-read", "clipboard-write"], {
    origin: new URL(page.url()).origin,
  });
  const copy = dialog.getByRole("button", { name: "Copy seed", exact: true });
  await copy.click();
  await expect
    .poll(() => page.evaluate(() => navigator.clipboard.readText()))
    .toBe("0");
  await expect(
    page.getByText("Copied", { exact: true }).filter({ visible: true }),
  ).toBeVisible();
  await seed.fill("43");
  await copy.click();
  await expect(page).toHaveURL((url) => url.searchParams.get("seed") === "43");
  await expect
    .poll(() => page.evaluate(() => navigator.clipboard.readText()))
    .toBe("43");
  await seed.fill("0");
  await seed.press("Enter");
  const shuffle = dialog.getByRole("button", { name: "Shuffle", exact: true });
  const seedBounds = await dialog.locator(".result-sort__seed-input").boundingBox();
  const shuffleBounds = await shuffle.boundingBox();
  expect(seedBounds).not.toBeNull();
  expect(shuffleBounds).not.toBeNull();
  expect(Math.abs(seedBounds!.y - shuffleBounds!.y)).toBeLessThanOrEqual(1);
  expect(Math.abs(seedBounds!.height - shuffleBounds!.height)).toBeLessThanOrEqual(1);
  const copyBounds = await dialog
    .getByRole("button", { name: "Copy seed", exact: true })
    .boundingBox();
  expect(copyBounds).not.toBeNull();
  expect(copyBounds!.x).toBeGreaterThanOrEqual(seedBounds!.x + seedBounds!.width / 2);
  expect(copyBounds!.x + copyBounds!.width).toBeLessThanOrEqual(
    seedBounds!.x + seedBounds!.width,
  );
  expect(copyBounds!.y).toBeGreaterThanOrEqual(seedBounds!.y);
  expect(copyBounds!.y + copyBounds!.height).toBeLessThanOrEqual(
    seedBounds!.y + seedBounds!.height,
  );
  expect(seedBounds!.x + seedBounds!.width).toBeLessThanOrEqual(shuffleBounds!.x);
  expect(shuffleBounds!.x + shuffleBounds!.width).toBeLessThanOrEqual(390);
  await shuffle.click();
  await expect(page).toHaveURL(
    (url) =>
      url.searchParams.get("sort") === "random" && url.searchParams.get("seed") !== "0",
  );
  const shuffled = new URL(page.url()).searchParams.get("seed");
  await page.reload();
  await expect(
    page.getByRole("button", { name: "Sort: Random", exact: true }),
  ).toBeVisible();
  expect(new URL(page.url()).searchParams.get("seed")).toBe(shuffled);
});

for (const width of [390, 320]) {
  test(`sorting and complete page navigation fit within the result pane at ${width}px`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 1000 });
    await page.goto(
      "/search?sort=unit_price_asc&filter=" +
        encodeURIComponent(
          encodeFilterUrl({
            kind: "in",
            clause_id: "kind",
            field: "kind",
            values: ["creature"],
          }),
        ),
    );
    const results = page.getByRole("region", { name: "Search results", exact: true });
    await expect(results.locator("[data-record-key]").first()).toBeVisible();
    const pane = await results.boundingBox();
    expect(pane).not.toBeNull();
    // Check actual control shells against the pane, not just document overflow.
    for (const control of await results.locator(".pager > *").all()) {
      const box = await control.boundingBox();
      expect(box).not.toBeNull();
      expect(box!.x).toBeGreaterThanOrEqual(pane!.x);
      expect(box!.x + box!.width).toBeLessThanOrEqual(pane!.x + pane!.width);
    }
    const navigation = results.getByRole("group", { name: "Result pages" });
    const previous = await navigation
      .getByRole("button", { name: "Previous page" })
      .boundingBox();
    const next = await navigation
      .getByRole("button", { name: "Next page" })
      .boundingBox();
    expect(previous).not.toBeNull();
    expect(next).not.toBeNull();
    expect(Math.abs(previous!.y - next!.y)).toBeLessThanOrEqual(1);
    await navigation.getByRole("button", { name: "Next page" }).click();
    await expect(navigation.getByRole("textbox", { name: "Current page" })).toHaveValue(
      "2",
    );
    await expect(results.locator("[data-record-key]").first()).toBeVisible();
  });
}

// These checks use the caller's real source artifact and application service.
for (const width of [1440, 1100, 390]) {
  for (const similar of [false, true]) {
    test(`${similar ? "Similar" : "browse"} uses a usable result browser at ${width}px`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height: 1000 });
      await page.goto(
        similar
          ? "/search?similar=equipment-srd%3A7jolv0cuttvjI1JD"
          : "/search?filter=" +
              encodeURIComponent(
                encodeFilterUrl({
                  kind: "in",
                  clause_id: "kind",
                  field: "kind",
                  values: ["creature"],
                }),
              ),
      );
      const results = page.getByRole("region", { name: "Search results", exact: true });
      const row = page.locator("[data-record-key]").first();
      await expect(row).toBeVisible();
      const search = page.getByRole("searchbox", {
        name: similar ? "Similar search" : "Search records",
      });
      await expect(search).toBeVisible();
      if (similar) await expect(search).toHaveAttribute("readonly");
      const inputBox = await page.locator(".reference-search-entry").boundingBox();
      expect(inputBox).not.toBeNull();
      const controlHeightBox = inputBox;
      const primaryControls = page.locator(
        ".search-model-select, .filter-toolbar .common-choice-trigger, .filter-panel-actions button",
      );
      for (const control of await primaryControls.all()) {
        const controlBox = await control.boundingBox();
        expect(controlBox).not.toBeNull();
        expect(
          Math.abs(controlBox!.height - controlHeightBox!.height),
        ).toBeLessThanOrEqual(1);
      }
      const coverage = page.locator(".result-pane-header > span").first();
      await expect(coverage).toHaveText(similar ? /Top matches/ : /\d[\d,]* records/);
      if (similar) {
        await expect(search).toHaveValue("Wovenwood Shield (Minor)");
        await search.focus();
        await search.press("ControlOrMeta+A");
        expect(
          await search.evaluate((element: HTMLInputElement) =>
            element.value.slice(element.selectionStart ?? 0, element.selectionEnd ?? 0),
          ),
        ).toBe("Wovenwood Shield (Minor)");
      }
      for (const control of [
        results.getByRole("combobox", { name: "Records per page" }),
        results.getByRole("textbox", { name: "Current page" }),
        results.getByRole("button", { name: "Next page" }),
      ]) {
        const box = await control.boundingBox();
        expect(box).not.toBeNull();
        expect(box!.x).toBeGreaterThanOrEqual(0);
        expect(box!.x + box!.width).toBeLessThanOrEqual(width);
      }
      await row.click();
      const reader = page.getByRole("region", { name: "Record preview", exact: true });
      await expect(reader.locator(".record-reference__title")).toBeVisible();
      const box = await reader.boundingBox();
      expect(box!.width).toBeGreaterThan(width === 390 ? 340 : 600);
      if (width === 390) {
        await expect(search).toBeHidden();
        await reader.getByRole("button", { name: "Results", exact: true }).click();
        await expect(row).toBeFocused();
        await expect(row).toBeVisible();
      } else {
        const resultBox = await results.boundingBox();
        expect(box!.x).toBeGreaterThanOrEqual(resultBox!.x + resultBox!.width);
      }
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
      ).toBe(true);
      if (similar) {
        const geometry = async () =>
          Promise.all([
            page.locator(".reference-search-entry").boundingBox(),
            page
              .getByRole("combobox", { name: "Search model" })
              .locator("..")
              .boundingBox(),
            page
              .getByRole("combobox", { name: "Search ranking" })
              .locator("..")
              .boundingBox(),
          ]);
        const before = await geometry();
        const nativeInput = await search.elementHandle();
        await page.getByRole("button", { name: "Clear similar criterion" }).click();
        const normalSearch = page.getByRole("searchbox", { name: "Search records" });
        await expect(normalSearch).toBeFocused();
        expect(
          await normalSearch.evaluate(
            (element, previous) => element === previous,
            nativeInput,
          ),
        ).toBe(true);
        expect(new URL(page.url()).searchParams.has("similar")).toBe(false);
        const after = await geometry();
        for (let index = 0; index < before.length; index++) {
          for (const dimension of ["x", "y", "width", "height"] as const) {
            expect(
              Math.abs(before[index]![dimension] - after[index]![dimension]),
            ).toBeLessThanOrEqual(1);
          }
        }
      }
    });
  }
}

test("Back keeps page, selected result and nonzero list/reader positions", async ({
  page,
}) => {
  await page.goto("/search?q=fire&mode=text_search");
  await page.locator("[data-record-key]").first().waitFor();
  await page.getByRole("button", { name: "Next page", exact: true }).click();
  await expect(page.getByRole("textbox", { name: "Current page" })).toHaveValue("2");
  const row = page.locator("[data-record-key]").nth(4);
  await row.click();
  await expect(
    page.locator(".reference-reader__scroll .record-reference__title"),
  ).toBeVisible();
  await page.locator(".reference-results__scroll").evaluate((e) => (e.scrollTop = 200));
  const selected = await row.getAttribute("data-record-key");
  await page.getByRole("link", { name: "Open full page", exact: true }).click();
  await expect(page).toHaveURL(/\/records\//);
  await page.goBack();
  await expect(page.getByRole("textbox", { name: "Current page" })).toHaveValue("2");
  await expect(page.locator("[data-record-key][aria-current=true]")).toHaveAttribute(
    "data-record-key",
    selected!,
  );
  await expect
    .poll(() => page.locator(".reference-results__scroll").evaluate((e) => e.scrollTop))
    .toBe(200);
  // Use a long creature reader to establish actual nonzero reader continuity.
  const search = page.getByRole("searchbox", { name: "Search records", exact: true });
  await search.fill("Imeckus Stroon");
  await search.press("Enter");
  await expect(page.getByRole("textbox", { name: "Current page" })).toHaveValue("1");
  await expect(page.locator(".reference-results__scroll")).toHaveAttribute(
    "aria-busy",
    "false",
  );
  await page.locator("[data-record-key]").filter({ hasText: "Imeckus Stroon" }).click();
  await expect(
    page.getByRole("heading", { name: "Imeckus Stroon", exact: true }),
  ).toBeVisible();
  const scroll = page.locator(".reference-reader__scroll");
  await scroll.evaluate((e) => (e.scrollTop = 400));
  await expect.poll(() => scroll.evaluate((e) => e.scrollTop)).toBe(400);
  await page.getByRole("link", { name: "Open full page", exact: true }).click();
  await page.goBack();
  await expect.poll(() => scroll.evaluate((e) => e.scrollTop)).toBe(400);
  // Back remounts the browser without its original preview-trigger ref.
  await page.setViewportSize({ width: 390, height: 1000 });
  await page.getByRole("button", { name: "Results", exact: true }).click();
  await expect(page.locator('[data-record-key][aria-current="true"]')).toBeFocused();
});

test("reader matches keep result rows stable and navigate without scrolling app chrome", async ({
  page,
}) => {
  await page.goto("/search?q=fire&search_profile=bge-base");
  const row = page.locator(".reference-result").filter({
    has: page.locator('[data-record-key="pathfinder-monster-core:MX2cLbODuo4gECPJ"]'),
  });
  await expect(row).toBeVisible();
  // Offscreen rows use an estimated intrinsic height until actionability renders them.
  await row.locator("[data-record-key]").click({ trial: true });
  const before = await row.boundingBox();
  await row.locator("[data-record-key]").click();
  const preview = page.getByRole("region", { name: "Record preview", exact: true });
  const matches = preview.getByRole("region", { name: "Search matches", exact: true });
  await expect(
    matches.getByRole("link", { name: "Breath Fire", exact: true }),
  ).toHaveCount(1);
  await expect(page.getByRole("button", { name: /matches in/ })).toHaveCount(0);
  expect((await row.boundingBox())!.height).toBe(before!.height);
  await expect
    .poll(() => page.locator(".reference-reader__scroll").evaluate((e) => e.scrollTop))
    .toBe(0);
  await matches.getByRole("link", { name: "Fire Mote", exact: true }).click();
  await expect(page.locator(".reference-landing-highlight")).toContainText("Fire Mote");
  expect(await page.evaluate(() => scrollY)).toBe(0);
  await expect(
    preview.getByRole("heading", { name: "Firewyrm", exact: true }),
  ).toHaveCount(1);
  await preview
    .getByRole("button", { name: "Return to selection", exact: true })
    .click();
  await matches.getByRole("link", { name: "Breath Fire", exact: true }).click();
  await expect(preview.getByRole("heading", { name: /^Breath Fire/ })).toBeVisible();
  await expect(page.locator(".reference-landing-highlight")).toContainText(
    "The firewyrm breathes",
  );
  await expect(page.locator("[data-atlas-passage-target]")).toBeFocused();
  expect(await page.evaluate(() => scrollY)).toBe(0);
  await preview
    .getByRole("button", { name: "Return to selection", exact: true })
    .click();
  await expect(matches).toBeVisible();
});

test("long match excerpts stay within two lines at desktop and phone widths", async ({
  page,
}) => {
  await page.goto(
    "/search/records/sky-kings-tomb-bestiary:RVdRHveFJmeCscI6?search_profile=bge-base&q=wriggly+bug",
  );
  const matches = page.getByRole("region", { name: "Search matches", exact: true });
  const excerpts = matches.getByText(
    /^(The cave worm flings|Brood Variance The cave worms|When a juvenile cave worm)/,
  );
  await expect(excerpts).toHaveCount(3);
  for (const width of [1440, 390]) {
    await page.setViewportSize({ width, height: 1000 });
    await expect(excerpts).toHaveCount(3);
    for (const excerpt of await excerpts.all()) {
      await expect(excerpt).toBeVisible();
      await expect
        .poll(() =>
          excerpt.evaluate(
            (e) =>
              e.getBoundingClientRect().height /
              parseFloat(getComputedStyle(e).lineHeight),
          ),
        )
        .toBeLessThanOrEqual(2.01);
    }
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth - innerWidth),
    ).toBe(0);
  }
  await matches.getByRole("link", { name: "Brood Variance", exact: true }).click();
  const section = page.locator("[data-atlas-passage-target]");
  await expect(section).toHaveText("Brood Variance");
  await expect(section).toBeInViewport();
  await expect(section).toBeFocused();
  for (const width of [390, 320]) {
    await page.setViewportSize({ width, height: 844 });
    const controls = page
      .locator(".reference-reader__header")
      .locator("button:visible, a:visible");
    const boxes = await controls.evaluateAll((elements) =>
      elements.map((element) => {
        const { x, y, width, height } = element.getBoundingClientRect();
        return { x, y, width, height };
      }),
    );
    for (const [index, box] of boxes.entries()) {
      expect(box.x).toBeGreaterThanOrEqual(0);
      expect(box.x + box.width).toBeLessThanOrEqual(width);
      for (const other of boxes.slice(index + 1)) {
        const overlapX =
          Math.min(box.x + box.width, other.x + other.width) - Math.max(box.x, other.x);
        const overlapY =
          Math.min(box.y + box.height, other.y + other.height) -
          Math.max(box.y, other.y);
        expect(overlapX > 0.5 && overlapY > 0.5).toBe(false);
      }
    }
  }
  await page.getByRole("button", { name: "Return to selection", exact: true }).click();
  await matches.getByRole("link", { name: "Clumsy Leap", exact: true }).click();
  await expect(page.getByRole("heading", { name: /^Clumsy Leap/ })).toBeVisible();
  await expect(page.locator(".record-reference__content")).toContainText(
    "The cave worm flings its body",
  );
});

test("relationship scope stays tappable within a phone viewport and preserves exact selection", async ({
  browser,
  baseURL,
}) => {
  const context = await browser.newContext({
    baseURL,
    viewport: { width: 390, height: 844 },
    hasTouch: true,
    isMobile: true,
    colorScheme: "dark",
  });
  const page = await context.newPage();
  try {
    const constraint = {
      direction: "incoming",
      selection: {
        record_key: "conditionitems:dfCMdR4wnpbYNTix",
        embedded_path: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
      include_contents: true,
    };
    await page.goto(
      "/search?relationships=" +
        encodeURIComponent("1." + JSON.stringify([constraint])),
    );
    const criterion = page.getByRole("group", { name: "Relationship filter" });
    await expect(criterion).toContainText("Stunned");
    const trigger = criterion.getByRole("button", { name: /Links to Stunned/ });
    await expect(trigger).toHaveText("Links to Stunned");
    await trigger.tap();
    const popover = page
      .getByRole("tooltip")
      .filter({ hasText: "Look for links in matching records" });
    await expect(popover).toBeVisible();
    const main = popover.getByRole("radio", { name: "Main entry only" });
    const whole = popover.getByRole("radio", { name: "Whole entry" });
    for (const control of [popover, main, whole]) {
      const box = await control.boundingBox();
      expect(box).not.toBeNull();
      expect(box!.x).toBeGreaterThanOrEqual(0);
      expect(box!.y).toBeGreaterThanOrEqual(0);
      expect(box!.x + box!.width).toBeLessThanOrEqual(390);
      expect(box!.y + box!.height).toBeLessThanOrEqual(844);
    }
    await main.tap();
    await expect(trigger).toHaveText("Links to Stunned");
    await expect(trigger).toBeFocused();
    const restored = JSON.parse(
      new URL(page.url()).searchParams.get("relationships")!.slice(2),
    );
    expect(restored).toEqual([{ ...constraint, include_contents: false }]);
    await trigger.tap();
    await expect(main).toBeChecked();
    await page.keyboard.press("Escape");
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
    await expect(trigger).toBeFocused();
    await trigger.tap();
    await popover.getByRole("link", { name: "Open Stunned", exact: true }).tap();
    await expect(page).toHaveURL(
      (url) =>
        decodeURIComponent(url.pathname) === "/records/conditionitems:dfCMdR4wnpbYNTix",
    );
    expect(JSON.parse(new URL(page.url()).searchParams.get("selection")!)).toEqual(
      constraint.selection,
    );
    await expect(
      page.getByRole("heading", { name: "Stunned", exact: true }),
    ).toBeVisible();
  } finally {
    await context.close();
  }
});

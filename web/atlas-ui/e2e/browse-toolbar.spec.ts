import { expect, test } from "@playwright/test";
import { encodeFilterUrl } from "../src/shared/filters/filterUrl";

const seed = "equipment-srd:7jolv0cuttvjI1JD";
const similar = `/search?similar=${encodeURIComponent(seed)}`;

test("selection clear preserves criteria and full-page navigation never flashes in the browse toolbar", async ({
  page,
}) => {
  await page.goto(similar);
  const row = page.locator(".reference-results [data-record-key]").first();
  await expect(row).toBeVisible();
  const recordKey = await row.getAttribute("data-record-key");
  let release!: () => void;
  let requested!: () => void;
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  const pending = new Promise<void>((resolve) => {
    requested = resolve;
  });
  await page.route(
    (url) => decodeURIComponent(url.pathname) === `/api/records/${recordKey}`,
    async (route) => {
      const response = await route.fetch();
      requested();
      await gate;
      await route.fulfill({ response });
    },
  );
  const toolbar = page.locator(".reference-browser__toolbar");
  await toolbar.evaluate((host) => {
    const observer = new MutationObserver(() => {
      if (host.querySelector('a[aria-label="Open full page"]'))
        host.setAttribute("data-navigation-flash", "true");
    });
    observer.observe(host, { childList: true, subtree: true });
  });
  await row.click();
  try {
    await pending;
    await expect(
      toolbar.getByRole("button", { name: "Clear selection", exact: true }),
    ).toBeVisible();
    await expect(
      toolbar.getByRole("link", { name: "Open full page", exact: true }),
    ).toHaveCount(0);
  } finally {
    release();
  }
  await expect(
    page.locator(".reference-reader .record-reference__title"),
  ).toBeVisible();
  await expect(toolbar).not.toHaveAttribute("data-navigation-flash", "true");
  await expect(
    page
      .locator(".reference-reader")
      .getByRole("link", { name: "Open full page", exact: true }),
  ).toBeVisible();
  await toolbar
    .getByRole("button", { name: "Clear selection", exact: true })
    .press("Enter");
  await expect(page).toHaveURL(
    (url) => url.pathname === "/search" && url.searchParams.get("similar") === seed,
  );
  await expect(page.getByText("Select a record", { exact: true })).toBeVisible();
  await expect(row).toBeFocused();
  await expect(
    toolbar.getByRole("button", { name: "Clear selection", exact: true }),
  ).toHaveCount(0);
});

test("Similar entry and applied link chips reflow without changing search-control height", async ({
  page,
}) => {
  const filter = encodeFilterUrl({
    kind: "relationship",
    clause_id: "link-layout",
    relationship: {
      direction: "incoming",
      include_contents: true,
      selection: {
        record_key: "equipment-srd:e9Xiei2wduOEGI5r",
        embedded_path: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
    },
  });
  await page.goto(`${similar}&filter=${encodeURIComponent(filter)}`);
  const entry = page.locator(".reference-search-entry");
  await expect(
    entry.getByRole("link", { name: "Wovenwood Shield (Minor)", exact: true }),
  ).toBeVisible();
  const chip = page.getByRole("group", { name: "Relationship filter", exact: true });
  await expect(chip).toBeVisible();
  for (const width of [1440, 1024, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 844 });
    const geometry = await page.locator(".filter-toolbar").evaluate((host) => {
      const box = (selector: string) => {
        const { x, y, width, height, bottom } = host
          .querySelector(selector)!
          .getBoundingClientRect();
        return { x, y, width, height, bottom };
      };
      return {
        entry: box(".reference-search-entry"),
        run: box('button[aria-label="Run similar search"]'),
        settings: box('button[aria-label="Search settings"]'),
        tools: box(".filter-controls__tools"),
        chip: box(".relationship-filter"),
      };
    });
    expect(geometry.entry.height).toBeCloseTo(geometry.run.height, 0);
    expect(geometry.entry.height).toBeCloseTo(geometry.settings.height, 0);
    expect(geometry.entry.y).toBeCloseTo(geometry.settings.y, 0);
    expect(geometry.chip.y).toBeGreaterThanOrEqual(geometry.tools.bottom);
    expect(geometry.chip.x + geometry.chip.width).toBeLessThanOrEqual(width);
    await expect(
      chip.getByRole("button", { name: "Remove relationship filter", exact: true }),
    ).toBeVisible();
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1),
    ).toBe(true);
  }
  await chip.getByRole("button", { name: /Links to/ }).click();
  await expect(
    page.getByRole("link", { name: "Open Wovenwood Shield (Major)", exact: true }),
  ).toBeVisible();
});

test("selected filters keep trigger dimensions stable and restore focus when a chip is removed", async ({
  page,
}) => {
  const encoded =
    "3.NoIghiA0odIJZRAa3gOwCZVAYwE4CmYALgK6EgC6lca2CSqm9OAFmHmDsQXldTVABnJMU7xiIyAjQ4ANqQwEhAfTBoAnvS7ws0sBgwT4ANwL9KQA";
  await page.goto(`/search?search_profile=bge-base&q=cat&filter=${encoded}`);
  const filters = page.getByRole("group", { name: "Applied filters", exact: true });
  const creature = filters.getByRole("button", {
    name: "Remove included Creature from Kind",
    exact: true,
  });
  await expect(creature).toBeVisible();
  const baseline = await page
    .getByRole("button", { name: "Edit Kind filter", exact: true })
    .boundingBox();
  let spacing: { padding: string; gap: string } | undefined;
  for (const width of [1440, 1150, 1024, 992, 991, 768, 680, 560, 390, 320]) {
    await page.setViewportSize({ width, height: 844 });
    const trigger = await page
      .getByRole("button", { name: "Edit Kind filter", exact: true })
      .boundingBox();
    expect(trigger!.width).toBeCloseTo(baseline!.width, 0);
    expect(trigger!.height).toBeCloseTo(baseline!.height, 0);
    const layout = await page.locator(".filter-toolbar").evaluate((host) => {
      const style = getComputedStyle(host);
      const tools = host
        .querySelector(".filter-controls__tools")!
        .getBoundingClientRect();
      const selections = host
        .querySelector(".filter-controls__selections")!
        .getBoundingClientRect();
      return {
        spacing: { padding: style.padding, gap: style.gap },
        toolsBottom: tools.bottom,
        selectionsTop: selections.top,
      };
    });
    spacing ??= layout.spacing;
    expect(layout.spacing).toEqual(spacing);
    expect(layout.selectionsTop).toBeGreaterThanOrEqual(layout.toolsBottom);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1),
    ).toBe(true);
  }
  await creature.press("Enter");
  await expect(
    page.getByRole("button", { name: "Edit Kind filter", exact: true }),
  ).toBeFocused();
  await expect(creature).toHaveCount(0);
  await expect(
    filters.getByRole("button", {
      name: "Remove excluded Character from Kind",
      exact: true,
    }),
  ).toBeVisible();
});

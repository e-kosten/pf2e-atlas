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
  const entry = page.locator(".reference-search-entry--similar");
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
        entry: box(".reference-search-entry--similar"),
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
    page.getByRole("dialog", { name: "Edit link condition", exact: true }),
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
    await expect
      .poll(async () => {
        const trigger = await page
          .getByRole("button", { name: "Edit Kind filter", exact: true })
          .boundingBox();
        return trigger ? { width: trigger.width, height: trigger.height } : null;
      })
      .toEqual({ width: baseline!.width, height: baseline!.height });
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

test("search and filter rows align and the results bar keeps its wide-screen chrome at the narrow breakpoint", async ({
  page,
}) => {
  await page.goto(similar);
  await expect(
    page.locator(".reference-results [data-record-key]").first(),
  ).toBeVisible();
  for (const colorScheme of ["light", "dark"] as const) {
    await page.emulateMedia({ colorScheme });
    await expect(page.locator("html")).toHaveAttribute("data-theme", colorScheme);
    let wideStyle:
      | {
          background: string;
          padding: string;
          fontSize: string;
          fontWeight: string;
          height: number;
        }
      | undefined;
    for (const width of [1440, 1024, 992, 991, 768]) {
      await page.setViewportSize({ width, height: 900 });
      const bar = page.locator(".reference-browser__toolbar");
      if (width <= 991) await expect(bar).toHaveClass(/reference-results__header/);
      else await expect(bar).not.toHaveClass(/reference-results__header/);
      await expect(bar).toBeVisible();
      const geometry = await page
        .locator(".reference-browser__toolbar")
        .evaluate((bar) => {
          const caption = bar.querySelector(".result-pane-header > span")!;
          const sort = bar.querySelector(".result-pane-header button")!;
          const a = caption.getBoundingClientRect();
          const b = sort.getBoundingClientRect();
          return {
            style: {
              background: getComputedStyle(bar).backgroundColor,
              padding: getComputedStyle(
                bar.querySelector(".browse-pane-header__content")!,
              ).padding,
              fontSize: getComputedStyle(caption).fontSize,
              fontWeight: getComputedStyle(caption).fontWeight,
              height: bar.getBoundingClientRect().height,
            },
            captionCenter: a.y + a.height / 2,
            sortCenter: b.y + b.height / 2,
          };
        });
      wideStyle ??= geometry.style;
      expect(geometry.style).toEqual(wideStyle);
      expect(geometry.captionCenter).toBeCloseTo(geometry.sortCenter, 0);
      const search = await page.locator(".filter-controls__search-row").boundingBox();
      const controls = await page.locator(".filter-controls__tools").boundingBox();
      expect(search!.x).toBeCloseTo(controls!.x, 0);
      expect(search!.width).toBeCloseTo(controls!.width, 0);
      const settings = await page
        .getByRole("button", { name: "Search settings", exact: true })
        .boundingBox();
      const lastFilter = await page
        .locator(".filter-controls__tools button")
        .last()
        .boundingBox();
      expect(settings!.x + settings!.width).toBeCloseTo(
        lastFilter!.x + lastFilter!.width,
        0,
      );
    }
  }
});

test("link picker keeps target filters while refreshing its connected corpus and stable header", async ({
  page,
}) => {
  const filter = encodeFilterUrl({
    kind: "compare",
    field: "kind",
    op: "eq",
    value: "creature",
  });
  await page.goto(`/search?filter=${encodeURIComponent(filter)}`);
  await expect(
    page.locator(".reference-results [data-record-key]").first(),
  ).toBeVisible();
  await page.getByRole("button", { name: "Links", exact: true }).click();
  const picker = page.getByRole("dialog", { name: "Add link condition", exact: true });
  const search = picker.getByRole("searchbox", { name: "Search records", exact: true });
  await expect(search).toBeVisible();
  await expect(
    picker.locator(".reference-results [data-record-key]").first(),
  ).toBeVisible();
  await expect(picker.getByRole("button", { name: "Links", exact: true })).toHaveCount(
    0,
  );
  const top = await picker.locator(".link-picker-context").boundingBox();
  const searchBox = await search.boundingBox();
  expect(top!.y + top!.height).toBeLessThanOrEqual(searchBox!.y);
  await picker.getByRole("button", { name: "Edit Kind filter", exact: true }).click();
  await page
    .getByRole("button", {
      name: "Kind: Spell, neutral; activate to include",
      exact: true,
    })
    .click();
  await page.keyboard.press("Escape");
  await expect(
    picker.getByRole("button", { name: "Remove included Spell from Kind" }),
  ).toBeVisible();
  await expect(
    picker.locator(".reference-results [data-record-key]").first(),
  ).toBeVisible();
  await picker.locator(".reference-results [data-record-key]").first().click();
  const add = picker.getByRole("button", { name: "Add", exact: true });
  await expect(add).toBeEnabled();
  for (const width of [1440, 390, 320]) {
    await page.setViewportSize({ width, height: 844 });
    await expect
      .poll(() =>
        picker.locator(".reference-picker__footer").evaluate((footer) => {
          const buttons = [
            ...footer.querySelectorAll(".reference-picker__actions button"),
          ];
          const [cancel, add] = buttons.map((button) => button.getBoundingClientRect());
          return (
            Math.abs(cancel.y - add.y) < 1 &&
            Math.abs(footer.getBoundingClientRect().height - cancel.height) < 1 &&
            buttons.every(
              (button) =>
                button.getBoundingClientRect().right <=
                footer.getBoundingClientRect().right,
            ) &&
            footer.scrollWidth <= footer.clientWidth &&
            !footer.querySelector("a")
          );
        }),
      )
      .toBe(true);
  }
  await page.setViewportSize({ width: 1440, height: 844 });
  let release!: () => void;
  let requested!: () => void;
  const hold = new Promise<void>((resolve) => {
    release = resolve;
  });
  const pending = new Promise<void>((resolve) => {
    requested = resolve;
  });
  await page.route("**/api/search/page", async (route) => {
    const request = route.request().postDataJSON();
    if (request.connection_scope?.direction !== "outgoing") return route.continue();
    const response = await route.fetch();
    requested();
    await hold;
    await route.fulfill({ response });
  });
  await picker.getByText("Linked from", { exact: true }).click();
  try {
    await pending;
    await expect(search).toBeVisible();
    await expect(
      picker.getByRole("button", { name: "Remove included Spell from Kind" }),
    ).toBeVisible();
    await expect(add).toBeDisabled();
    await expect(picker.getByText("Direction changed. Choose a record.")).toBeVisible();
  } finally {
    release();
  }
  for (const [width, height] of [
    [1440, 844],
    [768, 844],
    [390, 844],
    [844, 390],
    [320, 844],
  ]) {
    await page.setViewportSize({ width, height });
    await expect(
      picker.getByRole("combobox", { name: "Link filter scope", exact: true }),
    ).toBeVisible();
    await expect(search).toBeVisible();
    await expect
      .poll(() =>
        picker.locator(".filter-toolbar").evaluate((toolbar) => {
          const bottom = toolbar.getBoundingClientRect().bottom;
          return [
            toolbar.querySelector('[aria-label="Search records"]'),
            toolbar.querySelector('[aria-label="Advanced filters"]'),
          ].every(
            (control) => control && control.getBoundingClientRect().bottom <= bottom,
          );
        }),
      )
      .toBe(true);
    await expect(
      picker.getByRole("button", { name: "Cancel", exact: true }),
    ).toBeVisible();
    expect(await picker.evaluate((host) => host.scrollWidth <= host.clientWidth)).toBe(
      true,
    );
  }
});

test("editing a link opens its current target and clearing never restores it", async ({
  page,
}) => {
  const filter = encodeFilterUrl({
    kind: "relationship",
    clause_id: "edit-preview",
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
  await page.goto(`/search?filter=${encodeURIComponent(filter)}`);
  await page
    .getByRole("group", { name: "Relationship filter", exact: true })
    .getByRole("button", { name: /Links to Wovenwood Shield/ })
    .click();
  const picker = page.getByRole("dialog", { name: "Edit link condition", exact: true });
  await expect(
    picker.getByRole("heading", { name: "Wovenwood Shield (Major)", exact: true }),
  ).toBeVisible();
  await expect(picker.locator(".reference-picker__footer a")).toHaveCount(0);
  await picker.getByRole("button", { name: "Clear selection", exact: true }).click();
  await expect(picker.getByText("Select a record", { exact: true })).toBeVisible();
  await picker.getByText("Linked from", { exact: true }).click();
  await expect(
    picker.getByRole("button", { name: "Apply", exact: true }),
  ).toBeDisabled();
  await expect(picker.locator(".record-reference__title")).toHaveCount(0);
});

async function renderedContrast(
  locator: import("@playwright/test").Locator,
  radio = false,
) {
  let ratio = 0;
  await expect
    .poll(async () => {
      ratio = await locator.evaluate(async (element, radio) => {
        const root = radio ? element.closest("label")! : element;
        await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
        getComputedStyle(root).getPropertyValue("color");
        await Promise.all(
          root
            .getAnimations({ subtree: true })
            .filter(
              (animation) =>
                animation.effect?.getComputedTiming().iterations !== Infinity,
            )
            .map((animation) => animation.finished.catch(() => {})),
        );
        if (!root.isConnected) return 0;
        const ink = radio ? root.lastElementChild! : root;
        const channels = (color: string) => color.match(/[\d.]+/g)!.map(Number);
        let background = [0, 0, 0, 0];
        for (
          let current: Element | null = root;
          current;
          current = current.parentElement
        ) {
          const value = channels(getComputedStyle(current).backgroundColor);
          const alpha = value[3] ?? 1;
          background = [
            ...background
              .slice(0, 3)
              .map(
                (channel, index) =>
                  channel + value[index] * alpha * (1 - background[3]),
              ),
            background[3] + alpha * (1 - background[3]),
          ];
          if (background[3] >= 0.999) break;
        }
        const luminance = (rgb: number[]) =>
          rgb
            .slice(0, 3)
            .map((value) => value / 255)
            .map((value) =>
              value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4,
            )
            .reduce(
              (sum, value, index) => sum + value * [0.2126, 0.7152, 0.0722][index],
              0,
            );
        const values = [
          luminance(channels(getComputedStyle(ink).color)),
          luminance(background),
        ].sort((a, b) => b - a);
        return (values[0] + 0.05) / (values[1] + 0.05);
      }, radio);
      return ratio;
    })
    .toBeGreaterThan(0);
  return ratio;
}

test("shared selected and hover states keep readable rendered contrast in both themes", async ({
  page,
}) => {
  const filter = encodeFilterUrl({
    kind: "relationship",
    clause_id: "contrast",
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
  for (const colorScheme of ["light", "dark"] as const) {
    await page.emulateMedia({ colorScheme });
    await page.goto("/records/equipment-srd:e9Xiei2wduOEGI5r");
    await expect(page.locator("html")).toHaveAttribute("data-theme", colorScheme);
    await page
      .getByRole("button", { name: "Explore related records", exact: true })
      .click();
    const tab = page.getByRole("tab", { selected: true }).first();
    await expect(tab).toBeVisible();
    expect(
      await renderedContrast(tab),
      `${colorScheme} selected tab`,
    ).toBeGreaterThanOrEqual(4.5);
    await tab.hover();
    expect(
      await renderedContrast(tab),
      `${colorScheme} hovered selected tab`,
    ).toBeGreaterThanOrEqual(4.5);
    await page.getByRole("button", { name: "Theme", exact: true }).click();
    const appearance = page.getByRole("radio", { name: "System", exact: true });
    await expect(appearance).toBeChecked();
    expect(
      await renderedContrast(appearance, true),
      `${colorScheme} selected segmented option`,
    ).toBeGreaterThanOrEqual(4.5);
    await page.keyboard.press("Escape");
    await page.goto(`/search?filter=${encodeURIComponent(filter)}`);
    await page
      .getByRole("group", { name: "Relationship filter", exact: true })
      .getByRole("button", { name: /Links to Wovenwood Shield/ })
      .click();
    const picker = page.getByRole("dialog", {
      name: "Edit link condition",
      exact: true,
    });
    const solid = picker.getByRole("radio", { name: "Links to", exact: true });
    await expect(solid).toBeChecked();
    expect(
      await renderedContrast(solid, true),
      `${colorScheme} selected solid radio`,
    ).toBeGreaterThanOrEqual(4.5);
    const cancel = picker.getByRole("button", { name: "Cancel", exact: true });
    await cancel.hover();
    expect(
      await renderedContrast(cancel),
      `${colorScheme} hovered outlined button`,
    ).toBeGreaterThanOrEqual(4.5);
    await page.mouse.down();
    try {
      expect(
        await renderedContrast(cancel),
        `${colorScheme} pressed outlined button`,
      ).toBeGreaterThanOrEqual(4.5);
    } finally {
      await page.mouse.up();
    }
  }
});

test("Advanced link edits inside OR disclose all-entry discovery and send static context", async ({
  page,
}) => {
  const filter = encodeFilterUrl({
    kind: "any_of",
    children: [
      { kind: "compare", field: "kind", op: "eq", value: "creature" },
      {
        kind: "relationship",
        clause_id: "or-edit",
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
      },
    ],
  });
  await page.goto("/search");
  const trigger = page.getByRole("button", { name: "Advanced filters", exact: true });
  await expect(trigger).toBeVisible();
  await page.evaluate(() => document.fonts.ready);
  const idleWidth = (await trigger.boundingBox())!.width;
  await expect(trigger.locator(".filter-controls__advanced-dot")).toHaveCount(0);
  await page.goto(`/search?q=cat&filter=${encodeURIComponent(filter)}`);
  await expect(trigger).toHaveAttribute(
    "aria-description",
    "Additional filters applied",
  );
  expect((await trigger.boundingBox())!.width).toBeCloseTo(idleWidth, 0);
  for (const colorScheme of ["light", "dark"] as const) {
    await page.emulateMedia({ colorScheme });
    await expect(page.locator("html")).toHaveAttribute("data-theme", colorScheme);
    await expect(trigger).toHaveAttribute("data-applied", "true");
    await expect(trigger.locator(".filter-controls__advanced-dot")).toHaveCount(0);
    expect(await renderedContrast(trigger)).toBeGreaterThanOrEqual(4.5);
  }
  await trigger.click();
  const advanced = page.getByRole("dialog", { name: "Advanced filters", exact: true });
  const response = page.waitForResponse(
    async (response) =>
      response.url().endsWith("/api/search/page") &&
      !!response.request().postDataJSON()?.connection_scope,
  );
  await expect(advanced.getByText(/Wovenwood Shield.*Whole entry/)).toBeVisible();
  await advanced.getByRole("button", { name: "Edit link", exact: true }).click();
  const picker = page.getByRole("dialog", { name: "Edit link condition", exact: true });
  await expect(
    picker.getByText("Searching all entries · Complex filter"),
  ).toBeVisible();
  const result = await response;
  expect(result.ok()).toBe(true);
  const context = result.request().postDataJSON().connection_scope.parent;
  expect(context.exclude_clause_id).toBeTruthy();
  expect(context.filter.kind).toBe("any_of");
  expect(context).not.toHaveProperty("criteria");
  expect(context).not.toHaveProperty("query");
  expect((await result.json()).connection_scope).toEqual({
    kind: "all_entries",
    reason: "non_conjunctive_context",
  });
  const guidance = picker.locator(".link-picker-context__description");
  await guidance.evaluate((element) => {
    const observer = new MutationObserver(() => {
      if (element.textContent !== "Searching all entries · Complex filter")
        element.setAttribute("data-scope-flash", "true");
    });
    observer.observe(element, { childList: true, subtree: true, characterData: true });
  });
  let release!: () => void;
  let requested!: () => void;
  let fulfilled!: () => void;
  const hold = new Promise<void>((resolve) => {
    release = resolve;
  });
  const pending = new Promise<void>((resolve) => {
    requested = resolve;
  });
  const finished = new Promise<void>((resolve) => {
    fulfilled = resolve;
  });
  await page.route("**/api/search/page", async (route) => {
    if (route.request().postDataJSON().connection_scope?.direction !== "outgoing")
      return route.continue();
    const response = await route.fetch();
    requested();
    await hold;
    await route.fulfill({ response });
    fulfilled();
  });
  await picker.getByText("Linked from", { exact: true }).click();
  try {
    await pending;
    await expect(guidance).toHaveText("Searching all entries · Complex filter");
    await expect(guidance).not.toHaveAttribute("data-scope-flash", "true");
  } finally {
    release();
    await finished;
    await page.unrouteAll({ behavior: "wait" });
  }
  await picker.getByText("Links to", { exact: true }).click();
  await expect(guidance).toHaveText("Searching all entries · Complex filter");
  await expect(guidance).not.toHaveAttribute("data-scope-flash", "true");
});

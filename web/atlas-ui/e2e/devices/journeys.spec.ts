import { expect, test } from "./fixtures";
import type { Page } from "@playwright/test";
import {
  activate,
  landedSearch,
  landing,
  noPageOverflow,
  readThenBack,
  sentinel,
} from "./reading";

test("theme dismissal returns focus without stealing the next modal's focus", async ({
  page,
}, info) => {
  await page.goto("/lists");
  const theme = page.getByRole("button", { name: "Theme", exact: true });
  await activate(theme, info);
  await activate(page.getByText("Dark", { exact: true }), info);
  await page.keyboard.press("Escape");
  await expect(page.getByText("Dark", { exact: true })).toBeHidden();
  await expect(theme).toBeFocused();
  await activate(theme, info);
  await activate(page.getByRole("button", { name: "New list", exact: true }), info);
  const dialog = page.getByRole("dialog", { name: "New List", exact: true });
  await expect(dialog).toBeVisible();
  const name = dialog.getByRole("textbox", { name: /Name$/ });
  await name.fill("Retained draft");
  await expect(name).toBeFocused();
  await expect(page.getByText("Dark", { exact: true })).toBeHidden();
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  const create = page.getByRole("button", { name: "New list", exact: true });
  await expect(create).toBeFocused();
  await activate(create, info);
  await expect(name).toHaveValue("Retained draft");
});

test("a long saved-list choice stays bounded through a pending and failed Add", async ({
  page,
}, info) => {
  const slug = `overlay-${info.project.name}-${Date.now()}`;
  const name =
    "Expedition medicine and emergency supplies for the northern mountain watch";
  const response = await page.request.post("/api/lists", {
    data: { slug, name, tags: [] },
  });
  expect(response.ok()).toBeTruthy();
  const { list } = await response.json();
  let release = () => {};
  const gate = new Promise<void>((resolve) => {
    release = resolve;
  });
  let writes = 0;
  try {
    await page.route(`**/api/lists/${slug}/items`, async (route) => {
      writes += 1;
      await gate;
      await route.fulfill({
        status: 503,
        json: { code: "unavailable", message: "Service unavailable. Try again." },
      });
    });
    await page.goto(`/records/${sentinel}`);
    const trigger = page.getByRole("button", {
      name: "Add to saved list",
      exact: true,
    });
    await activate(trigger, info);
    const dialog = page.getByRole("dialog", { name: "Add to List", exact: true });
    const selector = dialog.getByRole("combobox", { name: "Saved list", exact: true });
    await expect(selector).toBeEnabled();
    await activate(selector, info);
    const option = page.getByText(name, { exact: true });
    await expect
      .poll(async () => {
        const choice = await option.boundingBox();
        const field = await dialog.locator(".list-picker").boundingBox();
        return choice && field ? field.width - choice.width : Infinity;
      })
      .toBeLessThan(48);
    await activate(option, info);
    await expect(page.getByRole("listbox")).toBeHidden();
    await expect(selector.locator("..")).toHaveText(name);
    const assertBounds = async () => {
      const bounds = (await dialog.boundingBox())!;
      const control = (await selector.boundingBox())!;
      const viewport = page.viewportSize()!;
      expect(bounds.x).toBeGreaterThanOrEqual(0);
      expect(bounds.x + bounds.width).toBeLessThanOrEqual(viewport.width);
      expect(control.x + control.width).toBeLessThanOrEqual(bounds.x + bounds.width);
      await noPageOverflow(page);
    };
    await assertBounds();
    await activate(dialog.getByRole("button", { name: "Add", exact: true }), info);
    await expect.poll(() => writes).toBe(1);
    await expect(
      dialog.getByRole("button", { name: "Cancel", exact: true }),
    ).toBeDisabled();
    await expect(
      dialog.getByRole("button", { name: "Close", exact: true }),
    ).toBeDisabled();
    await expect(selector).toBeDisabled();
    await page.keyboard.press("Escape");
    if (info.project.use.hasTouch) await page.touchscreen.tap(2, 2);
    else await page.mouse.click(2, 2);
    await expect(dialog).toBeVisible();
    release();
    await expect(dialog.getByRole("alert")).toContainText("Service unavailable");
    await expect(
      dialog.getByRole("button", { name: "Add", exact: true }),
    ).toBeEnabled();
    await expect(selector.locator("..")).toHaveText(name);
    await expect(page.getByRole("listbox")).toBeHidden();
    await assertBounds();
    await page.screenshot({
      path: info.outputPath("list-add-failure.png"),
      animations: "disabled",
    });
    await activate(dialog.getByRole("button", { name: "Cancel", exact: true }), info);
    await expect(dialog).toBeHidden();
    await expect(trigger).toBeFocused();
    expect(writes).toBe(1);
    expect(
      (await (await page.request.get(`/api/lists/${list.list_key}`)).json()).items,
    ).toHaveLength(0);
  } finally {
    release();
    await page.request.delete(`/api/lists/${list.list_key}`);
  }
});

test("compact filter links reload and recover without losing the selected entry", async ({
  page,
}) => {
  await page.goto(landedSearch);
  const title = page.getByRole("heading", { name: "Atlas Sentinel", exact: true });
  await expect(title).toBeVisible();
  expect(new URL(page.url()).searchParams.get("filter")).toMatch(/^3\./);
  await page.reload();
  await expect(title).toBeVisible();

  const broken = new URL(page.url());
  const retainedLanding = broken.searchParams.get("landing");
  broken.searchParams.set("filter", "2.obsolete");
  broken.searchParams.set("sort", "name_asc");
  broken.searchParams.set("limit", "50");
  await page.goto(broken.href);
  const notice = page.getByText(
    "Filters in this link couldn’t be restored and were cleared.",
    { exact: true },
  );
  await expect(notice).toBeVisible();
  await expect(title).toBeVisible();
  const recovered = new URL(page.url());
  expect(recovered.searchParams.has("filter")).toBe(false);
  expect(recovered.searchParams.get("landing")).toBe(retainedLanding);
  expect(recovered.searchParams.get("sort")).toBe("name_asc");
  expect(recovered.searchParams.get("limit")).toBe("50");
  await noPageOverflow(page);

  await page.reload();
  await expect(title).toBeVisible();
  await expect(notice).toHaveCount(0);
  expect(new URL(page.url()).searchParams.has("filter")).toBe(false);
});

test("focused titles and list text filters follow their primary page", async ({
  page,
}) => {
  const slug = `focused-titles-${Date.now()}`;
  const created = await page.request.post("/api/lists", {
    data: { slug, name: "Mountain Watch", tags: [] },
  });
  expect(created.ok()).toBeTruthy();
  const { list } = await created.json();
  try {
    for (const record_ref of [sentinel, "browser-conditions:offguard00000001"]) {
      const added = await page.request.post(`/api/lists/${slug}/items`, {
        data: { list_ref: slug, record_ref },
      });
      expect(added.ok()).toBeTruthy();
    }
    await page.goto("/search");
    await expect(page).toHaveTitle("Search · Atlas");
    const search = page.getByRole("searchbox", { name: "Search records" });
    await search.fill(sentinel);
    await search.press("Enter");
    await expect(
      page.getByRole("heading", { name: "Atlas Sentinel", exact: true }),
    ).toBeVisible();
    await expect(page).toHaveTitle("Search · Atlas");

    await page.goto(`/records/${sentinel}`);
    await expect(page).toHaveTitle("Atlas Sentinel · Atlas");
    await page.goto(`/lists/${slug}/records/${sentinel}`);
    await expect(page).toHaveTitle("Mountain Watch · Atlas");
    await expect(
      page.getByRole("heading", { name: "Atlas Sentinel", exact: true }),
    ).toBeVisible();
    await page.goto(`/lists/${slug}`);
    const input = page.getByRole("searchbox", { name: "Search records" });
    await expect(page.locator(".list-member__drag-handle")).toHaveCount(2);
    await expect(page.getByRole("combobox", { name: "Search model" })).toHaveCount(0);
    await input.fill("zzzz-no-saved-entry");
    await input.press("Enter");
    await expect(page.getByText("No saved records match these filters")).toBeVisible();
    await expect(page).toHaveTitle("Mountain Watch · Atlas");
    await input.fill("");
    await expect(page.locator(".list-member__drag-handle")).toHaveCount(2);
    await expect(page.locator("[data-sortable-key]").first()).toHaveAttribute(
      "data-sortable-key",
      sentinel,
    );
    await page.goto("/lists");
    await expect(page).toHaveTitle("Lists · Atlas");
    await page.goto("/encounters");
    await expect(page).toHaveTitle("Encounters · Atlas");
  } finally {
    await page.request.delete(`/api/lists/${list.list_key}`);
  }
});

test("list drops retain order and geometry through delayed writes and refreshes", async ({
  page,
}, info) => {
  const slug = `reorder-${info.project.name}-${Date.now()}`;
  const created = await page.request.post("/api/lists", {
    data: { slug, name: "Reorder check", tags: [] },
  });
  expect(created.ok()).toBeTruthy();
  const { list } = await created.json();
  const keys = [sentinel, "browser-conditions:offguard00000001"];
  let releaseWrite = () => {};
  let releaseRead = () => {};
  const writeGate = new Promise<void>((resolve) => {
    releaseWrite = resolve;
  });
  const readGate = new Promise<void>((resolve) => {
    releaseRead = resolve;
  });
  let writes = 0;
  let refreshStarted = false;
  try {
    for (const key of keys) {
      const added = await page.request.post(`/api/lists/${slug}/items`, {
        data: { list_ref: slug, record_ref: key },
      });
      expect(added.ok()).toBeTruthy();
    }
    await page.goto(`/lists/${slug}`);
    const rows = page.locator("[data-sortable-key]");
    const handles = page.locator(".list-member__drag-handle");
    await expect(handles).toHaveCount(2);
    const source = handles.first();
    const sourceName = (await source.getAttribute("aria-label"))!;
    const persistedKeys = async () =>
      (
        (await (await page.request.get(`/api/lists/${slug}`)).json()).items as {
          record_key: string;
        }[]
      ).map((item) => item.record_key);
    await page.route(`**/api/lists/${slug}/items/reorder`, async (route) => {
      writes += 1;
      await writeGate;
      await route.continue();
    });
    await page.route(`**/api/lists/${slug}/filter`, async (route) => {
      refreshStarted = true;
      await readGate;
      await route.continue();
    });
    const keyboardFirst = Boolean(info.project.use.hasTouch);
    if (keyboardFirst) {
      await source.focus();
      // A normal key hold lets the sortable sensor attach its deferred document
      // listener before the next keyboard command.
      await page.keyboard.press("Space", { delay: 20 });
      await expect(source).toHaveAttribute("aria-pressed", "true");
      await page.keyboard.press("ArrowDown");
      await expect(page.getByRole("status")).toContainText(
        `was moved over droppable area ${keys[1]}`,
      );
      await page.keyboard.press("Space");
    } else {
      const from = (await source.boundingBox())!;
      const to = (await handles.nth(1).boundingBox())!;
      await page.mouse.move(from.x + from.width / 2, from.y + from.height / 2);
      await page.mouse.down();
      await page.mouse.move(to.x + to.width / 2, to.y + to.height / 2, { steps: 12 });
      await page.mouse.up();
    }
    await expect.poll(() => writes).toBe(1);
    await expect(rows.first()).toHaveAttribute("data-sortable-key", keys[1]);
    await expect(handles.first()).toBeDisabled();
    await expect(page.getByRole("button", { name: /^Remove / }).first()).toBeDisabled();
    const top = (await page.getByLabel("List items", { exact: true }).boundingBox())!.y;
    releaseWrite();
    await expect.poll(() => refreshStarted).toBe(true);
    await expect(
      page.getByLabel("Updating saved records").locator(".ant-spin-spinning"),
    ).toBeVisible();
    expect(Math.abs((await rows.first().boundingBox())!.y - top)).toBeLessThan(1);
    expect(writes).toBe(1);
    releaseRead();
    const movedHandle = page.getByRole("button", { name: sourceName, exact: true });
    await expect(movedHandle).toBeEnabled();
    if (keyboardFirst) await expect(movedHandle).toBeFocused();
    expect(await persistedKeys()).toEqual([keys[1], keys[0]]);
    // A second keyboard move starts from the newly saved order, then survives reload.
    await movedHandle.focus();
    await page.keyboard.press("Space", { delay: 20 });
    await expect(movedHandle).toHaveAttribute("aria-pressed", "true");
    await page.keyboard.press("ArrowUp");
    await expect(page.getByRole("status")).toContainText(
      `was moved over droppable area ${keys[1]}`,
    );
    await page.keyboard.press("Space");
    await expect.poll(() => writes).toBe(2);
    await expect(movedHandle).toBeEnabled();
    await expect(movedHandle).toBeFocused();
    expect(await persistedKeys()).toEqual(keys);
    await page.reload();
    await expect(rows.first()).toHaveAttribute("data-sortable-key", keys[0]);
    await noPageOverflow(page);
    // Cancelling pickup never submits another durable mutation.
    await handles.first().focus();
    await page.keyboard.press("Space", { delay: 20 });
    await expect(handles.first()).toHaveAttribute("aria-pressed", "true");
    await page.keyboard.press("Escape");
    await expect(handles.first()).not.toHaveAttribute("aria-pressed", "true");
    expect(writes).toBe(2);
    expect(await persistedKeys()).toEqual(keys);
  } finally {
    releaseWrite();
    releaseRead();
    await page.unrouteAll({ behavior: "wait" });
    expect(
      (await page.request.delete(`/api/lists/${list.list_key}`)).ok(),
    ).toBeTruthy();
  }
});

test("filters, exact late activity, preview dismissal and Results remain usable", async ({
  page,
}, info) => {
  const ready = await page.request.get("/api/readiness");
  expect(ready.ok()).toBeTruthy();
  expect((await ready.json()).search_profiles).toEqual([
    expect.objectContaining({
      stored_vectors: true,
      assets_ready: false,
      usable: false,
    }),
  ]);
  await page.goto("/search");
  const kind = page.getByRole("button", { name: "Edit Kind filter", exact: true });
  await activate(kind, info);
  const choice = page.getByRole("button", { name: /^Kind: Creature, neutral;/ });
  await activate(choice, info);
  const close = page.getByRole("button", { name: "Close", exact: true });
  if (await close.isVisible()) await activate(close, info);
  else await page.keyboard.press("Escape");
  await expect(page.locator("[data-record-key]").first()).toBeVisible();
  await noPageOverflow(page);
  await page.goto(landedSearch);
  const host = page.locator(".reference-reader__scroll");
  await expect(host.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
  expect(await host.evaluate((node) => node.scrollTop)).toBeGreaterThan(0);
  const outer = await page.evaluate(() => scrollY);
  await readThenBack(page, host, info, "Off-Guard", "close");
  expect(await page.evaluate(() => scrollY)).toBe(outer);
  const full = page.getByRole("link", { name: "Open full page", exact: true });
  expect(
    new URL((await full.getAttribute("href"))!, page.url()).searchParams.get("landing"),
  ).toBe(JSON.stringify(landing));
  await noPageOverflow(page);
  const results = page.getByRole("button", { name: "Results", exact: true });
  if (await results.isVisible()) {
    await activate(results, info);
    await expect(page.locator('[data-record-key][aria-current="true"]')).toBeFocused();
    await expect(page.getByRole("searchbox", { name: "Search records" })).toBeVisible();
  }
});

test("standalone records scroll as pages from the gutter and keyboard", async ({
  page,
  browserName,
}, info) => {
  await page.goto(`/records/${sentinel}`);
  await expect(
    page.getByRole("heading", { name: "Atlas Sentinel", exact: true }),
  ).toBeVisible();
  // Playwright does not provide mouse-wheel input in mobile WebKit.
  if (browserName !== "webkit" || !info.project.use.isMobile) {
    await page.mouse.move(8, 320);
    await page.mouse.wheel(0, 600);
    await expect.poll(() => page.evaluate(() => scrollY)).toBeGreaterThan(100);
  }
  await page.keyboard.press("Home");
  await expect.poll(() => page.evaluate(() => scrollY)).toBe(0);
  await page.keyboard.press("PageDown");
  await expect.poll(() => page.evaluate(() => scrollY)).toBeGreaterThan(100);
  expect(
    await page.locator(".record-view__pane").evaluate((node) => node.scrollTop),
  ).toBe(0);
  await noPageOverflow(page);
});

test("closing a reference with a feature-action tooltip preserves the owner viewport and position", async ({
  page,
}, info) => {
  await page.goto(
    `/records/${sentinel}?landing=${encodeURIComponent(JSON.stringify(landing))}`,
  );
  const host = page.locator(".record-view__pane");
  await expect(host.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
  const activity = host.locator(".embedded-ability-reading__entry").filter({
    has: page.getByRole("heading", { name: /^Signal Pulse\b/ }),
  });
  const link = activity.getByRole("link", { name: "Off-Guard", exact: true });
  await link.scrollIntoViewIfNeeded();
  await link.focus();
  const readPosition = () =>
    host.evaluate((node) => ({ pane: node.scrollTop, page: scrollY }));
  const position = await readPosition();
  expect(Math.max(position.pane, position.page)).toBeGreaterThan(0);
  await link.press("Enter");
  const drawer = page.getByRole("dialog", { name: "Off-Guard", exact: true });
  await expect(drawer).toBeVisible();
  const save = drawer.getByRole("button", { name: "Add to saved list", exact: true });
  await save.focus();
  await save.hover();
  await expect(page.getByRole("tooltip", { name: "Add to saved list" })).toBeVisible();
  await page.evaluate(() => {
    const probe = { running: true, widths: [] as number[] };
    (
      window as typeof window & { referenceTransition: typeof probe }
    ).referenceTransition = probe;
    const sample = () => {
      if (!probe.running) return;
      probe.widths.push(Math.max(innerWidth, document.documentElement.scrollWidth));
      requestAnimationFrame(sample);
    };
    sample();
  });
  await activate(
    drawer.getByRole("button", { name: "Close reference preview", exact: true }),
    info,
  );
  await expect(drawer).toBeHidden();
  await expect(link).toBeFocused();
  const widths = await page.evaluate(() => {
    const probe = (
      window as typeof window & {
        referenceTransition: { running: boolean; widths: number[] };
      }
    ).referenceTransition;
    probe.running = false;
    return probe.widths;
  });
  await info.attach("reference-transition-widths", {
    body: JSON.stringify(widths),
    contentType: "application/json",
  });
  expect(Math.max(...widths)).toBe(widths[0]);
  expect(Math.min(...widths)).toBe(widths[0]);
  await expect.poll(readPosition).toEqual(position);
  await noPageOverflow(page);
});

test("reference rank menus accept pointer selection and retain their trigger", async ({
  page,
}, info) => {
  await page.goto("/records/browser-rules:rule000000000001");
  const link = page.getByRole("link", { name: "Lantern Ward", exact: true });
  await activate(link, info);
  const drawer = page.getByRole("dialog", { name: "Lantern Ward", exact: true });
  await expect(drawer).toBeVisible();
  const changeRank = drawer.getByRole("button", { name: "Change rank", exact: true });
  const trigger = await changeRank.elementHandle();
  await activate(changeRank, info);
  await activate(page.getByRole("menuitem", { name: "Rank 4", exact: true }), info);
  await expect(drawer.getByText("Spell 4", { exact: true })).toBeVisible();
  expect(await trigger!.evaluate((node) => node.isConnected)).toBe(true);
  await expect(changeRank).toBeFocused();
  const href = await drawer
    .getByRole("link", { name: "Open reference full page", exact: true })
    .getAttribute("href");
  expect(
    JSON.parse(new URL(href!, page.url()).searchParams.get("selection")!).spell_rank,
  ).toBe(4);
  await activate(
    drawer.getByRole("button", { name: "Close reference preview", exact: true }),
    info,
  );
  await expect(drawer).toBeHidden();
  await expect(link).toBeFocused();
  await noPageOverflow(page);
});

test("list tags, editing and explicit Add survive constrained space", async ({
  page,
}, info) => {
  const name = `Device list ${info.project.name} ${Date.now()}`;
  const slug = name.toLowerCase().replace(/\s+/g, "-");
  const created = await page.request.post("/api/lists", {
    data: {
      slug,
      name,
      description: "Readable collection context. ".repeat(18),
      tags: Array.from({ length: 20 }, (_, index) => `Long review tag ${index + 1}`),
    },
  });
  expect(created.ok()).toBeTruthy();
  const { list } = await created.json();
  let releaseAdd: (() => void) | undefined;
  try {
    await page.goto(`/lists/${list.slug}`);
    const more = page.getByRole("button", { name: /^\+\d+ more$/ });
    if (await more.isVisible()) {
      await activate(more, info);
      await expect(
        page.getByLabel("All list tags", { exact: true }).locator("span"),
      ).toHaveCount(20);
      // Touch activation does not establish keyboard focus in WebKit.
      await more.focus();
      await page.keyboard.press("Escape");
      await expect(page.getByLabel("All list tags", { exact: true })).toBeHidden();
      await expect(more).toBeFocused();
    }
    await activate(page.getByRole("link", { name: "Edit list", exact: true }), info);
    await page
      .getByRole("textbox", { name: "Description", exact: true })
      .fill("A retained description draft with a useful purpose.");
    await activate(page.getByRole("button", { name: "Save", exact: true }), info);
    await expect(page).toHaveURL(new RegExp(`/lists/${list.slug}$`));
    await activate(
      page.getByRole("button", { name: "Add records", exact: true }),
      info,
    );
    const dialog = page.getByRole("dialog", { name: "Add records", exact: true });
    await expect(dialog).toBeVisible();
    if (info.project.name === "tablet-webkit") {
      await page.setViewportSize({ width: 844, height: 390 });
      // Select a fresh candidate after rotation. Reachable footer buttons alone
      // do not prove that the result rows have a usable scroll area.
      await expect
        .poll(() =>
          dialog
            .locator(".reference-results__scroll")
            .evaluate((node) => node.clientHeight),
        )
        .toBeGreaterThan(100);
    }
    const row = dialog.locator(`[data-record-key="${sentinel}"]`);
    await activate(row, info);
    await expect(
      dialog.getByRole("button", { name: "Add", exact: true }),
    ).toBeEnabled();
    const results = dialog.getByRole("button", { name: "Results", exact: true });
    if (await results.isVisible()) await activate(results, info);
    await dialog
      .getByRole("searchbox", { name: "Search records" })
      .fill("unfinished candidate note");
    if (info.project.name === "tablet-webkit") {
      await page.setViewportSize({ width: 768, height: 1024 });
      await page.setViewportSize({ width: 844, height: 390 });
      await expect(
        dialog.getByRole("searchbox", { name: "Search records" }),
      ).toHaveValue("unfinished candidate note");
    }
    // Reading alone must leave the list unchanged. Add is the durable action.
    expect(
      (await (await page.request.get(`/api/lists/${list.slug}`)).json()).items,
    ).toHaveLength(0);
    let addArrived = false;
    const pendingAdd = new Promise<void>((resolve) => {
      releaseAdd = resolve;
    });
    await page.route(
      (url) => url.pathname === `/api/lists/${list.slug}/items`,
      async (route) => {
        if (route.request().method() === "POST") {
          addArrived = true;
          await pendingAdd;
        }
        await route.continue();
      },
    );
    await activate(dialog.getByRole("button", { name: "Add", exact: true }), info);
    await expect.poll(() => addArrived).toBe(true);
    await expect(
      dialog.getByRole("button", { name: /^(loading )?Add$/ }),
    ).toBeDisabled();
    await expect(
      dialog.getByRole("button", { name: "Cancel", exact: true }),
    ).toBeDisabled();
    await expect(dialog.getByRole("searchbox", { name: "Search records" })).toHaveValue(
      "unfinished candidate note",
    );
    releaseAdd!();
    await expect(
      dialog.getByRole("status").filter({ hasText: "Added to list" }),
    ).toBeVisible();
    await expect(dialog.getByRole("searchbox", { name: "Search records" })).toHaveValue(
      "unfinished candidate note",
    );
    await expect(
      dialog.getByRole("button", { name: "Add", exact: true }),
    ).toBeDisabled();
    await activate(dialog.getByRole("button", { name: "Cancel", exact: true }), info);
    await expect(dialog).toBeHidden();
    await noPageOverflow(page);
    await expect
      .poll(
        async () =>
          (await (await page.request.get(`/api/lists/${list.slug}`)).json()).items
            .length,
      )
      .toBe(1);
  } finally {
    releaseAdd?.();
    expect(
      (await page.request.delete(`/api/lists/${list.list_key}`)).ok(),
    ).toBeTruthy();
  }
});

test("encounter references retain their owner, scroll and native destination", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Device encounter ${info.project.name} ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/record`, {
          data: {
            encounter_ref: encounter.encounter_key,
            record_ref: sentinel,
            quantity: 1,
          },
        })
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    if ((page.viewportSize()?.width ?? 1440) < 768) {
      await activate(page.locator("[data-participant-key]").first(), info);
      await expect(page.getByRole("button", { name: "Back to roster" })).toBeVisible();
    }
    const originalSize = page.viewportSize()!;
    if (originalSize.width < 768)
      await page.setViewportSize({ width: originalSize.width, height: 560 });
    await activate(page.getByRole("link", { name: "Signal Pulse", exact: true }), info);
    const content = page.getByLabel("Reference content", { exact: true });
    const drawer = page.getByRole("dialog", { name: /^Signal Pulse\b/ });
    await expect(drawer).toBeVisible();
    const ownerReturn = drawer.getByRole("link", {
      name: "Return to Atlas Sentinel",
      exact: true,
    });
    await expect(ownerReturn).toHaveCount(0);
    const host = page.locator(".record-preview-drawer__body");
    await expect(host).toBeVisible();
    const heading = drawer.getByRole("heading", { name: /^Signal Pulse\b/ });
    await expect(heading).toBeFocused();
    if (originalSize.width >= 768) {
      const mountedDrawer = await drawer.elementHandle();
      await page.setViewportSize({ width: 844, height: 390 });
      await expect
        .poll(async () => {
          const box = await drawer.boundingBox();
          return (
            !!box &&
            box.x >= -1 &&
            box.y >= -1 &&
            box.x + box.width <= 845 &&
            box.y + box.height <= 391
          );
        })
        .toBe(true);
      expect(await mountedDrawer!.evaluate((node) => node.isConnected)).toBe(true);
      await noPageOverflow(page);
    }
    await expect(heading).toBeFocused();
    const outer = await page.evaluate(() => scrollY);
    // Capture a real scroll position even when the nested reference is already visible.
    await host.evaluate((node) => {
      node.scrollTop = node.scrollHeight;
    });
    await readThenBack(page, host, info, "Signal Protocol");
    expect(await page.evaluate(() => scrollY)).toBe(outer);
    // A real captured scroll must not restart the same ability's initial landing.
    const position = await host.evaluate((node) => node.scrollTop);
    const scrollDelta = Math.min(200, position);
    expect(scrollDelta).toBeGreaterThan(0);
    await host.evaluate((node, delta) => {
      node.scrollTop -= delta;
    }, scrollDelta);
    await expect
      .poll(() => host.evaluate((node) => node.scrollTop))
      .toBeCloseTo(position - scrollDelta, 0);
    const full = page.getByRole("link", {
      name: "Open reference full page",
      exact: true,
    });
    expect(
      new URL((await full.getAttribute("href"))!, page.url()).searchParams.get(
        "selection",
      ),
    ).toBe(JSON.stringify(landing));
    await expect(full).toBeInViewport();
    await page.screenshot({ path: info.outputPath("preview-toolbar.png") });
    if (!info.project.use.hasTouch) {
      // Scrolled record content must not intercept the toolbar above its viewport.
      await expect
        .poll(() =>
          full.evaluate((node) => {
            const box = node.getBoundingClientRect();
            return node.contains(
              document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2),
            );
          }),
        )
        .toBe(true);
      // Middle activation exercises the native link destination independently
      // of the host OS's Cmd/Ctrl convention. Component tests guard both modifiers.
      const opened = page.context().waitForEvent("page", { timeout: 5000 });
      await full.click({ button: "middle" });
      const other = await opened;
      // A native background tab can render while Chromium still reports a
      // pending navigation. Check the destination and usable content directly.
      await expect.poll(() => other.url()).toMatch(/\/records\/browser-actors:/);
      await expect(
        other.getByRole("heading", { name: /^Signal Pulse\b/ }),
      ).toBeVisible();
      expect(new URL(other.url()).searchParams.get("selection")).toBe(
        JSON.stringify(landing),
      );
      await expect(page).toHaveURL(new RegExp(`/encounters/${encounter.slug}$`));
      await other.close();
    }
    await activate(
      page.getByRole("button", {
        name: "Close reference preview",
        exact: true,
      }),
      info,
    );
    await expect(content).toBeHidden();
    await expect(
      page.getByRole("link", { name: "Signal Pulse", exact: true }),
    ).toBeFocused();
    await page.setViewportSize(originalSize);
    await activate(page.getByRole("button", { name: "Edit HP", exact: true }), info);
    const hpDialog = page.getByRole("dialog", { name: "Edit HP", exact: true });
    const hp = hpDialog.getByRole("textbox", { name: "Current HP", exact: true });
    await expect(hp).toHaveValue("60");
    let writes = 0;
    page.on("request", (request) => {
      if (
        request.method() === "POST" &&
        new URL(request.url()).pathname.startsWith(`${path}/participants/`) &&
        new URL(request.url()).pathname.endsWith("/hp")
      )
        writes++;
    });
    const detailRead = (url: URL) =>
      url.pathname === path || url.pathname === `/api/encounters/${encounter.slug}`;
    // Transport failure only: keep real requests, mutations and DTOs. Never
    // replace readiness, source data or a successful backend response.
    await page.route(detailRead, (route) => route.abort("failed"));
    await hp.fill("55");
    await activate(hpDialog.getByRole("button", { name: "Save", exact: true }), info);
    const feedback = hpDialog
      .getByRole("alert")
      .filter({ hasText: "Saved, but the encounter could not refresh" });
    await expect(feedback).toBeVisible();
    const settingsAction = page.getByRole("button", {
      name: "Settings",
      exact: true,
    });
    await expect(settingsAction).toBeVisible();
    await expect(settingsAction).toBeDisabled();
    expect(writes).toBe(1);
    await page.unroute(detailRead);
    await activate(
      hpDialog.getByRole("button", { name: "Retry encounter refresh" }),
      info,
    );
    await expect(feedback).toBeHidden();
    await expect(hpDialog).toBeHidden();
    await expect(page.getByLabel("HP summary", { exact: true })).toHaveText("55/60");
    expect(writes).toBe(1);
    await noPageOverflow(page);
  } finally {
    expect((await page.request.delete(path)).ok()).toBeTruthy();
  }
});

test("encounter roster, repeated Add, settings focus and HP intents survive narrow layouts", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Workflow ${info.project.name} ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/record`, {
          data: {
            encounter_ref: encounter.encounter_key,
            record_ref: sentinel,
            quantity: 8,
            initiative: 12,
          },
        })
      ).ok(),
    ).toBeTruthy();
    for (const [index, name] of [
      "Mira",
      "Tamsin the Lantern Keeper",
      "Corin",
      "Vel",
    ].entries()) {
      expect(
        (
          await page.request.post(`${path}/participants/manual`, {
            data: {
              encounter_ref: encounter.encounter_key,
              display_name: name,
              max_hp: 40,
              initiative: 20 - index,
            },
          })
        ).ok(),
      ).toBeTruthy();
    }
    await page.goto(`/encounters/${encounter.slug}`);
    const row = page
      .locator("[data-participant-key]")
      .filter({ hasText: "Tamsin the Lantern Keeper" });
    await row.scrollIntoViewIfNeeded();
    if (info.project.use.hasTouch) await row.tap();
    else {
      await row.focus();
      await page.keyboard.press("Enter");
    }
    await expect(page.getByLabel("HP summary", { exact: true })).toBeVisible();
    const back = page.getByRole("button", { name: "Back to roster", exact: true });
    if (await back.isVisible()) {
      await expectParticipantWidth(page);
      await activate(back, info);
      await expect(row).toBeFocused();
      await expect(row).toBeInViewport();
    }
    const handle = page.getByRole("button", {
      name: "Reorder Tamsin the Lantern Keeper",
      exact: true,
    });
    await handle.focus();
    await page.keyboard.press("Space");
    await expect(handle).toHaveAttribute("aria-pressed", "true");
    const beforeMove = await (await page.request.get(path)).json();
    const targetKey = beforeMove.participants[0].participant_key;
    await page.keyboard.press("ArrowUp");
    await expect(
      page.getByRole("status").filter({ hasText: `droppable area ${targetKey}.` }),
    ).toBeVisible();
    await page.keyboard.press("Space");
    await expect
      .poll(async () => {
        const response = await page.request.get(path);
        const view = await response.json();
        return view.participants
          .map((participant: { display_name: string }) => participant.display_name)
          .indexOf("Tamsin the Lantern Keeper");
      })
      .toBe(0);
    await expect(
      page.getByRole("button", { name: "Add participants", exact: true }),
    ).toBeEnabled();
    await activate(
      page.getByRole("button", { name: "Add participants", exact: true }),
      info,
    );
    await activate(page.getByRole("menuitem", { name: "Add PC…", exact: true }), info);
    await activate(
      page
        .getByRole("dialog", { name: "Add PC", exact: true })
        .getByRole("button", { name: "Cancel", exact: true }),
      info,
    );
    await activate(page.getByRole("button", { name: "Settings", exact: true }), info);
    const settings = page.getByRole("dialog", {
      name: "Settings",
      exact: true,
    });
    await activate(settings.locator("label").filter({ hasText: /^Name$/ }), info);
    await expect(
      settings.getByRole("textbox", { name: /^(?:\*\s*)?Name$/ }),
    ).toBeFocused();
    await activate(settings.getByRole("button", { name: "Cancel", exact: true }), info);
    await activate(
      page.getByRole("button", { name: "Add participants", exact: true }),
      info,
    );
    await activate(
      page.getByRole("menuitem", { name: "Add creature or hazard…", exact: true }),
      info,
    );
    const picker = page.getByRole("dialog", {
      name: "Add creature or hazard",
      exact: true,
    });
    // WebKit can complete a tap while the modal's opening transform still moves its target.
    await expect(picker).toBeVisible();
    await expect(picker).not.toHaveClass(/ant-zoom/);
    await activate(picker.locator(`[data-record-key="${sentinel}"]`).first(), info);
    await expect(
      picker.getByRole("button", { name: "Add", exact: true }),
    ).toBeEnabled();
    await picker.getByLabel("Quantity", { exact: true }).fill("2");
    await picker.getByLabel("Initiative", { exact: true }).fill("7");
    await activate(picker.getByRole("button", { name: "Add", exact: true }), info);
    await expect(picker.getByRole("status")).toContainText("Added 2 × Atlas Sentinel");
    await expect(picker.getByLabel("Quantity", { exact: true })).toHaveValue("1");
    await expect(picker.getByLabel("Initiative", { exact: true })).toHaveValue("");
    await expect(
      picker.getByRole("button", { name: "Add", exact: true }),
    ).toBeEnabled();
    await activate(picker.getByRole("button", { name: "Add", exact: true }), info);
    await expect(picker.getByRole("status")).toContainText("Added 1 × Atlas Sentinel");
    await activate(picker.getByRole("button", { name: "Done", exact: true }), info);
    await activate(page.getByRole("button", { name: "Start turn", exact: true }), info);
    await expect
      .poll(
        async () =>
          (await (await page.request.get(path)).json()).current_turn_participant_key,
      )
      .toBeTruthy();
    const view = await (await page.request.get(path)).json();
    await expect(
      page.locator(".encounter-record-pane").getByRole("heading", {
        name: view.participants.find(
          (p: { participant_key: string }) =>
            p.participant_key === view.current_turn_participant_key,
        ).display_name,
        exact: true,
      }),
    ).toBeVisible();
    const amount = page.getByRole("textbox", { name: "HP change", exact: true });
    await amount.fill("-4 - 2");
    await amount.press("Enter");
    await expect(page.getByLabel("HP summary", { exact: true })).toHaveText("34/40");
    await expect(amount).toHaveValue("");
    await expect(amount).toBeFocused();
    await amount.fill("4 + 2");
    await amount.press("Enter");
    await expect(page.getByLabel("HP summary", { exact: true })).toHaveText("40/40");
    await expect(amount).toHaveValue("");
    await expect(amount).toBeFocused();
    await amount.fill("4 + 2");
    await activate(page.getByRole("button", { name: "Damage", exact: true }), info);
    await expect(page.getByLabel("HP summary", { exact: true })).toHaveText("34/40");
    if ((page.viewportSize()?.width ?? 1440) >= 768)
      await page.setViewportSize({ width: 320, height: 700 });
    if (await back.isVisible()) await expectParticipantWidth(page);
    await noPageOverflow(page);
  } finally {
    expect((await page.request.delete(path)).ok()).toBeTruthy();
  }
});

async function expectParticipantWidth(page: Page) {
  // Visibility and document overflow do not catch a contained flex item
  // collapsing to its padding while its parent remains full width.
  await expect
    .poll(async () => {
      const reader = await page.locator(".encounter-record-pane").boundingBox();
      const parent = await page.locator(".workspace-pane--results").boundingBox();
      if (!reader || !parent || parent.width === 0) return 0;
      return reader.width / parent.width;
    })
    .toBeGreaterThan(0.95);
  const reader = (await page.locator(".encounter-record-pane").boundingBox())!;
  const viewport = page.viewportSize()!;
  expect(reader.width).toBeGreaterThan(viewport.width * 0.8);
  expect(reader.x).toBeGreaterThanOrEqual(0);
  expect(reader.x + reader.width).toBeLessThanOrEqual(viewport.width + 1);
}

test("named parties add everyone directly and archive views stay separate", async ({
  page,
}, info) => {
  const name = `Device party ${info.project.name} ${Date.now()}`;
  const created = await page.request.post("/api/encounters", { data: { name } });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  let partyKey: string | undefined;
  try {
    await page.goto("/encounters");
    await activate(page.getByRole("button", { name: "Parties", exact: true }), info);
    await activate(page.getByRole("button", { name: "New Party", exact: true }), info);
    let editor = page.getByRole("dialog", { name: "New Party", exact: true });
    await activate(editor.getByRole("button", { name: "Cancel", exact: true }), info);
    await expect(
      page.getByRole("button", { name: "New Party", exact: true }),
    ).toBeFocused();
    await activate(page.getByRole("button", { name: "New Party", exact: true }), info);
    editor = page.getByRole("dialog", { name: "New Party", exact: true });
    await editor.getByLabel("Party name", { exact: true }).fill(name);
    for (const [index, pc] of ["Aster", "Bram", "Cora", "Dain"].entries()) {
      await activate(editor.getByRole("button", { name: "Add PC", exact: true }), info);
      await editor.getByLabel(`PC ${index + 1}`, { exact: true }).fill(pc);
    }
    await noPageOverflow(page);
    await activate(editor.getByRole("button", { name: "Save", exact: true }), info);
    await expect(
      page
        .getByRole("dialog", { name: "Parties", exact: true })
        .getByRole("button", { name, exact: true }),
    ).toBeVisible();
    const parties = await (await page.request.get("/api/parties")).json();
    partyKey = parties.parties.find(
      (party: { name: string }) => party.name === name,
    ).party_key;
    const manager = page.getByRole("dialog", { name: "Parties", exact: true });
    if (!info.project.use.hasTouch) {
      const initialSize = page.viewportSize()!;
      await page.setViewportSize({ width: 320, height: 720 });
      const deletion = manager.getByRole("button", {
        name: `Delete ${name}`,
        exact: true,
      });
      const bounds = await deletion.boundingBox();
      expect(bounds).not.toBeNull();
      expect(bounds!.x).toBeGreaterThanOrEqual(0);
      expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(320);
      await page.setViewportSize(initialSize);
    }
    await activate(page.getByRole("button", { name: "Done", exact: true }), info);
    await expect(
      page.getByRole("button", { name: "Parties", exact: true }),
    ).toBeFocused();
    await page.goto(`/encounters/${encounter.slug}`);
    const add = page.getByRole("button", { name: "Add participants", exact: true });
    if (!info.project.use.hasTouch) {
      await add.focus();
      await page.keyboard.press("Enter");
      await expect(
        page.getByRole("menu", { name: "Add participants", exact: true }),
      ).toBeVisible();
      await page.keyboard.press("Escape");
      await expect(
        page.getByRole("menu", { name: "Add participants", exact: true }),
      ).toBeHidden();
      await expect(add).toBeFocused();
    }
    await activate(add, info);
    const item = page.getByRole("menuitem", { name: new RegExp(`${name}.*Aster`) });
    await expect(item).toBeEnabled();
    await activate(item, info);
    await expect(
      page.getByRole("dialog", { name: "Add Party", exact: true }),
    ).toHaveCount(0);
    await expect(page.locator("[data-participant-key]")).toHaveCount(4);
    const detail = await (await page.request.get(path)).json();
    expect(
      detail.participants.map((p: { display_name: string }) => p.display_name),
    ).toEqual(["Aster", "Bram", "Cora", "Dain"]);
    expect(
      detail.participants.every(
        (p: { max_hp?: number; current_hp?: number; initiative?: number }) =>
          p.max_hp == null && p.current_hp == null && p.initiative == null,
      ),
    ).toBe(true);
    await noPageOverflow(page);
    await activate(
      page.getByRole("button", { name: "Actions for Dain", exact: true }),
      info,
    );
    await activate(
      page.getByRole("menuitem", { name: "Remove Dain", exact: true }),
      info,
    );
    const removal = page.getByRole("dialog", { name: "Remove Dain?", exact: true });
    await activate(removal.getByRole("button", { name: "Remove", exact: true }), info);
    await expect(page.locator("[data-participant-key]")).toHaveCount(3);
    await page.reload();
    await expect(page.locator("[data-participant-key]")).toHaveCount(3);
    expect(
      (
        await page.request.patch(path, {
          data: {
            encounter_key: encounter.encounter_key,
            slug: encounter.slug,
            name,
            status: "archived",
          },
        })
      ).ok(),
    ).toBeTruthy();
    await page.goto("/encounters");
    await expect(
      page.getByRole("tab", { name: "Active", exact: true }),
    ).toHaveAttribute("aria-selected", "true");
    await expect(page.getByRole("link", { name, exact: true })).toHaveCount(0);
    await activate(page.getByRole("tab", { name: "Archived", exact: true }), info);
    await expect(page.getByRole("link", { name, exact: true })).toBeVisible();
    await noPageOverflow(page);
  } finally {
    await page.request.delete(path);
    if (partyKey)
      await page.request.delete(`/api/parties/${partyKey}`, {
        data: { expected_revision: 0 },
      });
  }
});

test("list entry notes stay beneath the reader and removal confirms in place", async ({
  page,
}, info) => {
  const slug = `note-review-${info.project.name}-${Date.now()}`;
  const created = await page.request.post("/api/lists", {
    data: { slug, name: "Note review" },
  });
  expect(created.ok()).toBeTruthy();
  const { list } = await created.json();
  try {
    const added = await page.request.post(`/api/lists/${list.list_key}/items`, {
      data: {
        list_ref: list.list_key,
        record_ref: sentinel,
        note: "A useful guardian for the garden",
      },
    });
    expect(added.ok()).toBeTruthy();
    await page.goto(`/lists/${slug}/records/${sentinel}`);
    const footer = page.getByRole("region", { name: "List entry note" });
    await expect(footer.getByText("A useful guardian for the garden")).toBeVisible();
    const before = await footer.boundingBox();
    const reader = page.locator(".reference-reader__scroll");
    await reader.evaluate((node) => {
      node.scrollTop = node.scrollHeight;
    });
    const after = await footer.boundingBox();
    expect(Math.abs(after!.y - before!.y)).toBeLessThan(2);
    await activate(footer.getByRole("button", { name: "Edit", exact: true }), info);
    await footer
      .getByRole("textbox", { name: "Note", exact: true })
      .fill("An ally who remembers the party");
    await activate(footer.getByRole("button", { name: "Save", exact: true }), info);
    await expect(footer.getByText("An ally who remembers the party")).toBeVisible();
    await page.screenshot({ path: info.outputPath("note-footer.png") });
    await expect(
      footer.getByRole("button", { name: "Edit", exact: true }),
    ).toBeFocused();
    await noPageOverflow(page);
    const results = page.getByRole("button", { name: "Results", exact: true });
    if (await results.isVisible()) await activate(results, info);
    const remove = page.getByRole("button", {
      name: "Remove Atlas Sentinel",
      exact: true,
    });
    const box = await remove.boundingBox();
    expect(box!.width).toBeGreaterThanOrEqual(44);
    expect(box!.height).toBeGreaterThanOrEqual(44);
    await activate(remove, info);
    await expect(page.getByText("Remove from list?", { exact: true })).toBeVisible();
    await activate(page.getByRole("button", { name: "Cancel", exact: true }), info);
    await expect(page.getByText("Remove from list?", { exact: true })).toBeHidden();
    await expect(remove).toBeFocused();
    await activate(remove, info);
    await activate(page.getByRole("button", { name: "Remove", exact: true }), info);
    await expect(remove).toBeHidden();
    expect(
      (await (await page.request.get(`/api/lists/${list.list_key}`)).json()).items,
    ).toHaveLength(0);
  } finally {
    expect(
      (await page.request.delete(`/api/lists/${list.list_key}`)).ok(),
    ).toBeTruthy();
  }
});

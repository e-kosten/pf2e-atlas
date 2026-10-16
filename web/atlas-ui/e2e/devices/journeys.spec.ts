import { expect, test, type Page } from "@playwright/test";
import {
  activate,
  landedSearch,
  landing,
  noPageOverflow,
  readThenBack,
  sentinel,
} from "./reading";

test("filters, exact late activity, Back and Results remain usable", async ({
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
  await readThenBack(page, host, info);
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
      await page.keyboard.press("Escape");
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
    await activate(page.getByRole("link", { name: "Signal Pulse", exact: true }), info);
    const content = page.getByLabel(
      info.project.name === "desktop-chromium"
        ? "Reference preview"
        : "Reference content",
      { exact: true },
    );
    await expect(
      content.getByRole("heading", { name: /^Signal Pulse\b/ }),
    ).toBeVisible();
    const ownerReturn = content.getByRole("link", {
      name: "Return to Atlas Sentinel",
      exact: true,
    });
    const widePreview = info.project.name === "desktop-chromium";
    if (widePreview) await expect(ownerReturn).toHaveCount(0);
    else await activate(ownerReturn, info);
    const host = page.locator(
      info.project.name === "desktop-chromium"
        ? ".record-preview-popover__body"
        : ".record-preview-drawer__body",
    );
    await expect(host.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
    const outer = await page.evaluate(() => scrollY);
    await readThenBack(page, host, info);
    expect(await page.evaluate(() => scrollY)).toBe(outer);
    // A real captured scroll must not restart the same ability's initial landing.
    const position = await host.evaluate((node) => node.scrollTop);
    await host.evaluate((node) => {
      node.scrollTop -= 200;
    });
    await expect
      .poll(() => host.evaluate((node) => node.scrollTop))
      .toBeCloseTo(position - 200, 0);
    const full = page.getByRole("link", {
      name: "Open reference full page",
      exact: true,
    });
    expect(
      new URL((await full.getAttribute("href"))!, page.url()).searchParams.get(
        widePreview ? "selection" : "landing",
      ),
    ).toBe(JSON.stringify(landing));
    await expect(full).toBeInViewport();
    if (!info.project.use.hasTouch) {
      // Middle activation exercises the native link destination independently
      // of the host OS's Cmd/Ctrl convention. Component tests guard both modifiers.
      const opened = page.context().waitForEvent("page", { timeout: 5000 });
      await full.click({ button: "middle" });
      const other = await opened;
      await expect(other).toHaveURL(/\/records\/browser-actors:/);
      expect(
        new URL(other.url()).searchParams.get(widePreview ? "selection" : "landing"),
      ).toBe(JSON.stringify(landing));
      await expect(page).toHaveURL(new RegExp(`/encounters/${encounter.slug}$`));
      await other.close();
    }
    await activate(
      page.getByRole("button", { name: "Close reference preview", exact: true }),
      info,
    );
    await expect(content).toBeHidden();
    await expect(
      page.getByRole("link", { name: "Signal Pulse", exact: true }),
    ).toBeFocused();
    const hp = page.getByRole("textbox", { name: "HP", exact: true });
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
    await activate(
      page.getByRole("button", { name: "Set", exact: true }).first(),
      info,
    );
    const feedback = page
      .getByRole("alert")
      .filter({ hasText: "Saved, but the encounter could not refresh" });
    await expect(feedback).toBeVisible();
    const settingsAction = page.getByRole("button", {
      name: "Encounter settings",
      exact: true,
    });
    await expect(settingsAction).toBeVisible();
    await expect(settingsAction).toBeDisabled();
    expect(writes).toBe(1);
    await page.unroute(detailRead);
    await activate(page.getByRole("button", { name: "Retry encounter refresh" }), info);
    await expect(feedback).toBeHidden();
    await expect(hp).toHaveValue("55");
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
    await expect(page.getByRole("textbox", { name: "HP", exact: true })).toBeVisible();
    const back = page.getByRole("button", { name: "Back to roster", exact: true });
    if (await back.isVisible()) {
      await expectParticipantWidth(page);
      await activate(back, info);
      await expect(row).toBeFocused();
      await expect(row).toBeInViewport();
    }
    await activate(
      page.getByRole("button", {
        name: "Actions for Tamsin the Lantern Keeper",
        exact: true,
      }),
      info,
    );
    await activate(page.getByRole("menuitem", { name: /Move before/ }), info);
    await expect(
      page.getByRole("button", { name: "Add PC", exact: true }),
    ).toBeEnabled();
    await activate(page.getByRole("button", { name: "Add PC", exact: true }), info);
    await activate(
      page
        .getByRole("dialog", { name: "Add PC", exact: true })
        .getByRole("button", { name: "Cancel", exact: true }),
      info,
    );
    await activate(
      page.getByRole("button", { name: "Encounter settings", exact: true }),
      info,
    );
    const settings = page.getByRole("dialog", {
      name: "Encounter settings",
      exact: true,
    });
    await activate(settings.locator("label").filter({ hasText: /^Name$/ }), info);
    await expect(
      settings.getByRole("textbox", { name: /^(?:\*\s*)?Name$/ }),
    ).toBeFocused();
    await activate(settings.getByRole("button", { name: "Cancel", exact: true }), info);
    await activate(
      page.getByRole("button", { name: "Add Creature", exact: true }),
      info,
    );
    const picker = page.getByRole("dialog", {
      name: "Add creature or hazard",
      exact: true,
    });
    await activate(picker.locator(`[data-record-key="${sentinel}"]`).first(), info);
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
    await amount.fill("4 + 2");
    await amount.press("Enter");
    expect(
      (await (await page.request.get(path)).json()).participants.find(
        (p: { participant_key: string }) =>
          p.participant_key === view.current_turn_participant_key,
      ).current_hp,
    ).toBe(40);
    await activate(page.getByRole("button", { name: "Damage", exact: true }), info);
    await expect(page.getByRole("textbox", { name: "HP", exact: true })).toHaveValue(
      "34",
    );
    if (info.project.name === "desktop-chromium")
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

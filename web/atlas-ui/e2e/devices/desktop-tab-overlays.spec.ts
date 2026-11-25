import { expect, test } from "./fixtures";
import { expectControlGaps } from "./controlGeometry";
import { markdownSource, replaceMarkdown } from "./markdownEditor";

test("only the visible tab searches selected record text and keyboard close follows focus", async ({
  page,
}, info) => {
  await page.addInitScript(() => {
    Object.assign(window, { __ATLAS_WINDOWS_TABS__: true });
  });
  await page.goto("/records/browser-conditions:offguard00000001");
  const paragraph = page
    .locator("article[data-selection-search] .prepared-content p")
    .first();
  await expect(paragraph).toBeVisible();
  await page.keyboard.press("Control+t");
  const search = page.getByRole("tab", { name: "Search", exact: true });
  const record = page.getByRole("tab", { name: "Off-Guard", exact: true });
  await expect(search).toHaveAttribute("aria-selected", "true");
  await search.focus();
  await page.keyboard.press("Control+Shift+Tab");
  await expect(record).toBeFocused();
  await paragraph.click({ clickCount: 3, position: { x: 4, y: 8 } });
  const text = await page.evaluate(() => window.getSelection()?.toString().trim());
  expect(text).toBeTruthy();
  const action = page.getByRole("link", { name: "Search Atlas", exact: true });
  await expect(action).toHaveCount(1);
  await info.attach("selected-record-text", {
    body: await page.screenshot(),
    contentType: "image/png",
  });
  await action.click();
  await expect(page.getByRole("searchbox", { name: "Search records" })).toHaveValue(
    text!,
  );
  await page.keyboard.press("Control+Tab");
  await expect(page.getByRole("searchbox", { name: "Search records" })).toHaveValue("");
  const empty = page.getByRole("tab", { selected: true });
  await empty.focus();
  await page.keyboard.press("Control+Shift+Tab");
  await expect(page.getByRole("tab", { selected: true })).toBeFocused();
  await page.keyboard.press("Delete");
  await expect(page.getByRole("tablist")).toHaveCount(0);
  await expect(page.getByRole("searchbox", { name: "Search records" })).toHaveValue("");
});

test("desktop shortcuts keep visible edges, equal height and even spacing as statuses appear", async ({
  page,
}) => {
  await page.addInitScript(() => {
    Object.assign(window, { __ATLAS_WINDOWS_TABS__: true });
  });
  await page.goto("/search");
  const bar = page.locator(".topbar__actions");
  const controls = bar.getByRole("button");
  const theme = bar.getByRole("button", { name: "Theme" });
  await expect(controls).toHaveCount(1);
  await expect(theme).not.toHaveCSS("border-top-color", "rgba(0, 0, 0, 0)");

  await page.evaluate(() => {
    Object.assign(window, {
      __ATLAS_DESKTOP_SETUP__: {
        token: "toolbar-fixture",
        setup: { job: 1, sequence: 1, status: "failed" },
        update: { version: "0.2.3", phase: { kind: "available" } },
      },
    });
    window.dispatchEvent(new Event("atlas-desktop-setup"));
  });
  const setup = bar.getByRole("button", { name: "Review setup" });
  await expect(setup).toBeVisible();
  await expect(bar.getByRole("button", { name: "Update available" })).toBeVisible();
  await expectControlGaps(controls, 8, 4);
  const border = await setup.evaluate(
    (button) => getComputedStyle(button).borderTopColor,
  );
  await expect(bar.getByRole("button", { name: "Settings" })).toHaveCSS(
    "border-top-color",
    border,
  );
  await expect(theme).toHaveCSS("border-top-color", border);
  const height = await theme.evaluate((button) => getComputedStyle(button).height);
  await expect(bar).toHaveCSS("height", height);
  for (const control of await controls.all())
    await expect(control).toHaveCSS("height", height);
});

test("only visible desktop tab overlays receive Escape and suspended drafts resume", async ({
  page,
}) => {
  const { encounter } = await (
    await page.request.post("/api/encounters", {
      data: { name: `Tab overlay check ${Date.now()}` },
    })
  ).json();
  try {
    await page.addInitScript(() => {
      Object.assign(window, { __ATLAS_WINDOWS_TABS__: true });
    });
    await page.goto("/search");
    const openSettings = page.getByRole("button", {
      name: "Search settings",
      exact: true,
    });
    const settings = page.getByRole("dialog", { name: "Search settings", exact: true });
    const cancel = settings.getByRole("button", { name: "Cancel", exact: true });
    await openSettings.click();
    await expect(settings).toBeVisible();

    await page.keyboard.press("Control+t");
    await openSettings.click();
    await settings.getByRole("button", { name: /Advanced settings/ }).click();
    const ranking = settings.getByRole("combobox", { name: "Search ranking" });
    await ranking.click();
    await expect(ranking).toHaveAttribute("aria-expanded", "true");

    await page.keyboard.press("Control+t");
    await expect(page.getByRole("tab")).toHaveCount(3);
    await page.getByRole("link", { name: "Encounters", exact: true }).click();
    await page.getByRole("link", { name: encounter.name, exact: true }).click();
    await page.getByRole("button", { name: "Add participants", exact: true }).click();
    const openTurn = page.getByRole("menuitem", {
      name: "Add custom turn…",
      exact: true,
    });
    await openTurn.click();
    await expect(openTurn).toBeHidden();
    const turn = page.getByRole("dialog", { name: "Add custom turn", exact: true });
    const notes = turn.getByRole("textbox", { name: "Notes", exact: true });
    await replaceMarkdown(notes, "Draft retained between tabs");
    await turn.getByRole("button", { name: "Heading", exact: true }).click();
    const menu = page.getByRole("menuitem", { name: "Heading 1", exact: true });
    await expect(menu).toBeVisible();

    await page.keyboard.press("Control+Tab");
    await cancel.focus();
    await page.keyboard.press("Escape");
    await expect(settings).toBeHidden();
    await expect(openSettings).toBeFocused();

    await page.keyboard.press("Control+Tab");
    await expect(ranking).toBeVisible();
    if ((await ranking.getAttribute("aria-expanded")) === "true") {
      await ranking.focus();
      await page.keyboard.press("Escape");
      await expect(ranking).toHaveAttribute("aria-expanded", "false");
    }
    await cancel.focus();
    await page.keyboard.press("Escape");
    await expect(settings).toBeHidden();

    await page.keyboard.press("Control+Tab");
    await expect.poll(() => markdownSource(notes)).toBe("Draft retained between tabs");
    await expect(menu).toBeHidden();
    await expect(turn).toBeVisible();
    const openPicker = turn.getByRole("button", { name: "Record link", exact: true });
    await openPicker.click();
    const picker = page.getByRole("dialog", {
      name: "Insert record link",
      exact: true,
    });
    await expect(picker).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(picker).toBeHidden();
    await expect(turn).toBeVisible();
    await expect.poll(() => markdownSource(notes)).toBe("Draft retained between tabs");

    await notes.pressSequentially("[[la");
    const suggestions = page.getByRole("listbox", { name: "Record link suggestions" });
    await expect(suggestions).toBeVisible();
    await page.keyboard.press("Control+Tab");
    await openSettings.click();
    await expect(settings).toBeVisible();
    await page.keyboard.press("Control+Shift+Tab");
    await expect(suggestions).toBeHidden();
    await expect
      .poll(() => markdownSource(notes))
      .toBe("Draft retained between tabs[[la");
    await replaceMarkdown(
      notes,
      "[Lantern Ward](record:browser-spells:spell00000000001)",
    );
    await turn.getByText("Preview", { exact: true }).click();
    await turn.getByRole("link", { name: "Lantern Ward", exact: true }).click();
    const reference = page.getByRole("dialog", { name: "Lantern Ward", exact: true });
    await expect(reference).toBeVisible();
    await page.keyboard.press("Control+Tab");
    await cancel.focus();
    await page.keyboard.press("Escape");
    await expect(settings).toBeHidden();
    await page.keyboard.press("Control+Shift+Tab");
    await expect(reference).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(reference).toBeHidden();
    await expect(turn).toBeVisible();

    await page.keyboard.press("Control+Tab");
    await openSettings.click();
    await page.keyboard.press("Control+Shift+Tab");
    await turn.getByText("Write", { exact: true }).click();
    await replaceMarkdown(notes, "");
    await turn.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(turn).toBeHidden();
    const difficulty = page.getByRole("dialog", {
      name: "Encounter difficulty",
      exact: true,
    });
    await page.getByRole("button", { name: /^Encounter difficulty:/ }).click();
    await expect(difficulty).toBeVisible();
    await page.keyboard.press("Control+Tab");
    await page.keyboard.press("Escape");
    await expect(settings).toBeHidden();
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
  }
});

test("a save that completes in an inactive desktop tab runs its close lifecycle", async ({
  page,
}) => {
  let saved: { party_key: string; revision: number } | undefined;
  let started!: () => void;
  let release!: () => void;
  const requestStarted = new Promise<void>((resolve) => {
    started = resolve;
  });
  const holdSave = new Promise<void>((resolve) => {
    release = resolve;
  });
  try {
    await page.addInitScript(() => {
      Object.assign(window, { __ATLAS_WINDOWS_TABS__: true });
    });
    await page.goto("/search");
    await page.getByRole("button", { name: "Search settings", exact: true }).click();
    const settings = page.getByRole("dialog", { name: "Search settings", exact: true });
    await expect(settings).toBeVisible();
    await page.keyboard.press("Control+t");
    await page.getByRole("link", { name: "Characters", exact: true }).click();
    await page.getByRole("link", { name: "Parties", exact: true }).click();
    await page.getByRole("button", { name: "New party", exact: true }).click();
    const editor = page.getByRole("dialog", { name: "New party", exact: true });
    const name = `Pending tab party ${Date.now()}`;
    await editor.getByRole("textbox", { name: "Party name" }).fill(name);
    await page.route("**/api/parties", async (route) => {
      if (route.request().method() !== "POST") return route.continue();
      started();
      await holdSave;
      const response = await route.fetch();
      if (response.ok()) saved = await response.json();
      await route.fulfill({ response });
    });
    await editor.getByRole("button", { name: "Create party", exact: true }).click();
    await requestStarted;
    await page.keyboard.press("Control+Shift+Tab");
    const cancel = settings.getByRole("button", { name: "Cancel", exact: true });
    await cancel.focus();
    const committed = page.waitForResponse(
      (response) =>
        response.url().endsWith("/api/parties") &&
        response.request().method() === "POST",
    );
    release();
    expect((await committed).ok()).toBeTruthy();
    await expect(cancel).toBeFocused();
    await page.keyboard.press("Control+Tab");
    await expect(editor).toBeHidden();
    await expect(page.getByRole("heading", { name, exact: true })).toBeVisible();
    await expect(page.getByRole("banner").locator('a[aria-current="page"]')).toHaveText(
      "Characters",
    );
    await expect(
      page
        .getByRole("navigation", { name: "Character pages" })
        .locator('a[aria-current="page"]'),
    ).toHaveText("Parties");
    await page.getByRole("button", { name: "New party", exact: true }).click();
    await expect(editor).toBeVisible();
    await expect(editor.getByRole("textbox", { name: "Party name" })).toHaveValue("");
    await page.keyboard.press("Escape");
    await expect(editor).toBeHidden();
  } finally {
    release();
    if (saved) {
      await page.request.delete(`/api/parties/${saved.party_key}`, {
        data: { expected_revision: saved.revision },
      });
    }
  }
});

test("Settings reopens in one tab and returns to the focused encounter draft", async ({
  page,
}) => {
  const { encounter } = await (
    await page.request.post("/api/encounters", {
      data: { name: `Settings draft ${Date.now()}` },
    })
  ).json();
  try {
    await page.addInitScript(() => {
      const attachment = {
        token: "browser-fixture",
        setup: { job: 0, sequence: 0, status: "idle" },
      };
      let sequence = 0;
      const invoke = async (command: string, args: { destination?: string }) => {
        if (command === "desktop_settings_open") {
          Object.assign(window, {
            __ATLAS_DESKTOP_SETUP__: {
              ...attachment,
              settingsRequest: { id: ++sequence, destination: args.destination },
            },
          });
          window.dispatchEvent(new Event("atlas-desktop-setup"));
        } else if (command === "desktop_setup_action") {
          return { command: '& "C:\\Atlas\\atlas.exe"' };
        }
      };
      Object.assign(window, {
        __ATLAS_WINDOWS_TABS__: true,
        __ATLAS_DESKTOP_SETUP__: attachment,
        __TAURI__: { core: { invoke } },
        __TAURI_INTERNALS__: { invoke },
      });
    });
    await page.goto("/search");
    const search = page.getByRole("searchbox", { name: "Search records", exact: true });
    await search.press("Control+,");
    await expect(
      page.getByRole("heading", { name: "Settings", exact: true }),
    ).toBeVisible();
    await page.getByRole("link", { name: "Terminal access" }).click();
    const terminal = page.getByRole("heading", {
      name: "Terminal access",
      exact: true,
    });
    await expect(terminal).toBeVisible();
    await page.keyboard.press("Control+t");
    await page.getByRole("link", { name: "Encounters", exact: true }).click();
    await page.getByRole("link", { name: encounter.name, exact: true }).click();
    await page.getByRole("button", { name: "Add participants", exact: true }).click();
    await page.getByRole("menuitem", { name: "Add custom turn…", exact: true }).click();
    const turn = page.getByRole("dialog", { name: "Add custom turn", exact: true });
    const notes = turn.getByRole("textbox", { name: "Notes", exact: true });
    await replaceMarkdown(notes, "Retain this turn across Settings");
    await notes.press("Control+,");
    await expect(terminal).toBeVisible();
    await page.keyboard.press("Control+,");
    await expect(page.getByRole("tab")).toHaveCount(3);
    await page.getByRole("link", { name: "Back to Settings" }).click();
    await expect(page.getByRole("link", { name: "Terminal access" })).toBeFocused();
    await page.keyboard.press("Control+w");
    await expect(turn).toBeVisible();
    await expect(notes).toBeFocused();
    await expect
      .poll(() => markdownSource(notes))
      .toBe("Retain this turn across Settings");
    await expect(page.getByRole("tab")).toHaveCount(2);
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
  }
});

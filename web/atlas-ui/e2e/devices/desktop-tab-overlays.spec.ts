import { expect, test } from "./fixtures";
import { markdownSource, replaceMarkdown } from "./markdownEditor";

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

    await page.keyboard.press("Control+Tab");
    await openSettings.click();
    await expect(settings).toBeVisible();
    await page.keyboard.press("Control+Shift+Tab");
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
    await page.getByRole("link", { name: "Encounters", exact: true }).click();
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

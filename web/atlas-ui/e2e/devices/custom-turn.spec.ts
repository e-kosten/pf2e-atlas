import { expect, test } from "./fixtures";

test("a custom turn previews its linked notes and becomes the active turn", async ({
  page,
}) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Void Font rehearsal ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  try {
    await page.goto(`/encounters/${encounter.slug}`);
    await expect(page.getByText("Add a participant or custom turn.")).toBeVisible();
    const addTrigger = page.getByRole("button", { name: "Add participants" });
    await addTrigger.click();
    await page.getByRole("menuitem", { name: "Add custom turn…" }).click();
    const editor = page.getByRole("dialog", { name: "Add custom turn" });
    await editor.getByRole("textbox", { name: "Name" }).fill("Lair action · Void Font");
    await editor.getByRole("spinbutton", { name: "Initiative" }).fill("20");
    const expand = editor.getByRole("button", { name: "Expand editor" });
    await expand.focus();
    await expand.press("Enter");
    const expanded = page.getByRole("dialog", { name: "Notes", exact: true });
    const collapse = expanded.getByRole("button", { name: "Collapse editor" });
    await expect(collapse).toBeFocused();
    const note = page.getByRole("textbox", { name: "Notes", exact: true });
    await note.fill("## Trigger\n\nThe font pulses. Resolve ");
    const heading = expanded.getByRole("button", { name: "Heading", exact: true });
    await heading.click();
    const headings = page.getByRole("menuitem", { name: "Heading 1", exact: true });
    await expect(headings).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(headings).toBeHidden();
    await expect(expanded).toBeVisible();

    const picker = page.getByRole("dialog", { name: "Insert record link" });
    await expanded.getByRole("button", { name: "Insert record link" }).click();
    await expect(picker).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(picker).toBeHidden();
    await expect(expanded).toBeVisible();

    await note.press("End");
    await note.pressSequentially("[[");
    await expect(
      picker.getByRole("searchbox", { name: "Search records" }),
    ).toBeFocused();
    await picker.locator('[data-record-key="browser-spells:spell00000000001"]').click();
    await picker.getByRole("button", { name: "Insert link" }).click();
    await expect(picker).toBeHidden();
    await expect(note).toBeFocused();
    const beforeFormatting = await note.evaluate((textarea: HTMLTextAreaElement) => {
      const start = textarea.value.indexOf("font");
      textarea.setSelectionRange(start, start + "font ".length);
      return [textarea.selectionStart, textarea.selectionEnd];
    });
    await page.keyboard.press("Escape");
    await expect(expand).toBeFocused();
    await expand.press("Enter");
    expect(
      await note.evaluate((textarea: HTMLTextAreaElement) => [
        textarea.selectionStart,
        textarea.selectionEnd,
      ]),
    ).toEqual(beforeFormatting);
    await expanded.getByRole("button", { name: "Bold" }).click();
    await expect(note).toHaveValue(/The \*\*font\*\* pulses/);
    await expect(note).toBeFocused();
    await expanded.getByText("Preview", { exact: true }).click();
    await collapse.click();
    await expect(expand).toBeFocused();
    await expect(editor.getByRole("heading", { name: "Trigger" })).toBeVisible();
    await expect(editor.locator("strong")).toHaveText("font");
    await expect(editor.getByRole("link", { name: "Lantern Ward" })).toBeVisible();
    await editor.getByText("Write", { exact: true }).click();
    await expect(note).toHaveValue(/record:browser-spells:spell00000000001/);
    await editor.getByRole("button", { name: "Add", exact: true }).click();
    await expect(editor).toBeHidden();
    await expect(addTrigger).toBeFocused();

    await page.getByRole("button", { name: "Start turn" }).click();
    await expect(
      page.getByRole("button", { name: "Lair action · Void Font, Current turn" }),
    ).toBeVisible();
    const inspector = page.getByRole("region", { name: "Custom turn" });
    await expect(inspector).toBeVisible();
    const editTrigger = inspector.getByRole("button", { name: "Edit custom turn" });
    await editTrigger.click();
    const edit = page.getByRole("dialog", { name: "Edit custom turn" });
    await expect(edit).toBeVisible();
    await edit.getByRole("button", { name: "Expand editor" }).click();
    await expect(note).toHaveValue(/The \*\*font\*\* pulses/);
    await collapse.click();
    await expect(edit.getByRole("textbox", { name: "Name" })).toHaveValue(
      "Lair action · Void Font",
    );
    await edit.getByRole("button", { name: "Cancel" }).click();
    await expect(edit).toBeHidden();
    await expect(editTrigger).toBeFocused();
    await editTrigger.click();
    await note.fill("");
    await edit.getByRole("button", { name: "Close", exact: true }).click();
    const keep = edit.getByRole("button", { name: "Keep editing", exact: true });
    await expect(keep).toBeFocused();
    await keep.click();
    await expect(note).toHaveValue("");
    await note.focus();
    await page.keyboard.press("Escape");
    await expect(keep).toBeFocused();
    await edit.getByRole("button", { name: "Discard", exact: true }).click();
    await expect(edit).toBeHidden();
    await expect(editTrigger).toBeFocused();
    await expect(page.getByRole("heading", { name: "Trigger" })).toBeVisible();
    const link = page.getByRole("link", { name: "Lantern Ward" });
    await link.focus();
    await link.press("Enter");
    await expect(
      page.getByRole("heading", { name: "Lantern Ward", exact: true }),
    ).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(link).toBeFocused();
    await page
      .getByRole("button", { name: "Lair action · Void Font, Current turn" })
      .click();
    await addTrigger.click();
    await page.getByRole("menuitem", { name: "Add custom turn…" }).click();
    await expect(editor).toBeVisible();
    await page.setViewportSize({ width: 390, height: 640 });
    await expect(addTrigger).toBeHidden();
    await editor.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(editor).toBeHidden();
    await expect(page.locator("#atlas-main")).toBeFocused();
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
  }
});

test("closing Add custom turn protects a changed draft and returns focus", async ({
  page,
}) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Draft rehearsal ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  try {
    await page.setViewportSize({ width: 860, height: 440 });
    await page.goto(`/encounters/${encounter.slug}`);
    const trigger = page.getByRole("button", { name: "Add participants", exact: true });
    const editor = page.getByRole("dialog", { name: "Add custom turn", exact: true });
    const cancel = editor.getByRole("button", { name: "Cancel", exact: true });
    const keep = editor.getByRole("button", { name: "Keep editing", exact: true });
    const open = async () => {
      await trigger.click();
      await page
        .getByRole("menuitem", { name: "Add custom turn…", exact: true })
        .click();
      await expect(
        page.getByRole("menu", { name: "Add participants", includeHidden: true }),
      ).toBeHidden();
      await expect(editor).toBeVisible();
    };

    await open();
    const name = editor.getByRole("textbox", { name: "Name" });
    await name.fill("Reverted name");
    await name.clear();
    await cancel.click();
    await expect(editor).toBeHidden();
    await expect(trigger).toBeFocused();

    await open();
    await editor.getByRole("button", { name: "Expand editor" }).click();
    const expanded = page.getByRole("dialog", { name: "Notes", exact: true });
    await expanded.getByRole("textbox", { name: "Notes" }).fill("Keep this draft.");
    await page.keyboard.press("Escape");
    const note = editor.getByRole("textbox", { name: "Notes" });
    await expect(note).toHaveValue("Keep this draft.");
    await note.focus();
    await page.keyboard.press("Escape");
    await expect(keep).toBeFocused();
    await keep.click();
    await expect(note).toBeFocused();

    await name.fill("Failed turn");
    await editor.getByRole("spinbutton", { name: "Initiative" }).fill("20");
    await page.route(
      "**/participants/initiative-entry",
      (route) =>
        route.fulfill({
          status: 400,
          contentType: "application/json",
          body: JSON.stringify({ code: "invalid_request", message: "Add failed." }),
        }),
      { times: 1 },
    );
    await editor.getByRole("button", { name: "Add", exact: true }).click();
    await expect(editor.getByText("Custom turn could not be saved")).toBeVisible();
    await expect(note).toHaveValue("Keep this draft.");
    await cancel.click();
    await expect(keep).toBeFocused();
    const discard = editor.getByRole("button", { name: "Discard", exact: true });
    await expect(discard).toBeInViewport({ ratio: 1 });
    await discard.click();
    await expect(editor).toBeHidden();
    await expect(trigger).toBeFocused();
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
  }
});

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
    await page.getByRole("button", { name: "Add participants" }).click();
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

    await page.getByRole("button", { name: "Start turn" }).click();
    await expect(
      page.getByRole("button", { name: "Lair action · Void Font, Current turn" }),
    ).toBeVisible();
    const inspector = page.getByRole("region", { name: "Custom turn" });
    await expect(inspector).toBeVisible();
    await inspector.getByRole("button", { name: "Edit custom turn" }).click();
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
    await expect(page.getByRole("heading", { name: "Trigger" })).toBeVisible();
    const link = page.getByRole("link", { name: "Lantern Ward" });
    await link.focus();
    await link.press("Enter");
    await expect(
      page.getByRole("heading", { name: "Lantern Ward", exact: true }),
    ).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(link).toBeFocused();
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
  }
});
